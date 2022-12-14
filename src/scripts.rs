//! Modifies the game's script system to run CLEO scripts alongside vanilla scripts, and provides
//! an API for interfacing with the script system.

use once_cell::sync::Lazy;

use crate::{
    call_original,
    check::{self, ScriptIssue},
    hook,
    menu::{self, MenuMessage, TabData},
    settings::Settings,
    targets, touch,
};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::{atomic::Ordering, Mutex},
};

#[repr(C, align(8))]
#[derive(Debug)]
struct GameScript {
    // Do not use these: scripts should never be linked.
    next: usize,
    previous: usize,

    name: [u8; 8],

    base_ip: *const u16,
    ip: *const u16,

    call_stack: [usize; 8],
    stack_pos: u16,

    locals: [u32; 40],
    timers: [i32; 2],

    active: bool,
    bool_flag: bool,

    use_mission_cleanup: bool,
    is_external: bool,
    ovr_textbox: bool,

    attach_type: u8,

    wakeup_time: u32,
    condition_count: u16,
    not_flag: bool,

    checking_game_over: bool,
    game_over: bool,

    skip_scene_pos: i32,
    is_mission: bool,
}

impl GameScript {
    fn new(ip: *const u16, active: bool) -> GameScript {
        GameScript {
            next: 0,
            previous: 0,
            name: *b"unnamed!",
            base_ip: ip,
            ip,
            call_stack: [0; 8],
            stack_pos: 0,
            locals: [0; 40],
            timers: [0; 2],
            active,
            bool_flag: false,
            use_mission_cleanup: false,
            is_external: false,
            ovr_textbox: false,
            attach_type: 0,
            wakeup_time: 0,
            condition_count: 0,
            not_flag: false,
            checking_game_over: false,
            game_over: false,
            skip_scene_pos: 0,
            is_mission: false,
        }
    }
}

/// A wrapper for game-compatible script structures that allows use with both game code and CLEO code.
#[derive(Debug)]
pub struct CleoScript {
    game_script: GameScript,

    /// The source of the script's data. This is kept with the game script because the pointers in the
    /// game script are to positions within this vector, so the vector can only be safely dropped once
    /// those pointers are not needed anymore.
    pub bytes: Vec<u8>,

    /// The name shown to the user in the menu.
    pub name: String,

    /// A problem found with the script. Multiple may be reported during checking, but only one is kept.
    pub issue: Option<check::ScriptIssue>,

    /// A hash of the script's bytes. This hash can be used to identify the script.
    pub hash: u64,
}

impl CleoScript {
    fn new(bytes: Vec<u8>, name: String) -> CleoScript {
        let mut script = CleoScript {
            game_script: GameScript::new(bytes.as_ptr().cast(), false),
            bytes,
            name,
            issue: None,
            hash: 0,
        };

        script.hash = script.generate_hash();
        script
    }

