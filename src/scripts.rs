//! Modifies the game's script system to run CLEO scripts alongside vanilla scripts, and provides
//! an API for interfacing with the script system.

use crate::{call_original, hook, targets, touch};
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
    bytes: Vec<u8>,
}

impl CleoScript {
    fn new(bytes: Vec<u8>) -> CleoScript {
        log::info!(
            "verify::check() returned {:#?}",
            crate::check::check_bytecode(&bytes)
        );

        CleoScript {
            game_script: GameScript::new(bytes.as_ptr().cast(), false),
            bytes,
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
                // Simply deactivate the script.
                self.game_script.active = false;
                true
            }

            0xdd0..=0xdd4 | 0xdde | 0xdd8..=0xdda | 0xdd7 => {
                log::error!("opcode {:#x} unsupported on iOS", opcode);
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
                    log::warn!("returning invalid touch state for zone {}", zone);
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
                    log::warn!("returning invalid touch state for zone {}", zone);
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
    Invoked(CleoScript, String),

    // todo: Add support for PC CS scripts.
    /// CSA scripts. These start when the game has finished loading.
    Running(CleoScript),
}

// fixme: We shouldn't need to implement Sync/Send manually.
unsafe impl Sync for Script {}
unsafe impl Send for Script {}

impl Script {
    fn update_all(scripts: &mut [Script]) {
        for script in scripts {
            let script = match script {
                Script::Invoked(script, _) => script,
                Script::Running(script) => script,
            };

            script.update();
        }
    }
}

lazy_static::lazy_static! {
    static ref SCRIPTS: Mutex<Vec<Script>> = Mutex::new(vec![]);
}

fn load_script(path: &impl AsRef<std::path::Path>) -> std::io::Result<CleoScript> {
    Ok(CleoScript::new(std::fs::read(path)?))
}

pub fn load_running_script(path: &impl AsRef<std::path::Path>) -> std::io::Result<()> {
    SCRIPTS
        .lock()
        .unwrap()
        .push(Script::Running(load_script(path)?));

    Ok(())
}

pub fn load_invoked_script(path: &impl AsRef<std::path::Path>) -> std::io::Result<()> {
    SCRIPTS.lock().unwrap().push(Script::Invoked(
        load_script(path)?,
        path.as_ref()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    ));

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
            Script::Invoked(script, _) => {
                script.game_script.active = false;
                script.reset();
            }
            Script::Running(script) => {
                script.game_script.active = true;
                script.reset();
            }
        }
    }
}

/// Information to be displayed in the script menu for a given script.
pub struct MenuInfo {
    pub name: String,
    pub running: bool,
    // todo: Introduce 'warning' field for MenuInfo (Option<String>).
}

impl MenuInfo {
    pub fn all() -> Vec<MenuInfo> {
        SCRIPTS
            .lock()
            .unwrap()
            .iter()
            .map(MenuInfo::new)
            .flatten()
            .collect()
    }

    fn new(script: &Script) -> Option<MenuInfo> {
        match script {
            Script::Invoked(script, name) => Some(MenuInfo {
                name: name.clone(),
                running: script.game_script.active,
            }),
            _ => None,
        }
    }

    pub fn activate(&mut self) {
        // A linear search by name is fine, because this shouldn't be called from
        //  performance-critical code.
        for script in SCRIPTS.lock().unwrap().iter_mut() {
            if let Script::Invoked(script, name) = script {
                if name != &self.name {
                    continue;
                }

                script.game_script.active = true;
                break;
            }
        }

        self.running = !self.running;
    }
}

pub fn hook() {
    targets::script_tick::install(script_update);
    targets::reset_before_start::install(script_reset);
}
