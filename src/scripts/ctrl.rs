use std::sync::Mutex;

use crate::scripts::base::Script;

use super::{asm, base, game, js};
use anyhow::{Context, Result};
use byteorder::WriteBytesExt;
use crossbeam_channel::{Receiver, Sender};
use once_cell::sync::{Lazy, OnceCell};

/// A connection between this module and a JavaScript-based script that allows us to receive
/// requests and send back responses.
pub struct JsConn {
    sender: Sender<Option<js::RespMsg>>,
    receiver: Receiver<js::ReqMsg>,
}

impl JsConn {
    /// Create a new connection using a sender and receiver.
    pub fn new(sender: Sender<Option<js::RespMsg>>, receiver: Receiver<js::ReqMsg>) -> JsConn {
        JsConn { sender, receiver }
    }

    fn send(&self, msg: Option<js::RespMsg>) {
        self.sender
            .send(msg)
            .expect("Failed to send response message");
    }

    fn next(&mut self) -> Option<js::ReqMsg> {
        self.receiver.try_recv().ok()
    }
}

/// A proxy for a JavaScript-based script that behaves like a full script.
struct JsScript {
    /// The connection through which the JS script can make requests that we respond to.
    conn: JsConn,

    puppet: game::CleoScript,

    join_handle: Option<std::thread::JoinHandle<()>>,
}

impl JsScript {
    /// Create a new script using the given communication structure.
    fn new(conn: JsConn) -> Result<JsScript> {
        Ok(JsScript {
            conn,
            puppet: game::CleoScript::new(
                // No name required.
                String::new(),
                // A JS-based script should never have more than one instruction in it, so 1000
                // bytes is plenty of space.
                &mut &vec![0; 1000][..],
            )?,
            join_handle: None,
        })
    }
}

impl base::Script for JsScript {
    fn exec_single(&mut self) -> Result<base::FocusWish> {
        let request = match self.conn.next() {
            Some(r) => r,

            // If there's no request to handle, just move on to the next script.
            None => return Ok(base::FocusWish::MoveOn),
        };

        use js::ReqMsg::*;
        let response = match request {
            ExecInstr(opcode, args) => {
                // Clear anything that could affect instruction behaviour and return the
                // instruction pointer to the beginning of the script.
                self.puppet.reset();

                // Create a new instruction to compile and execute.
                let instr = asm::Instr::new(opcode, args);
                let byte_count = instr.write(&mut self.puppet.bytecode_mut())?;

                // Execute the instruction we just assembled.
                let focus_wish = self.puppet.exec_single()?;

                // Clear the bytecode we created so that the next instruction is not mixed with old
                // bytes.
                self.puppet.bytecode_mut()[..byte_count].fill(0);

                // Send back the boolean flag of the script. This could be considered the return
                // value of the SCM instruction.
                let bool_flag = self.puppet.bool_flag();
                self.conn.send(Some(js::RespMsg::BoolFlag(bool_flag)));

                return Ok(focus_wish);
            }
            GetVar(_) => todo!(),
            SetVar(_, _) => todo!(),
            ReportErr(err) => {
                self.conn.send(Some(js::RespMsg::Exit));
                return Err(err);
            }
            JoinHandle(handle) => {
                self.join_handle = Some(handle);
                None
            }
        };

        self.conn.send(response);

        todo!()
    }

    fn is_ready(&self) -> bool {
        self.puppet.is_ready()
    }

    fn wakeup_time(&self) -> base::GameTime {
        self.puppet.wakeup_time()
    }

    fn reset(&mut self) {
        // We're not going to respond to any more requests from `exec_single` (since it's not going
        // to be called again), so just tell the script to exit. Next time it sends a message and
        // checks for a response, it'll get this message and exit.
        self.conn.send(Some(js::RespMsg::Exit));

        // The script may also report an error on exiting, and if it does, it'll hang while it
        // waits for a reply. To stop that happening, we just send a message now that will be
        // consumed when the error is reported, allowing the script to exit.
        self.conn.send(None);

        if let Some(join_handle) = self.join_handle.take() {
            if let Err(err) = join_handle.join() {
                log::error!("Script thread panicked on `join()`: {:?}", err);
            }
        }

        log::info!("Successfully shut down remote JS script.");

        // Reset the puppet, ready for executing more bytecode.
        self.puppet.reset();

        todo!()
    }

    fn identity(&self) -> base::Identity {
        todo!()
    }

    fn set_state(&mut self, state: base::State) {
        self.puppet.set_state(state);
    }

    fn name(&self) -> std::borrow::Cow<'_, str> {
        todo!()
    }

    fn add_flag(&mut self, flag: base::Flag) {
        self.puppet.add_flag(flag);
    }
}

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
            match res {
                ModRes::RunningScript(path) => {
                    let mut script = super::game::CleoScript::new(
                        path.display().to_string(),
                        &mut std::io::BufReader::new(std::fs::File::open(path).unwrap()),
                    )
                    .expect("Failed to load script");

                    script.set_state(base::State::Auto(true));
                    runtime.add_script(Box::new(script));
                }
                ModRes::LazyScript(path) => todo!(),
                ModRes::JsScript(path) => todo!(),
                _ => (),
            }
        }

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

pub fn init() {
    crate::targets::init_stage_three::install(ScriptRuntime::load_hook);
    crate::targets::script_tick::install(ScriptRuntime::tick_hook);
    crate::targets::reset_before_start::install(ScriptRuntime::reset_hook);
}
