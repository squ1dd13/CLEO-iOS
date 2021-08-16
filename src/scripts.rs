//! Modifies the game's script system to run CLEO scripts alongside vanilla scripts, and provides
//! an API for interfacing with the script system.

use crate::{
    call_original,
    check::{self, CompatIssue},
    hook,
    menu::{self, MenuMessage, TabData},
    targets, touch,
};
use std::sync::Mutex;

#[repr(C, align(8))]
struct GameScript {
    // Do not use these; scripts should never be linked.
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

/// Wrapper for game-compatible script structures that allows use with both game code and CLEO code.
struct CleoScript {
    game_script: GameScript,

    /// The source of the script's data. This is kept with the game script because the pointers in the
    /// game script are to positions within this vector, so the vector can only be safely dropped once
    /// those pointers are not needed anymore.
    _bytes: Vec<u8>,

    /// The name shown to the user in the menu.
    name: String,

    /// A potential incompatibility that the script has. There may be several of these, but the `check`
    /// module only returns the first that is found.
    compat_issue: Option<check::CompatIssue>,
}

impl CleoScript {
    fn new(bytes: Vec<u8>, name: String) -> CleoScript {
        let compat_issue = match check::check_bytecode(&bytes) {
            Ok(v) => v,
            Err(err) => {
                log::error!("check_bytecode failed: {}", err);

                // It wouldn't be safe to assume that the script is valid because the check failed.
                Some(CompatIssue::CheckFailed)
            }
        };

        if let Some(issue) = &compat_issue {
            log::warn!("Compatibility issue: {}", issue);
        } else {
            log::info!("No compatibility issues detected.");
        }

        CleoScript {
            game_script: GameScript::new(bytes.as_ptr().cast(), false),
            _bytes: bytes,
            name,
            compat_issue,
        }
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

        let game_time: u32 = hook::get_global(0x1007d3af8);

        if self.game_script.wakeup_time > game_time {
            // Don't wake up yet.
            return;
        }

        while !self.update_once() {}
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

enum Script {
    /// CSI scripts. These do not run until the user tells them to using the menu.
    Invoked(CleoScript),

    // todo: Add support for PC CS scripts.
    /// CSA scripts. These start when the game has finished loading.
    Running { script: CleoScript, enabled: bool },
}

// fixme: We shouldn't need to implement Sync/Send manually.
unsafe impl Sync for Script {}
unsafe impl Send for Script {}

impl Script {
    fn update_all(scripts: &mut [Script]) {
        for script in scripts {
            let script = match script {
                Script::Invoked(script) => script,
                Script::Running { script, enabled } => {
                    if !*enabled {
                        continue;
                    }

                    script
                }
            };

            script.update();
        }
    }
}

lazy_static::lazy_static! {
    // fixme: Should we be using Arc<Mutex<...>> for the script vector?
    static ref SCRIPTS: Mutex<Vec<Script>> = Mutex::new(vec![]);
}

fn load_script(path: &impl AsRef<std::path::Path>) -> eyre::Result<CleoScript> {
    log::info!("Loading script {}", path.as_ref().display());
    Ok(CleoScript::new(
        std::fs::read(path)?,
        path.as_ref()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    ))
}

pub fn load_running_script(path: &impl AsRef<std::path::Path>) -> eyre::Result<()> {
    let script = load_script(path)?;

    // Only enable by default if the script has no compatibility issues. We have to decide here
    //  because the user is not normally in charge of launching CSA scripts.
    // todo: Allow switching between Off/On/Always On for CSA scripts in menu. Use '<' and '>' to show that there are more options available.
    let enabled = script.compat_issue.is_none();

    SCRIPTS
        .lock()
        .unwrap()
        .push(Script::Running { script, enabled });

    Ok(())
}

pub fn load_invoked_script(path: &impl AsRef<std::path::Path>) -> eyre::Result<()> {
    SCRIPTS
        .lock()
        .unwrap()
        .push(Script::Invoked(load_script(path)?));

    Ok(())
}

fn script_update() {
    Script::update_all(&mut SCRIPTS.lock().unwrap());
    call_original!(targets::script_tick);
}

fn script_reset() {
    call_original!(targets::reset_before_start);

    let mut scripts = SCRIPTS.lock().unwrap();

    for script in scripts.iter_mut() {
        match script {
            Script::Invoked(script) => {
                script.game_script.active = false;
                script.reset();
            }
            Script::Running { script, enabled: _ } => {
                script.game_script.active = true;
                script.reset();
            }
        }
    }
}

fn gen_compat_warning(invoked_errs: usize, running_errs: usize) -> Option<String> {
    fn gen_message(count: usize, extension: &str) -> String {
        const NUMBERS: &[&str] = &[
            "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
        ];

        if count == 0 {
            return "".to_string();
        }

        format!(
            "{} {} script{}",
            NUMBERS
                .get(count - 1)
                .map(|s| s.to_string())
                .unwrap_or_else(|| count.to_string()),
            extension,
            if count == 1 { "" } else { "s" }
        )
    }

    let mut output = String::new();

    if invoked_errs == 0 && running_errs == 0 {
        return None;
    }

    if invoked_errs != 0 {
        output += &gen_message(invoked_errs, "CSI");

        if running_errs != 0 {
            output += " and ";
        }
    }

    if running_errs != 0 {
        output += &gen_message(running_errs, "CSA");
    }

    let it_them = if invoked_errs + running_errs == 1 {
        "it"
    } else {
        "them"
    };

    // fixme: Not happy with the wording here. The end result sounds clunky.
    output +=
        format!(" may be incompatible with iOS. Use {} at your own risk.\nView the scripts below for more details.", it_them).as_str();

    // Make the first character uppercase.
    let mut characters = output.chars();
    let upper_first = characters
        .next()
        .unwrap()
        .to_uppercase()
        .collect::<String>();

    Some(upper_first + characters.as_str())
}

#[derive(Debug)]
pub enum ScriptState {
    Disabled,
    TempEnabled,
    AlwaysEnabled,
}

#[derive(Debug)]
pub enum ScriptStateMenu {
    Csi(bool),
    Csa(ScriptState),
}

// todo: Split MenuInfo into two different structs for the different types of script we have.

/// Information to be displayed in the script menu for a given script.
#[derive(Debug)]
pub struct MenuInfo {
    pub name: String,
    pub state: ScriptStateMenu,
    pub warning: Option<String>,
}

impl MenuInfo {
    fn new(script: &Script) -> Option<MenuInfo> {
        match script {
            Script::Invoked(script) => Some(MenuInfo {
                name: script.name.clone(),
                state: ScriptStateMenu::Csi(script.game_script.active),
                warning: script.compat_issue.as_ref().map(|issue| issue.to_string()),
            }),
            Script::Running { script, enabled } => Some(MenuInfo {
                name: script.name.clone(),
                state: ScriptStateMenu::Csa(if *enabled {
                    if script.compat_issue.is_some() {
                        ScriptState::TempEnabled
                    } else {
                        ScriptState::AlwaysEnabled
                    }
                } else {
                    ScriptState::Disabled
                }),
                warning: script.compat_issue.as_ref().map(|issue| issue.to_string()),
            }),
        }
    }

    pub fn activate(&mut self) {
        // A linear search by name is fine, because this shouldn't be called from
        //  performance-critical code.
        for script in SCRIPTS.lock().unwrap().iter_mut() {
            match script {
                Script::Invoked(script) => {
                    if script.name != self.name {
                        continue;
                    }

                    script.game_script.active = true;
                    break;
                }
                Script::Running { script, enabled } => {
                    if script.name != self.name {
                        continue;
                    }

                    *enabled = true;
                    script.game_script.active = true;

                    self.state = if *enabled {
                        ScriptStateMenu::Csa(ScriptState::TempEnabled)
                    } else {
                        ScriptStateMenu::Csa(ScriptState::Disabled)
                    }
                }
            }
        }

        if let ScriptStateMenu::Csi(state) = &self.state {
            self.state = ScriptStateMenu::Csi(!state);
        }
    }
}

impl menu::RowData for MenuInfo {
    fn title(&self) -> &str {
        &self.name
    }

    fn detail(&self) -> menu::RowDetail<'_> {
        if let Some(warning) = self.warning.as_deref() {
            menu::RowDetail::Warning(warning)
        } else {
            menu::RowDetail::Info("No compatibility issues detected.")
        }
    }

    fn value(&self) -> &str {
        match &self.state {
            ScriptStateMenu::Csi(state) => {
                if *state {
                    "CSI / Running"
                } else {
                    "CSI / Not running"
                }
            }
            ScriptStateMenu::Csa(state) => match state {
                ScriptState::Disabled => "CSA / Off",
                ScriptState::TempEnabled => "CSA / Temporarily On",
                ScriptState::AlwaysEnabled => "CSA / Always On",
            },
        }
    }

    fn tint(&self) -> Option<(u8, u8, u8)> {
        match &self.state {
            ScriptStateMenu::Csi(state) => {
                if *state {
                    Some(crate::gui::colours::GREEN)
                } else {
                    None
                }
            }
            ScriptStateMenu::Csa(state) => match state {
                ScriptState::Disabled => None,
                ScriptState::TempEnabled => Some(crate::gui::colours::GREEN),
                ScriptState::AlwaysEnabled => Some(crate::gui::colours::BLUE),
            },
        }
    }

    fn handle_tap(&mut self) -> bool {
        self.activate();
        MenuMessage::Hide.send();
        false
    }
}

pub fn tab_data() -> menu::TabData {
    let mut row_data = vec![];

    let mut csi_errs = 0usize;
    let mut csa_errs = 0usize;

    for script in SCRIPTS.lock().unwrap().iter() {
        let (cleo_script, err_inc) = match script {
            Script::Running { script, enabled: _ } => (script, &mut csa_errs),
            Script::Invoked(s) => (s, &mut csi_errs),
        };

        if cleo_script.compat_issue.is_some() {
            *err_inc += 1;
        }

        if let Some(info) = MenuInfo::new(script) {
            row_data.push(Box::new(info) as Box<dyn menu::RowData>)
        }
    }

    let warning = gen_compat_warning(csi_errs, csa_errs);

    TabData {
        name: "Scripts".to_string(),
        warning,
        row_data,
    }
}

pub fn init() {
    targets::script_tick::install(script_update);
    targets::reset_before_start::install(script_reset);
}
