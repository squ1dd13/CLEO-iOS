// fixme: This code is much older and lower-quality than the rest of the project.

use log::trace;
use log::{debug, error, info, warn};

use crate::touch;
use crate::{call_original, hook};

/// A loaded game script. This struct is compatible with the game's representation of loaded scripts,
/// but does not use all the fields that it could. As such, not all game functions will work with CLEO scripts.
/// Scripts used in Rust should be constructed in Rust (to avoid confusion about memory management
/// responsibilities between languages). Scripts from CLEO should never be mixed with vanilla scripts
/// to avoid situations where the owner of a script is unknown.
#[repr(C, align(8))]
#[derive(Debug)]
struct GameScript {
    // Do not use these: scripts should never be linked.
    next: usize,
    previous: usize,

    name: [u8; 8],
    base_ip: *mut u8,
    ip: *mut u8,

    call_stack: [*mut u8; 8],
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

#[derive(Debug)]
pub struct CleoScript {
    /// C structure mirroring standard GTA scripts. Interoperable with game code.
    vanilla_rep: GameScript,

    /// The bytes loaded from the script file that represent compiled code.
    bytecode: Vec<u8>,

    /// Whether or not this script is an injected (.csi) script. Injected scripts stay loaded
    /// all the time, but are marked as inactive when they are not currently executing.
    /// Typical behaviour is to unload inactive scripts, but we don't want that for injected
    /// scripts, so this field should be checked before unloading an inactive script.
    pub injected: bool,

    /// The name of the script to show in the script menu. This only really matters for
    /// injected scripts.
    pub display_name: String,
}

// fixme: We /really/ need to make scripts thread-safe.
static mut LOADED_SCRIPTS: Vec<CleoScript> = vec![];

pub fn loaded_scripts() -> &'static mut Vec<CleoScript> {
    unsafe { &mut LOADED_SCRIPTS }
}

impl CleoScript {
    pub fn new(mut bytecode: Vec<u8>, display_name: String, injected: bool) -> CleoScript {
        CleoScript {
            vanilla_rep: GameScript {
                name: *b"a script",
                base_ip: bytecode.as_mut_ptr(),
                ip: bytecode.as_mut_ptr(),
                call_stack: [std::ptr::null_mut(); 8],
                stack_pos: 0,

                // Injected scripts stay inactive until they are invoked by the user.
                active: !injected,

                next: 0,
                previous: 0,
                locals: [0; 40],
                timers: [0; 2],
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
            },

            bytecode,
            injected,
            display_name,
        }
    }

    pub fn name(&self) -> String {
        let name_iter = self.vanilla_rep.name.iter();
        let name_chars = name_iter.take_while(|c| c != &&0u8).map(|c| *c as char);

        name_chars.collect()
    }

    fn collect_value_args(&mut self, count: u32) {
        hook::slide::<fn(*mut CleoScript, u32)>(0x1001cf474)(&mut *self, count)
    }

    fn read_variable_arg<T: Copy>(&mut self) -> T {
        hook::slide::<fn(*mut CleoScript) -> T>(0x1001cfb04)(&mut *self)
    }

    fn update_bool_flag(&mut self, value: bool) {
        hook::slide::<fn(*mut CleoScript, bool)>(0x1001df890)(&mut *self, value)
    }

