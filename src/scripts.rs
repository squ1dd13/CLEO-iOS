use std::fs;
use std::iter::FromIterator;
use std::path::Path;
use std::path::{Component, PathBuf};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io,
};

use log::{debug, error, info, warn};

use crate::{call_original, files, hook};

/// Passive scripts have the extension "csi" and are invoked via the script menu.
#[allow(dead_code)]
struct PassiveScript {
    path: String,
    pub name: String,
}

impl PassiveScript {
    #[allow(dead_code)]
    fn new(path_string: String) -> PassiveScript {
        let path = PathBuf::from(&path_string);

        // We shouldn't get invalid names, but if we do, the default is just "???".
        let mut name: String = "???".into();

        if let Some(Component::Normal(string)) = path.components().last() {
            if let Some(string) = string.to_str() {
                name = string.into();
            }
        }

        if name == "???" {
            // Report the invalid path, since this is still an error.
            warn!("Unable to get PS name from path: {}", path_string);
        }

        PassiveScript {
            path: path_string,
            name,
        }
    }
}

/// A loaded game script. This struct is compatible with the game's representation of loaded scripts,
/// but does not use all the fields that it could. As such, not all game functions will work with CLEO scripts.
/// Scripts used in Rust should be constructed in Rust (to avoid confusion about memory management
/// responsibilities between languages). Scripts from CLEO should never be mixed with vanilla scripts
/// to avoid situations where the owner of a script is unknown.
#[repr(C, align(8))]
#[derive(Debug)]
struct VanillaScript {
    // Do not use these: scripts should never be linked.
    next: usize,     //Option<Box<VanillaScript>>,
    previous: usize, //Option<Box<VanillaScript>>,

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
pub struct Script {
    /// C structure mirroring standard GTA scripts. Interoperable with game code.
    vanilla_rep: VanillaScript,

    /// Identifies the component which is responsible for loading and unloading this script.
    /// Component IDs are unique to components, not scripts, so multiple scripts may share
    /// the same component ID.
    component_id: u64,
}

// fixme: We should be using a mutex for accessing LOADED_SCRIPTS.
static mut LOADED_SCRIPTS: Vec<Script> = vec![];

pub fn loaded_scripts() -> &'static mut Vec<Script> {
    unsafe { &mut LOADED_SCRIPTS }
}

impl Script {
    pub fn new(bytes: *mut u8, component_id: u64) -> Script {
        Script {
            vanilla_rep: VanillaScript {
                name: *b"a script",
                base_ip: bytes,
                ip: bytes,
                call_stack: [std::ptr::null_mut(); 8],
                stack_pos: 0,
                active: true,

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

            component_id,
        }
    }

    pub fn name(&self) -> String {
        let name_iter = self.vanilla_rep.name.iter();
        let name_chars = name_iter.take_while(|c| c != &&0u8).map(|c| *c as char);

        String::from_iter(name_chars)
    }

    fn collect_value_args(&mut self, count: u32) {
        hook::slide::<fn(*mut Script, u32)>(0x1001cf474)(&mut *self, count)
    }

    fn read_variable_arg<T: Copy>(&mut self) -> T {
        hook::slide::<fn(*mut Script) -> T>(0x1001cfb04)(&mut *self)
    }

    fn update_bool_flag(&mut self, value: bool) {
        hook::slide::<fn(*mut Script, bool)>(0x1001df890)(&mut *self, value)
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

                let state = if let Some(state) = crate::ui::query_zone(zone) {
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

                let out = if let Some(state) = crate::ui::query_zone(zone) {
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
        };

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

        type Handler = extern "C" fn(*mut VanillaScript, u16) -> u8;

        // Find the correct handler and call it.
        if opcode >= 0xa8c {
            // Handled by the default handler.
            return hook::slide::<Handler>(0x10020980c)(&mut self.vanilla_rep, opcode);
        }

        // Other opcodes have their handlers calculated. This formula is compiler-optimised, and
        //  I'm too lazy to figure out what it actually is.
        let offset = (((opcode as usize) * 1374389535usize) >> 33) & 0x3ffffff0;

        // Add the offset to the table pointer.
        let handler_entry: *const Handler = hook::slide(0x1005c11d8 + offset);
        let handler: Handler = unsafe { handler_entry.read() };

        let next_ptr: *const usize = &self.vanilla_rep.next;

        // todo: Clean up this mess.
        let receiver = unsafe {
            ((next_ptr) as usize
                + ((*hook::slide::<*const usize>(0x1005c11d8 + offset + 8)) >> 1usize))
                as *mut VanillaScript
        };

        let self_ptr = &mut self.vanilla_rep as *mut VanillaScript;

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
        loaded.retain(|script| script.vanilla_rep.active);
        let unloaded_count = count_before - loaded.len();

        if unloaded_count != 0 {
            info!("Unloaded {} inactive scripts.", unloaded_count);
        }
    }

    fn script_tick() {
        Script::unload_inactive();

        let game_time = Script::get_game_time();

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

    fn reset(&mut self) {
        // Reset everything other than the script bytes.
        self.vanilla_rep = VanillaScript {
            next: 0,
            previous: 0,
            name: *b"a script",
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

        // Avoid allocating new slices; reuse the old ones.
        self.vanilla_rep.call_stack.fill(std::ptr::null_mut());
        self.vanilla_rep.locals.fill(0);

        self.vanilla_rep.timers[0] = 0;
        self.vanilla_rep.timers[1] = 0;
    }
}

pub struct ScriptComponent {
    /// Shared bytecode storage for all instances of the script.
    bytes: Vec<u8>,

    /// ID matching scripts which are controlled by this component.
    component_id: u64,
}

impl ScriptComponent {
    pub fn new(path: &Path) -> io::Result<Box<dyn files::Component>> {
        let is_ext_valid = match path.extension() {
            Some(ext) => matches!(ext.to_str().unwrap_or("bad"), /*"csi" | */ "csa"),
            _ => false,
        };

        if !is_ext_valid {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }

        // A single file may only contain one script, so the hash of the path makes for
        //  a good component ID.
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);

        let mut component = ScriptComponent {
            bytes: fs::read(path)?,
            component_id: hasher.finish(),
        };

        // Launch an instance. This should only ever be for CSA scripts.
        component.init();

        Ok(Box::new(component))
    }

    fn init(&mut self) {
        loaded_scripts().push(Script::new(self.bytes.as_mut_ptr(), self.component_id));
    }
}

impl files::Component for ScriptComponent {
    fn unload(&mut self) {
        let scripts = loaded_scripts();

        let length_before = scripts.len();
        scripts.retain(|script| script.component_id != self.component_id);

        let scripts_removed = length_before - scripts.len();

        debug!(
            "Unloaded {} script{} with component ID {:#x}",
            scripts_removed,
            if scripts_removed == 1 { "" } else { "s" },
            self.component_id
        );
    }

    fn reset(&mut self) {
        for script in loaded_scripts().iter_mut() {
            if script.component_id == self.component_id {
                script.reset();
            }
        }
    }
}

pub fn hook() {
    debug!("Installing script hooks");
    crate::targets::script_tick::install(Script::script_tick);
}