    fn generate_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.bytes.hash(&mut hasher);
        hasher.finish()
    }

    fn reset(&mut self) {
        let base_ip = self.game_script.base_ip;
        let active = self.game_script.active;

        self.game_script = GameScript::new(base_ip, active);
    }

    fn update(&mut self) {
        if !self.game_script.active {
            return;
        }

        let game_time: u32 = hook::deref_global(0x1007d3af8);

        if self.game_script.wakeup_time > game_time {
            // Don't wake up yet.
            return;
        }

        if !Settings::shared().interrupt_loops.load(Ordering::SeqCst) {
            while !self.update_once() {}
            return;
        }

        // Create a shared map for offset encounters. This does not need to be shared (and it gets cleared
        //  every time this function runs), but a shared map removes the requirement for a new map to be
        //  created every update, and also means that there is enough capacity for most updates to go without
        //  extending the map. The shared map reduces the time taken for each script update by about 40%.
        static mut OFFSET_MAP_SHARED: once_cell::unsync::Lazy<HashMap<usize, usize>> =
            once_cell::unsync::Lazy::new(HashMap::new);

        // Clear the offset map and obtain a reference to it.
        let offset_encounters = unsafe {
            OFFSET_MAP_SHARED.clear();
            &mut OFFSET_MAP_SHARED
        };

        let mut previous_offset = 0usize;

        loop {
            let (offset, jumped_backwards) = {
                let offset = self.find_ip_offset();
                let jumped_backwards = offset < previous_offset;
                previous_offset = offset;

                (offset, jumped_backwards)
            };

            let encounters = if let Some(encounters) = offset_encounters.get_mut(&offset) {
                encounters
            } else {
                offset_encounters.insert(offset, 1);
                offset_encounters.get_mut(&offset).unwrap()
            };

            *encounters += 1;

            if *encounters >= 5 && jumped_backwards {
                // We've found a single offset in this run of instructions at least five times, and just
                //  jumped backwards to get to it. This must mean we are in some kind of loop.
                //
                // Loops are fine most of the time, but if they run for a long time and don't have any
                //  'wait' instructions, they cause the rest of the game to stop for a long period.
                // This can cause a little bit of lag at best, and obvious stuttering at worst.
                //
                // Since we don't get a chance to break up these loops naturally (because there are
                //  no 'wait' instructions, remember?) we need to force them to stop every now and then
                //  in order to let other scripts (and the rest of the game) get a chance to work.
                //
                // Interrupting a loop after a jump is one of the better ways of doing this, because
                //  it is the point least likely to sit between two instructions that work best running
                //  together. No solution is perfect, but we have to find some place to break the flow up.
                // A backwards jump is also a good place to break because it suggests we're at the start
                //  of the loop body.
                // bug: The script interrupt system stops some scripts working properly.
                break;
            }

            let can_interrupt = self.update_once();

            if can_interrupt {
                break;
            }
        }
    }

    fn find_ip_offset(&self) -> usize {
        self.game_script.ip as usize - self.game_script.base_ip as usize
    }

    fn update_once(&mut self) -> bool {
        let opcode = {
            let op_as_written = unsafe {
                let op = self.game_script.ip.read();
                self.game_script.ip = self.game_script.ip.add(1);
                op
            };

            self.game_script.not_flag = op_as_written & 0x8000 != 0;

            op_as_written & 0x7fff
        };

        if self.update_override(opcode) {
            // We're done here.
            return true;
        }

        type Handler = fn(*mut GameScript, u16) -> u8;

        // All opcodes >= 0xa8c are handled by a single function.
        if opcode >= 0xa8c {
            return hook::slide::<Handler>(0x10020980c)(&mut self.game_script, opcode) != 0;
        }

        let handler = {
            let handler_table: *const Handler = hook::slide(0x1005c11d8);

            // Each function handles 100 commands.
            let handler_index = opcode / 100;

            // Multiply by two because the table alternates between null pointers and function pointers,
            //  so each entry is actually 16 bytes (two pointers = 2 * 8).
            let handler_offset = handler_index as usize * 2;

            unsafe { handler_table.add(handler_offset).read() }
        };

        handler(&mut self.game_script, opcode) != 0
    }

    fn collect_value_args(&mut self, count: u32) {
        hook::slide::<fn(*mut CleoScript, u32)>(0x1001cf474)(&mut *self, count);
    }

    fn read_variable_arg<T: Copy>(&mut self) -> T {
        hook::slide::<fn(*mut CleoScript) -> T>(0x1001cfb04)(&mut *self)
    }

    fn update_bool_flag(&mut self, value: bool) {
        hook::slide::<fn(*mut CleoScript, bool)>(0x1001df890)(&mut *self, value)
    }

    /// Runs any extra code associated with the opcode. Returns true if the extra code
    /// should replace the original code completely, or false if the original code should
    /// run as well.
    fn update_override(&mut self, opcode: u16) -> bool {
        // 32 should be enough. I've only ever seen index 0 used, but since there
        //  are no details on the true number of "mutex vars" there should be, it's
        //  safest to go higher and hope we waste some memory.
        lazy_static::lazy_static! {
            static ref MUTEX_VARS: Mutex<[u32; 32]> = Mutex::new([0; 32]);
        }

        match opcode {
            // Intercept 'terminate' calls because the game's implementation will try to
            //  free our script data (which we don't want).
            0x004e => {
                self.reset();
                self.game_script.active = false;
                true
            }

            0xdd0..=0xdd4 | 0xdde | 0xdd8..=0xdda | 0xdd7 => {
                log::error!("Opcode {:#x} unsupported on iOS", opcode);

                // We know that continuing any further would lead to a crash at best, or strange behaviour caused by
                //  the script (and therefore game) being in an invalid and uncontrolled state at worst. It is best
                //  to preempt both of these things and exit the app ourselves.
                // This isn't really an issue, since to get to this point the user must have ignored the warnings
                //  about this script being incompatible with iOS.
                // todo: In the future, we should show an alert informing the user that the game will exit and telling them which script is at fault.
                crate::gui::exit_to_homescreen();

                true
            }

            0x0ddc => {
                self.collect_value_args(2);

                let (index, value) = {
                    let args: *const u32 = hook::slide(0x1007ad690);
                    (unsafe { args.read() }, unsafe { args.add(1).read() })
                };

                MUTEX_VARS.lock().unwrap()[index as usize] = value;
                true
            }

            0xddd => {
                let dest = self.read_variable_arg::<*mut u32>();
                self.collect_value_args(1);

                let index = {
                    let args: *const u32 = hook::slide(0x1007ad690);
                    unsafe { args.read() }
                };

                unsafe {
                    dest.write(MUTEX_VARS.lock().unwrap()[index as usize]);
                    true
                }
            }

            0x00e1 => {
                self.collect_value_args(2);

                let zone = unsafe { *hook::slide::<*const u32>(0x1007ad690).add(1) } as usize;

                let state = if let Some(state) = touch::query_zone(zone) {
                    state
                } else {
                    log::warn!("Returning invalid touch state for zone {}", zone);
                    false
                };

                self.update_bool_flag(state);
                true
            }

            0x0de0 => {
                let destination: *mut i32 = self.read_variable_arg();
                self.collect_value_args(2);

                let zone = unsafe { *hook::slide::<*const u32>(0x1007ad690) as usize };

                let out = if let Some(state) = touch::query_zone(zone) {
                    state as i32
                } else {
                    log::warn!("Returning invalid touch state for zone {}", zone);
                    0
                };

                unsafe {
                    *destination = out;
                }

                true
            }

            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
enum CsaState {
    EnabledNormally,
    Disabled,
    Forced,
}

impl CsaState {
    fn active(&self) -> bool {
        !matches!(self, CsaState::Disabled)
    }
}

#[derive(Debug)]
enum Script {
    /// CSI scripts. These do not run until the user tells them to using the menu.
    Csi(CleoScript),

    // todo: Add support for PC CS scripts.
    /// CSA scripts. These start when the game has finished loading.
    Csa { script: CleoScript, state: CsaState },
}

impl Script {
    pub fn get_cleo_script_mut(&mut self) -> &mut CleoScript {
        match self {
            Script::Csi(script) => script,
            Script::Csa { script, state: _ } => script,
        }
    }
}

// hack: Scripts should be properly thread-safe.
unsafe impl Sync for Script {}
unsafe impl Send for Script {}

impl Script {
    fn update_all(scripts: &mut [Script]) {
        for script in scripts.iter_mut() {
            let script = match script {
                Script::Csi(script) => script,
                Script::Csa { script, state } => {
                    if let CsaState::Disabled = state {
                        continue;
                    }

                    script
                }
            };

            // bug: Memory corruption caused by some scripts can make reading the bytes vector impossible.
            script.update();
        }
    }
}

lazy_static::lazy_static! {
    static ref SCRIPTS: Mutex<Vec<Script>> = Mutex::new(vec![]);
}

fn load_script(path: &impl AsRef<std::path::Path>) -> eyre::Result<CleoScript> {
    log::info!("Loading script {}", path.as_ref().display());
    Ok(CleoScript::new(
        std::fs::read(path)?,
        path.as_ref()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    ))
}

pub fn load_running_script(path: &impl AsRef<std::path::Path>) -> eyre::Result<()> {
    let mut script = load_script(path)?;

    // Set the script to inactive until we've checked it.
    script.game_script.active = false;

    SCRIPTS.lock().unwrap().push(Script::Csa {
        script,
        state: CsaState::Disabled,
    });

    Ok(())
}

pub fn load_invoked_script(path: &impl AsRef<std::path::Path>) -> eyre::Result<()> {
    SCRIPTS
        .lock()
        .unwrap()
        .push(Script::Csi(load_script(path)?));

    Ok(())
}

fn script_update() {
    Script::update_all(&mut SCRIPTS.lock().unwrap());
    call_original!(targets::script_tick);
}

static SAVED_STATES: Lazy<Mutex<HashMap<u64, CsaState>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn load_csa_states() {
    let bytes = match std::fs::read(crate::resources::get_documents_path("cleo_csa_states.bin")) {
        Ok(bytes) => bytes,
        Err(err) => {
            log::warn!("Unable to read CSA states: {}", err);
            return;
        }
    };

    let deserialised = match bincode::deserialize::<HashMap<u64, CsaState>>(&bytes) {
        Ok(deserialised) => deserialised,
        Err(err) => {
            log::error!("Error while deserialising CSA states: {}", err);
            return;
        }
    };

    *SAVED_STATES.lock().unwrap() = deserialised;

    log::info!("CSA states loaded successfully.");
}

fn save_csa_states() {
    let mut states = SAVED_STATES.lock().unwrap();
    states.clear();

    for script in SCRIPTS.lock().unwrap().iter() {
        if let Script::Csa { script, state } = script {
            if let Some(ScriptIssue::Duplicate(orig_name)) = &script.issue {
                log::info!(
                    "Not saving state for duplicate '{}' of script '{}'.",
                    script.name,
                    orig_name
                );
            } else {
                states.insert(script.hash, *state);
            }
        }
    }

    let bytes = bincode::serialize(&states as &HashMap<u64, CsaState>).unwrap();

    if let Err(err) = std::fs::write(
        crate::resources::get_documents_path("cleo_csa_states.bin"),
        bytes,
    ) {
        log::error!("Error while saving CSA script states: {}", err);
    } else {
        log::info!("CSA script states saved successfully.");
    }
}

fn get_csa_state(script: &CleoScript) -> CsaState {
    // If there is a state saved for the script, we use it (even if there are issues with the script).
    // This allows the user to judge whether or not they want to ignore the errors in a script.
    if let Some(state) = SAVED_STATES.lock().unwrap().get(&script.hash) {
        if let Some(ScriptIssue::Duplicate(_)) = script.issue {
            // If the script is a duplicate of another, then matching by hash won't work (because it would
            //  enable all versions of the script if the user enabled a single one), so we ignore saved states
            //  for versions marked as duplicates.
            CsaState::Disabled
        } else {
            *state
        }
    } else if script.issue.is_some() {
        CsaState::Disabled
    } else {
        CsaState::EnabledNormally
    }
}

fn script_reset() {
    call_original!(targets::reset_before_start);

    load_csa_states();

    let mut scripts = SCRIPTS.lock().unwrap();

    for script in scripts.iter_mut() {
        match script {
            Script::Csi(script) => {
                script.game_script.active = false;
                script.reset();
            }
            Script::Csa { script, state } => {
                *state = get_csa_state(script);
                script.game_script.active = state.active();

                script.reset();
            }
        }
    }
}

fn init_stage_three(p: usize) {
    log::info!("Scanning scripts.");

    let mut scripts = SCRIPTS.lock().unwrap();

    crate::check::check_all(
        scripts
            .iter_mut()
            .map(|script| script.get_cleo_script_mut())
            .collect(),
    );

    // Add the states to CSA scripts. We have to do this after script checking is complete, because
    //  `get_csa_state` produces different states for scripts with issues.
    for script in scripts.iter_mut() {
        if let Script::Csa { script, state } = script {
            *state = get_csa_state(script);
            script.game_script.active = state.active();
        }
    }

    log::info!("Finished scanning scripts. Allowing game to continue loading.");
    call_original!(crate::targets::init_stage_three, p);
}

pub struct CsiMenuInfo {
    name: String,
    state: bool,
    warning: Option<String>,
}

impl CsiMenuInfo {
    fn new(script: &Script) -> Option<CsiMenuInfo> {
        let script = if let Script::Csi(script) = script {
            script
        } else {
            return None;
        };

        Some(CsiMenuInfo {
            name: script.name.clone(),
            state: script.game_script.active,
            warning: script.issue.as_ref().map(|issue| issue.to_string()),
        })
    }

    fn activate(&mut self) {
        for script in SCRIPTS.lock().unwrap().iter_mut() {
            if let Script::Csi(script) = script {
                if script.name != self.name {
                    continue;
                }

                script.game_script.active = true;
                self.state = true;
                break;
            }
        }
    }
}

impl menu::RowData for CsiMenuInfo {
    fn title(&self) -> String {
        self.name.clone()
    }

    fn detail(&self) -> menu::RowDetail {
        let issues_str = if let Some(warning) = self.warning.as_deref() {
            warning
        } else {
            "No issues detected."
        };

        let info_str = if self.state {
            format!("Running. {}", issues_str)
        } else {
            format!("Not running. {}", issues_str)
        };

        if self.warning.is_some() {
            menu::RowDetail::Warning(info_str)
        } else {
            menu::RowDetail::Info(info_str)
        }
    }

    fn value(&self) -> &str {
        if self.state {
            "Running"
        } else {
            "Not running"
        }
    }

    fn tint(&self) -> Option<(u8, u8, u8)> {
        if self.state {
            Some(crate::gui::colours::GREEN)
        } else {
            None
        }
    }

    fn handle_tap(&mut self) -> bool {
        self.activate();

        // We don't allow queueing of scripts because we don't want to make it easy to enable multiple at the same time.
        MenuMessage::Hide.send();

        false
    }
}

struct CsaMenuInfo {
    name: String,
    state: CsaState,
    warning: Option<String>,
}

impl CsaMenuInfo {
    fn new(script: &Script) -> Option<CsaMenuInfo> {
        if let Script::Csa { script, state } = script {
            Some(CsaMenuInfo {
                name: script.name.clone(),
                state: *state,
                warning: script.issue.as_ref().map(|issue| issue.to_string()),
            })
        } else {
            None
        }
    }
}

impl menu::RowData for CsaMenuInfo {
    fn title(&self) -> String {
        self.name.clone()
    }

    fn detail(&self) -> menu::RowDetail {
        let issues_str = if let Some(warning) = self.warning.as_deref() {
            warning
        } else {
            "No issues detected."
        }
        .to_string();

        if self.warning.is_some() {
            menu::RowDetail::Warning(issues_str)
        } else {
            menu::RowDetail::Info(issues_str)
        }
    }

    fn value(&self) -> &str {
        match self.state {
            CsaState::EnabledNormally => "Enabled",
            CsaState::Disabled => "Disabled",
            CsaState::Forced => "Forced",
        }
    }

    fn tint(&self) -> Option<(u8, u8, u8)> {
        if let CsaState::Disabled = self.state {
            None
        } else {
            Some(crate::gui::colours::GREEN)
        }
    }

    fn handle_tap(&mut self) -> bool {
        // How we switch modes depends on the script's properties. If there are no errors,
        //  the only available modes are "Enabled" and "Disabled" (in that order). If there
        //  are errors, the modes are "Disabled" and "Forced".
        let new_state = if self.warning.is_none() {
            match self.state {
                CsaState::EnabledNormally => CsaState::Disabled,
                CsaState::Disabled => CsaState::EnabledNormally,
                CsaState::Forced => {
                    log::warn!("Error-free scripts should never be in 'Forced' mode. Returning to 'Disabled'.");
                    CsaState::Disabled
                }
            }
        } else {
            match self.state {
                CsaState::EnabledNormally => {
                    log::warn!("Scripts with errors should never be 'EnabledNormally'. Returning to 'Disabled'.");
                    CsaState::Disabled
                }
                CsaState::Disabled => CsaState::Forced,
                CsaState::Forced => CsaState::Disabled,
            }
        };

        self.state = new_state;

        for script in SCRIPTS.lock().unwrap().iter_mut() {
            if let Script::Csa { script, state } = script {
                if script.name != self.name {
                    continue;
                }

                if script.name != self.name {
                    continue;
                }

                *state = new_state;
                script.game_script.active = !matches!(new_state, CsaState::Disabled);
            }
        }

        save_csa_states();

        true
    }
}

fn gen_warning_string(count: usize) -> Option<String> {
    const NUMBERS: &[&str] = &[
        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    ];

    if count == 0 {
        return None;
    }

    Some(format!(
        "Problems identified with {} script{}. {} highlighted in orange.\nSee below for further details.",
        NUMBERS
            .get(count - 1)
            .map(|s| s.to_string())
            .unwrap_or_else(|| count.to_string()),
            if count == 1 { "" } else { "s" },
        if count == 1 { "This script is" } else { "These scripts are" },
    ))
}

pub fn tab_data_csa() -> menu::TabData {
    let mut row_data = vec![];

    let mut errs = 0usize;

    for script in SCRIPTS.lock().unwrap().iter() {
        let (cleo_script, err_inc) = if let Script::Csa { script, state: _ } = script {
            (script, &mut errs)
        } else {
            continue;
        };

        if cleo_script.issue.is_some() {
            *err_inc += 1;
        }

        if let Some(info) = CsaMenuInfo::new(script) {
            row_data.push(Box::new(info) as Box<dyn menu::RowData>)
        }
    }

    row_data.sort_by_cached_key(|x| x.title());

    TabData {
        name: "CSA".to_string(),
        warning: gen_warning_string(errs),
        row_data,
    }
}

pub fn tab_data_csi() -> menu::TabData {
    let mut row_data = vec![];

    let mut errs = 0usize;

    for script in SCRIPTS.lock().unwrap().iter() {
        let (cleo_script, err_inc) = if let Script::Csi(s) = script {
            (s, &mut errs)
        } else {
            continue;
        };

        if cleo_script.issue.is_some() {
            *err_inc += 1;
        }

        if let Some(info) = CsiMenuInfo::new(script) {
            row_data.push(Box::new(info) as Box<dyn menu::RowData>)
        }
    }

    row_data.sort_by_cached_key(|x| x.title());

    TabData {
        name: "CSI".to_string(),
        warning: gen_warning_string(errs),
        row_data,
    }
}

pub fn init() {
    log::info!("installing script hooks...");

    targets::script_tick::install(script_update);
    targets::reset_before_start::install(script_reset);
    targets::init_stage_three::install(init_stage_three);
}
