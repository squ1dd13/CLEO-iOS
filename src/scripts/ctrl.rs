//! Manages the script runtime. It is responsible for loading and controlling all
//! scripts used by CLEO.

use crate::ui::menu::data;

use super::{
    base::{self, Script},
    js,
};
use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use std::{sync::Mutex, borrow::Cow};

/// A structure that manages a group of scripts.
struct ScriptRuntime {
    scripts: Vec<Box<dyn base::Script>>,
}

impl ScriptRuntime {
    fn shared_mut<'rt>() -> std::sync::MutexGuard<'rt, ScriptRuntime> {
        // Safety: This is safe because the scripts are never accessed from two threads at the same time.
        // (Game code uses them on the same thread that our hooks run.)
        unsafe impl Send for ScriptRuntime {}

        static SHARED: OnceCell<Mutex<ScriptRuntime>> = OnceCell::new();

        SHARED
            .get_or_init(|| Mutex::new(ScriptRuntime { scripts: vec![] }))
            .lock()
            .unwrap()
    }

    fn add_script(&mut self, script: Box<dyn base::Script>) {
        self.scripts.push(script);
    }

    /// Updates each script in turn.
    fn update(&mut self) -> Result<()> {
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

        ScriptRuntime::shared_mut()
            .update()
            .expect("Script runtime error");

        crate::call_original!(crate::targets::script_tick);
    }

    fn reset_hook() {
        crate::call_original!(crate::targets::reset_before_start);

        /*
            On reset:
              - Lazy scripts should be switched off
              - Active scripts should be returned to their user-defined state (unless they have warnings attached)
        */
        ScriptRuntime::shared_mut().reset();
    }
}

pub struct ScriptRow<'s> {
    title: &'s str,
    
    detail: Vec<&'static str>,
    value: bool,
}

impl data::RowData for ScriptRow {
    fn title(&self) -> data::CowStr {
        data::CowStr::Borrowed(self.title)
    }

    fn detail(&self) -> Vec<data::CowStr> {
        self.detail.map(data::CowStr::Borrowed)
    }

    fn value(&self) -> data::CowStr {
        if self.value {
            "On"
        } else {
            "Off"
        }
    }

    fn tint(&self) -> super::view::Tint {
        
    }
}

pub fn tabs() -> (data::TabData<()>, data::TabData<) {

}

pub fn init() {
    crate::targets::init_stage_three::install(ScriptRuntime::load_hook);
    crate::targets::script_tick::install(ScriptRuntime::tick_hook);
    crate::targets::reset_before_start::install(ScriptRuntime::reset_hook);
}
