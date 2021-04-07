use std::convert::TryInto;
use std::fs;
use std::io;
use std::iter::FromIterator;
use std::path::Path;
use std::path::{Component, PathBuf};

use crate::{call_original, create_soft_target, hook};

/// Passive scripts have the extension "csi" and are invoked via the script menu.
struct PassiveScript {
    path: String,
    pub name: String,
}

impl PassiveScript {
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
            super::get_log().warning(format!("Unable to get PS name from path: {}", path_string));
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
struct VanillaScript {
    // Do not use these: scripts should never be linked.
    next: Option<Box<VanillaScript>>,
    previous: Option<Box<VanillaScript>>,

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

// static_assert_macro::static_assert!(std::mem::size_of::<VanillaScript>() == 304);

pub struct Script {
    vanilla_rep: VanillaScript,

    // Store the byte vector with the vanilla script so the vector is dropped when
    // we need it to be.
    bytes: Vec<u8>,
}

pub static mut LOADED_SCRIPTS: Vec<Script> = vec![];

impl Script {
    pub fn new(path: &Path) -> io::Result<Script> {
        let is_ext_valid = match path.extension() {
            Some(ext) => matches!(ext.to_str().unwrap_or("bad"), /*"csi" | */ "csa"),
            _ => false,
        };

        if !is_ext_valid {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }

        let mut script_bytes = fs::read(path)?;

        Ok(Script {
            vanilla_rep: VanillaScript {
                name: *b"8 bytes?",
                base_ip: script_bytes.as_mut_ptr(),
                ip: script_bytes.as_mut_ptr(),
                call_stack: [std::ptr::null_mut(); 8],
                stack_pos: 0,
                active: true,

                next: None,
                previous: None,
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

            bytes: script_bytes,
        })
    }

    pub fn load_dir(path: &str) -> io::Result<Vec<io::Result<Script>>> {
        let directory: Vec<io::Result<fs::DirEntry>> = fs::read_dir(path)?.collect();
        let mut scripts = Vec::<io::Result<Script>>::with_capacity(directory.len());

        for item in directory {
            if let Ok(entry) = item {
                scripts.push(Script::new(entry.path().as_path()));
            }
        }

        Ok(scripts)
    }

    pub fn name(&self) -> String {
        let name_iter = self.vanilla_rep.name.iter();
        let name_chars = name_iter.take_while(|c| c != &&0u8).map(|c| *c as char);

        String::from_iter(name_chars)
    }

    fn run_next(&mut self) -> u8 {
        if !self.vanilla_rep.active {
            return 1;
        };

        let instruction = unsafe {
            let instruction = std::ptr::read(self.vanilla_rep.ip as *mut i16);
            self.vanilla_rep.ip = self.vanilla_rep.ip.offset(2);

            instruction
        };

        // A negative written opcode indicates that the return is inverted.
        self.vanilla_rep.not_flag = instruction < 0;

        let opcode = instruction.abs() as u16;

        // todo: Check for instruction overrides here.

        // Intercept terminate() instructions to stop the game trying to free the script's memory.
        if opcode == 0x4e {
            // Set the script to inactive so we free it on the next tick.
            self.vanilla_rep.active = false;
            return 1;
        };

        type Handler = extern "C" fn(*mut VanillaScript, u16) -> u8;

        // Find the correct handler and call it.
        if opcode >= 0xa8c {
            // Handled by the default handler.
            return crate::hook::slide::<Handler>(0x10020980c)(&mut self.vanilla_rep, opcode);
        }

        // Other opcodes have their handlers calculated. This formula is compiler-optimised, and
        //  I'm too lazy to figure out what it actually is.
        let offset = (((opcode as usize) * 1374389535usize) >> 33) & 0x3ffffff0;

        // Add the offset to the table pointer.
        let handler_entry: *const Handler = crate::hook::slide(0x1005c11d8 + offset);
        let handler: Handler = unsafe { *handler_entry };

        // todo: Implement the weird receiver behaviour that the game has.
        let self_ptr = &mut self.vanilla_rep as *mut VanillaScript;
        handler(self_ptr, opcode)
    }

    pub fn run_block(&mut self) {
        loop {
            if self.run_next() != 0 {
                break;
            }
        }
    }

    fn get_game_time() -> u32 {
        unsafe { *crate::hook::slide::<*const u32>(0x1007d3af8) }
    }

    fn script_tick() {
        unsafe {
            let count_before = LOADED_SCRIPTS.len();
            LOADED_SCRIPTS.retain(|script| script.vanilla_rep.active);
            let unloaded_count = count_before - LOADED_SCRIPTS.len();

            if unloaded_count != 0 {
                crate::get_log().normal(format!("Unloaded {} inactive scripts.", unloaded_count));
            }

            let game_time = Script::get_game_time();

            for script in LOADED_SCRIPTS.iter_mut() {
                // Only run scripts if their activation time is not in the future.
                if script.vanilla_rep.wakeup_time > game_time {
                    // The script is sleeping, so leave it alone.
                    continue;
                }

                let name = script.name();
                crate::get_log().normal(format!("Updating {}...", name));

                // Run the next block of instructions.
                script.run_block();

                crate::get_log().normal("Updated.");
            }
        }

        call_original!(crate::targets::script_tick);
    }

    pub fn install_hooks() {
        crate::get_log().normal("Installing script hooks!");
        crate::targets::script_tick::install(Script::script_tick);
    }
}

/*
rec = (self + (*(handler_address + 8) / 8));
*/