    fn run_override(&mut self, opcode: u16) -> bool {
        match opcode {
            // We intercept terminate() instructions to stop the game trying to free our scripts' memory.
            0x4e => {
                // Set the script to inactive so we free it on the next tick.
                self.vanilla_rep.active = false;
                true
            }

            0xdd0..=0xdd4 | 0xdde => {
                error!(
                    "Gamecode interop unsupported on iOS! (Script '{}' used opcode {:#x}.)",
                    self.name(),
                    opcode
                );

                // todo: Don't crash on unsupported CLEO opcodes.

                true
            }

            0xdd8..=0xdda | 0xdd7 => {
                // In theory, 0xdda could be used to get valid memory addresses, but scripts are
                //  probably looking for bytes only present in the 32-bit game (if looking for
                //  functions).

                // todo: Allow memory operations on addresses within script variable space.

                error!(
                    "Memory r/w unsupported on iOS! (Script '{}' used opcode {:#x}.)",
                    self.name(),
                    opcode
                );

                true
            }

            0xddc => {
                // !!! Mutex instructions not implemented correctly.
                self.collect_value_args(2);
                true
            }

            0xe1 => {
                self.collect_value_args(2);

                let zone = unsafe { *hook::slide::<*const u32>(0x1007ad690 + 4) as usize };

                let state = if let Some(state) = touch::query_zone(zone) {
                    state
                } else {
                    warn!("Returning invalid touch state!");
                    false
                };

                self.update_bool_flag(state);
                true
            }

            0xde0 => {
                let destination: *mut i32 = self.read_variable_arg();
                self.collect_value_args(2);

                let zone = unsafe { *hook::slide::<*const u32>(0x1007ad690) as usize };

                let out = if let Some(state) = touch::query_zone(zone) {
                    state as i32
                } else {
                    warn!("Invalid touch state!");
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

    fn run_next(&mut self) -> u8 {
        if !self.vanilla_rep.active {
            return 1;
        }

        if self.vanilla_rep.ip.is_null() {
            panic!("Instruction pointer may not be null!");
        }

        let instruction = unsafe {
            let instruction = (self.vanilla_rep.ip as *mut u16).read();
            self.vanilla_rep.ip = self.vanilla_rep.ip.offset(2);

            instruction
        };

        self.vanilla_rep.not_flag = instruction & 0x8000 != 0;

        let opcode = (instruction & 0x7fff) as u16;

        if self.run_override(opcode) {
            return 1;
        }

        type Handler = extern "C" fn(*mut GameScript, u16) -> u8;

        // Find the correct handler and call it.
        if opcode >= 0xa8c {
            // Handled by the default handler.
            return hook::slide::<Handler>(0x10020980c)(&mut self.vanilla_rep, opcode);
        }

        // Each function handles 100 opcodes, so for every 100 we go up in the opcode, the offset
        //  in the function table goes up by 16 (as each entry has 16 bytes).
        let offset = opcode as usize / 100 * 16;

        // Add the offset to the table pointer.
        let handler_entry: *const Handler = hook::slide(0x1005c11d8 + offset);
        let handler: Handler = unsafe { handler_entry.read() };

        let next_ptr: *const usize = &self.vanilla_rep.next;

        // todo: Clean up this mess.
        let receiver = unsafe {
            ((next_ptr) as usize
                + ((*hook::slide::<*const usize>(0x1005c11d8 + offset + 8)) >> 1usize))
                as *mut GameScript
        };

        let self_ptr = &mut self.vanilla_rep as *mut GameScript;

        if receiver != self_ptr {
            warn!(
                "receiver ({:#x}) != self_ptr ({:#x})",
                receiver as usize, self_ptr as usize
            );
        }

        handler(receiver, opcode)
    }

    pub fn run_block(&mut self) {
        loop {
            if self.run_next() != 0 {
                break;
            }
        }
    }

    fn get_game_time() -> u32 {
        unsafe { *hook::slide::<*const u32>(0x1007d3af8) }
    }

    fn unload_inactive() {
        // Unload scripts that are not active. This code works in conjunction with the
        //  opcode override for terminate(), which marks scripts as inactive instead of
        //  actually terminating them. This means that CSI scripts that have ended
        //  will be unloaded here.
        // todo: Log a warning when CSA scripts get unloaded.
        let loaded = loaded_scripts();

        let count_before = loaded.len();

        loaded.retain(|script| {
            // We want to retain injected scripts whether or not they are active.
            script.injected || script.vanilla_rep.active
        });

        let unloaded_count = count_before - loaded.len();

        if unloaded_count != 0 {
            info!("Unloaded {} inactive scripts.", unloaded_count);
        }
    }

    fn script_tick() {
        CleoScript::unload_inactive();

        let game_time = CleoScript::get_game_time();

        for script in loaded_scripts() {
            // Only run scripts if their activation time is not in the future.
            if script.vanilla_rep.wakeup_time > game_time {
                // The script is sleeping, so leave it alone.
                continue;
            }

            // Run the next block of instructions.
            script.run_block();
        }

        call_original!(crate::targets::script_tick);
    }

    pub fn is_active(&self) -> bool {
        self.vanilla_rep.active
    }

    pub fn activate(&mut self) {
        self.vanilla_rep = GameScript {
            ip: self.vanilla_rep.base_ip,
            next: 0,
            previous: 0,
            stack_pos: 0,
            active: true,
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
            ..self.vanilla_rep
        };

        self.vanilla_rep.call_stack.fill(std::ptr::null_mut());

        // fixme: Do we need to do this?
        self.vanilla_rep.timers[0] = 0;
        self.vanilla_rep.timers[1] = 0;
    }

    pub fn reset(&mut self) {
        // Reset everything other than the script bytes.
        self.vanilla_rep = GameScript {
            ip: self.vanilla_rep.base_ip,
            next: 0,
            previous: 0,
            name: *b"a script",
            stack_pos: 0,
            active: !self.injected,
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
            ..self.vanilla_rep
        };

        // Avoid allocating new slices by reusing the old ones.
        self.vanilla_rep.call_stack.fill(std::ptr::null_mut());
        self.vanilla_rep.locals.fill(0);

        self.vanilla_rep.timers[0] = 0;
        self.vanilla_rep.timers[1] = 0;
    }
}

fn load_script(path: &impl AsRef<std::path::Path>, injected: bool) -> std::io::Result<()> {
    let script = CleoScript::new(
        std::fs::read(path)?,
        path.as_ref()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        injected,
    );

    unsafe {
        LOADED_SCRIPTS.push(script);
    }

    Ok(())
}

pub fn load_startup_script(path: &impl AsRef<std::path::Path>) -> std::io::Result<()> {
    load_script(path, false)
}

pub fn load_invoked_script(path: &impl AsRef<std::path::Path>) -> std::io::Result<()> {
    load_script(path, true)
}

fn reset_before_start() {
    trace!("Reset");
    call_original!(crate::targets::reset_before_start);

    for script in unsafe { LOADED_SCRIPTS.iter_mut() } {
        script.reset();
    }
}

pub fn hook() {
    debug!("Installing script hooks");
    crate::targets::script_tick::install(CleoScript::script_tick);
    crate::targets::reset_before_start::install(reset_before_start);
}
