//! Manages the script runtime. It is responsible for loading and controlling all
//! scripts used by CLEO.

use std::{borrow::Cow, collections::BTreeSet, sync::Mutex};

use anyhow::{Context, Result};
use crossbeam_channel::{Receiver, Sender};
use once_cell::sync::OnceCell;

use crate::ui::menu::{
    data::{self, RowData},
    view,
};

use super::{
    base::{self, Script},
    js,
};

/// A message used to report an interaction with a script in the menu. Contains the script name and
/// the state that the user switched it to.
type StateUpdate = (String, base::State);

/// A structure that manages a group of scripts.
struct Runtime {
    /// The scripts managed by this runtime. This is a collection of different types of scripts (as
    /// opposed to being just SCM or JS).
    scripts: Vec<Box<dyn base::Script>>,

    /// Receiver for getting messages from the menu.
    receiver: Receiver<StateUpdate>,

    /// Sender to clone and give to tab structures so that they can communicate with us.
    sender: Sender<StateUpdate>,
}

impl Runtime {
    fn shared_mut<'rt>() -> std::sync::MutexGuard<'rt, Runtime> {
        // Safety: This is safe because the scripts are never accessed from two threads at the same time.
        // (Game code uses them on the same thread that our hooks run.)
        unsafe impl Send for Runtime {}

        static SHARED: OnceCell<Mutex<Runtime>> = OnceCell::new();

        SHARED
            .get_or_init(|| {
                Mutex::new({
                    let (sender, receiver) = crossbeam_channel::unbounded();
                    Runtime {
                        scripts: vec![],
                        receiver,
                        sender,
                    }
                })
            })
            .lock()
            .unwrap()
    }

    fn add_script(&mut self, script: Box<dyn base::Script>) {
        self.scripts.push(script);
    }

    /// Updates each script in turn. Also processes any new messages that have arrived from the
    /// menu since the last update.
    fn update(&mut self) -> Result<()> {
        // Process new user interaction messages.
        for (name, state) in self.receiver.try_iter() {
            // fixme: Maybe do something better than a linear search by name here.
            // Find the script whose state we're changing.
            let script = match self.scripts.iter_mut().find(|s| s.name() == name) {
                Some(s) => s,
                None => {
                    return Err(anyhow::format_err!(
                        "Received menu update for script named '{}', but no such script found",
                        name
                    ));
                }
            };

            // Change the state to the new one from the message.
            script.set_state(state);
        }

        for script in &mut self.scripts {
            let update_start = std::time::Instant::now();

            script
                .exec_block()
                .with_context(|| format!("while updating script '{}'", script.name()))?;

            let update_end = std::time::Instant::now();
            let update_time = update_end - update_start;

            if update_time.as_millis() > 1 {
                script.add_flag(base::Flag::Slow);
                log::trace!("Update for '{}' took {:?}", script.name(), update_time);
            }
        }

        Ok(())
    }

    /// Resets all of the managed scripts.
    fn reset(&mut self) {
        for script in &mut self.scripts {
            script.reset();
        }
    }

    /// Removes all of the scripts from the runtime.
    fn clear(&mut self) {
        self.reset();
        self.scripts.clear();
    }

    fn load_hook(ptr: usize) {
        // todo: Script loading stuff.

        /*
            On load:
              - Load all scripts from files. (Don't keep scripts between loads.)
              - Check scripts for potential issues.
              - Set script default states to sensible values based on checking outcomes.
                - Enum with two variants: `State::Auto(bool)` and `State::User(bool)`
                - Scripts with issues should be off by default.
                - Other scripts on by default.
              - Load custom script states from user settings.
                - Overridden states should be saved with the path to the script (from the CLEO
                  directory) and the hash of the script bytes.
                - When loading, match states to scripts by hash. For scripts where there is another
                  script with the same hash, match by both hash and path.
        */

        let mut runtime = Self::shared_mut();
        runtime.clear();

        use crate::files::{res_iter, ModRes};

        for res in res_iter() {
            let mut script: Box<dyn base::Script> = match res {
                ModRes::RunningScript(path) => {
                    let script = super::game::CleoScript::new(
                        path.display().to_string(),
                        &mut std::io::BufReader::new(std::fs::File::open(path).unwrap()),
                    )
                    .expect("Failed to load script");

                    Box::new(script)
                }
                ModRes::LazyScript(_) => todo!(),
                ModRes::JsScript(path) => {
                    let script = js::ScriptUnit::load(path).expect("Failed to load JS script");
                    Box::new(script)
                }
                _ => continue,
            };

            script.set_state(base::State::Auto(true));
            runtime.add_script(script);
        }

        let cleo_js_bytes = include_bytes!("cleo.js");
        let mut script = js::ScriptUnit::from_bytes("cleo_js".to_string(), cleo_js_bytes).unwrap();
        script.set_state(base::State::Auto(true));
        runtime.add_script(Box::new(script));

        crate::call_original!(crate::targets::init_stage_three, ptr);
    }

    fn tick_hook() {
        // Script system error handling is very important. Invalid script behaviour can corrupt the
        // game state. At the very least we need to discard the game state by quitting to the main
        // menu, but we should also ensure that the game does not save with this invalid state.
        // todo: Prevent game saving and quit to main menu on script errors.

        Runtime::shared_mut()
            .update()
            .expect("Script runtime error");

        crate::hooks::SCRIPT_TICK.original()();
        // crate::call_original!(crate::targets::script_tick);
    }

