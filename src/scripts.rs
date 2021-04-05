use std::fs;
use std::io;
use std::iter::FromIterator;
use std::path::Path;
use std::path::{Component, PathBuf};

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

impl Script {
    pub fn new(path: &Path) -> io::Result<Script> {
        let is_ext_valid = match path.extension() {
            Some(ext) => matches!(ext.to_str().unwrap_or("bad"), "csi" | "csa"),
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

    fn into_inner(&self) -> &VanillaScript {
        &self.vanilla_rep
    }

    fn get_mut(&mut self) -> &mut VanillaScript {
        &mut self.vanilla_rep
    }
}
