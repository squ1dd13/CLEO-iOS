use super::{
    base::{self, Script},
    scm,
};
use anyhow::Result;

/// A script structure that manages a raw game script while providing a high-level interface.
pub struct CleoScript {
    /// The game script that this script uses to execute code.
    game_script: GameScript,

    /// The compiled code that `game_script` is running.
    bytecode: Vec<u8>,

    /// The name of the script. This will be shown to the user in the script menu.
    name: String,

    /// A record of any potential stability issues this script has.
    compat: scm::CompatReport,
}

type OpcodeFn = fn(&mut CleoScript, u16) -> Result<base::FocusWish>;

impl CleoScript {
    pub fn new(name: String, bytes: &mut impl std::io::Read) -> Result<CleoScript> {
        // Expect 1k of bytecode.
        let mut bytecode = Vec::with_capacity(1000);
        bytes.read_to_end(&mut bytecode)?;

        // It's unlikely that scripts will ever cause memory issues, but since the user may
        // have a lot of scripts, we'll try to make them as small as is reasonable.
        bytecode.shrink_to_fit();

        // Analyse the bytecode so we can warn the user about any possible problems later.
        let compat = scm::CompatReport::new(&bytecode)?;

        Ok(CleoScript {
            game_script: GameScript {
                base_ip: bytecode.as_ptr().cast(),
                ip: bytecode.as_ptr().cast(),
                ..Default::default()
            },
            bytecode,
            name,
            compat,
        })
    }

    fn opcode_func_base(script: &mut CleoScript, opcode: u16) -> Result<base::FocusWish> {
        type Handler = fn(*mut GameScript, u16) -> u8;

        // All opcodes >= 0xa8c are handled by a single function.
        let handler = if opcode >= 0xa8c {
            crate::hook::slide::<Handler>(0x10020980c)
        } else {
            let handler_table: *const Handler = crate::hook::slide(0x1005c11d8);

            // Each function handles 100 commands.
            let handler_index = opcode / 100;

            // Multiply by two because the table alternates between null pointers and function pointers,
            //  so each entry is actually 16 bytes (two pointers = 2 * 8).
            let handler_offset = handler_index as usize * 2;

            unsafe { handler_table.add(handler_offset).read() }
        };

        Ok(if handler(&mut script.game_script, opcode) == 0 {
            base::FocusWish::RetainFocus
        } else {
            base::FocusWish::MoveOn
        })
    }

    fn opcode_func(opcode: u16) -> Result<OpcodeFn> {
        Ok(match opcode {
            // This opcode terminates the script, but we have to re-implement it for CLEO
            // scripts so the game doesn't try to `free()` Rust-allocated memory.
            0x4e => |script, _| {
                script.reset();
                Ok(base::FocusWish::MoveOn)
            },

            // 0xddc | 0xddd => |script, opcode| {
            //     static MUTEX_VARS: OnceCell<[u32; 32]> = OnceCell::new();

            //     if opcode == 0xddc {
            //         let mutex_vars = MUTEX_VARS.get_or_init(|| [0; 32]);

            //         for arg in script.read_args().take(2) {

            //         }
            //     }

            //     Ok(base::FocusWish::RetainFocus)
            // },
            0xddc | 0xddd | 0xe1 | 0xde0 => {
                return Err(anyhow::format_err!(
                    "Opcode {:#x} not yet implemented",
                    opcode
                ))
            }

            // Some Android opcodes are unsupported on iOS.
            0xdd0..=0xdd4 | 0xdde | 0xdd8..=0xdda | 0xdd7 => {
                return Err(anyhow::format_err!(
                    "Opcode {:#x} is unsupported on iOS",
                    opcode
                ));
            }

            _ => {
                // No special case for this opcode, so just use the game's implementation.
                Self::opcode_func_base
            }
        })
    }

    fn game_time() -> base::GameTime {
        crate::hook::get_global(0x1007d3af8)
    }

    pub fn bytecode_mut(&mut self) -> &mut Vec<u8> {
        &mut self.bytecode
    }

    pub fn bool_flag(&self) -> bool {
        self.game_script.bool_flag
    }
}

impl base::Script for CleoScript {
    fn exec_single(&mut self) -> Result<base::FocusWish> {
        let (opcode, invert_return) = {
            let opcode_written = unsafe {
                let read = self.game_script.ip.read();
                self.game_script.ip = self.game_script.ip.add(1);
                read
            };

            // The lower 15 bits encode the actual opcode, while the 16th bit determines whether (1)
            // or not (0) the return value of the Boolean instruction being executed should be inverted.
            (opcode_written & 0x7fff, opcode_written & 0x8000 != 0)
        };

        self.game_script.not_flag = invert_return;

        // Find the function for executing instructions with this opcode.
        let instr_func = Self::opcode_func(opcode)?;
        instr_func(self, opcode)
    }

    fn is_ready(&self) -> bool {
        self.game_script.active && Self::game_time() >= self.game_script.wakeup_time
    }

    fn wakeup_time(&self) -> base::GameTime {
        self.game_script.wakeup_time
    }

    fn reset(&mut self) {
        // We retain only the instruction pointer, because that's all we set in `new`.
        self.game_script = GameScript {
            base_ip: self.game_script.base_ip,
            ip: self.game_script.base_ip,
            ..Default::default()
        };
    }

    fn identity(&self) -> base::Identity {
        let hash = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&self.bytecode, &mut hasher);
            std::hash::Hasher::finish(&hasher)
        };

        base::Identity::Scm(hash)
    }

    fn set_state(&mut self, state: base::State) {
        self.game_script.active = state.value();
    }

    fn name(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::from(&self.name)
    }
}

/// Mirror of the game's CRunningScript class structure. We need to use instances of this struct
/// when interfacing with game code. Most CLEO code should use our own higher-level wrappers.
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

    wakeup_time: base::GameTime,
    condition_count: u16,
    not_flag: bool,

    checking_game_over: bool,
    game_over: bool,

    skip_scene_pos: i32,
    is_mission: bool,
}

impl Default for GameScript {
    fn default() -> GameScript {
        GameScript {
            next: 0,
            previous: 0,
            name: *b"unnamed!",
            base_ip: std::ptr::null(),
            ip: std::ptr::null(),
            call_stack: [0; 8],
            stack_pos: 0,
            locals: [0; 40],
            timers: [0; 2],
            active: false,
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