    fn reset_hook() {
        crate::call_original!(crate::targets::reset_before_start);

        /*
            On reset:
              - Lazy scripts should be switched off
              - Active scripts should be returned to their user-defined state (unless they have
                 warnings attached)
        */
        Runtime::shared_mut().reset();
    }

    /// Returns an appropriate message to give the user based on the number of scripts with severe
    /// issues and the number with minor issues.
    fn warning_message(severe_count: usize, minor_count: usize) -> Option<data::TabMsg<'static>> {
        let found_severe = severe_count != 0;
        let found_minor = minor_count != 0;

        let string = match (found_minor, found_severe) {
            (false, false) => return None,

            // Both severe and minor issues.
            (true, true) => format!(
                "Found {} script{} with severe issues and {} with minor issues.",
                severe_count,
                if severe_count != 1 { "s" } else { "" },
                minor_count,
            ),

            // Just minor issues.
            (true, false) => format!(
                "Found {} script{} with minor issues.",
                minor_count,
                if minor_count != 1 { "s" } else { "" },
            ),

            // Just severe issues.
            (false, true) => format!(
                "Found {} script{} with severe issues.",
                severe_count,
                if severe_count != 1 { "s" } else { "" },
            ),
        };

        Some(data::TabMsg {
            text: Cow::Owned(string),

            // The warning should have the same importance as the most important issue considered
            // while generating a summary, so we use the same tints as the rows themselves.
            tint: if found_severe {
                view::Tint::Orange
            } else {
                view::Tint::Yellow
            },
        })
    }

    /// Returns the data for the script tab.
    fn tab_data<'data>(&self) -> data::TabData<'data, StateUpdate, ScriptRow> {
        // Produce a row structure for every script.
        let mut rows: Vec<ScriptRow> = self
            .scripts
            .iter()
            .map(|script| ScriptRow {
                title: script.name().to_string(),
                flags: script.flags().clone(),
                state: script.state(),
            })
            .collect();

        // Sort in alphabetical order. We cannot sort by a key because that would require cloning
        // every title.
        rows.sort_unstable_by(|left, right| left.title.cmp(&right.title));

        // Count the number of scripts that have severe or minor issues.
        // For simplicity, we count a script with any severe issues as having *only* severe issues,
        // regardless of whether it has minor issues too.
        let (severe, minor) =
            rows.iter()
                .fold((0, 0), |(severe, minor), row| match row.are_issues_bad() {
                    Some(true) => (severe + 1, minor),
                    Some(false) => (severe, minor + 1),
                    None => (severe, minor),
                });

        // Generate a warning message to give the user a heads-up about the issues.
        // Showing this at the top of the menu makes it harder for them to miss problems.
        let warning = Self::warning_message(severe, minor);

        data::TabData {
            title: Cow::Borrowed("Scripts"),
            message: warning,
            rows,
            sender: self.sender.clone(),
        }
    }
}

/// The data that is presented to the user through the script tab in the menu.
pub struct ScriptRow {
    /// The name of the script.
    title: String,

    /// The script flags, which will be shown beneath the name.
    flags: BTreeSet<base::Flag>,

    /// The script state, shown on the right-hand side of the row.
    state: base::State,
}

impl ScriptRow {
    /// Returns the severity of the script's issues in the range `0..=2`.
    fn are_issues_bad(&self) -> Option<bool> {
        // The first item in the set should be the most important, so the overall severity is based
        // on the first issue.
        self.flags.iter().next().map(base::Flag::is_severe)
    }
}

impl data::RowData<StateUpdate> for ScriptRow {
    fn title(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.title)
    }

    fn detail(&self) -> Vec<Cow<'_, str>> {
        self.flags
            .iter()
            .map(|flag| Cow::Owned(flag.to_string()))
            .collect()
    }

    fn value(&self) -> Cow<'_, str> {
        use base::State::*;

        Cow::Borrowed(match self.state {
            // States decided by the runtime.
            Auto(true) => "On (Auto)",
            Auto(false) => "Off (Auto)",

            // States that the user has set themselves.
            User(true) => "On (Custom)",
            User(false) => "Off (Custom)",

            // Invoked script states.
            Trigger(true) => "Active",
            Trigger(false) => "Inactive",
        })
    }

    fn tint(&self) -> view::Tint {
        match self.are_issues_bad() {
            // More important issues are orange.
            Some(true) => view::Tint::Orange,

            // Less important issues are yellow.
            Some(false) => view::Tint::Yellow,

            // If there are no issues, no special colouring is applied.
            None => view::Tint::White,
        }
    }

    fn tap_msg(&mut self) -> Option<StateUpdate> {
        // Show the user the opposite state (the one they wanted to switch to) and then send that
        // state to the runtime.
        self.state = self.state.opposite();
        Some((self.title.clone(), self.state))
    }
}

pub fn init() {
    crate::targets::init_stage_three::install(Runtime::load_hook);
    crate::hooks::SCRIPT_TICK.install(Runtime::tick_hook);
    // crate::targets::script_tick::install(Runtime::tick_hook);
    crate::targets::reset_before_start::install(Runtime::reset_hook);
}
