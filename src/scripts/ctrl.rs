use super::{base, game, js};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};

/// A connection between this module and a JavaScript-based script that allows us to
/// receive requests and send back responses.
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
                // A JS-based script should never have more than one instruction in it, so 1000 bytes is plenty of space.
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

        let response = match request {
            js::ReqMsg::ExecInstr(opcode, args) => {
                // 0. Clear stuff that could affect instruction behaviour (flags, bytecode etc.)
                // 1. Assemble the instruction
                // 2. Put the bytecode into the puppet script
                // 3. Execute a single instruction from the puppet script
                todo!()
            }
            js::ReqMsg::GetVar(_) => todo!(),
            js::ReqMsg::SetVar(_, _) => todo!(),
            js::ReqMsg::ReportErr(err) => {
                self.conn.send(Some(js::RespMsg::Exit));
                return Err(err);
            }
            js::ReqMsg::JoinHandle(handle) => {
                self.join_handle = Some(handle);
                None
            }
        };

        self.conn.send(response);

        todo!()
    }

    fn is_ready(&self) -> bool {
        todo!()
    }

    fn wakeup_time(&self) -> base::GameTime {
        todo!()
    }

    fn reset(&mut self) {
        // We're not going to respond to any more requests from `exec_single` (since it's not
        // going to be called again), so just tell the script to exit. Next time it sends a
        // message and checks for a response, it'll get this message and exit.
        self.conn.send(Some(js::RespMsg::Exit));

        // The script may also report an error on exiting, and if it does, it'll hang while
        // it waits for a reply. To stop that happening, we just send a message now that will
        // be consumed when the error is reported, allowing the script to exit.
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
}

/// A structure that manages a group of scripts.
struct ScriptRuntime {
    scripts: Vec<Box<dyn base::Script>>,
}

impl ScriptRuntime {
    fn new(scripts: Vec<Box<dyn base::Script>>) -> ScriptRuntime {
        ScriptRuntime { scripts }
    }

    /// Updates each script in turn.
    fn update(&mut self) -> Result<()> {
        for script in &mut self.scripts {
            script.exec_block()?;
        }

        Ok(())
    }

    /// Resets all of the managed scripts.
    fn reset(&mut self) {
        for script in &mut self.scripts {
            script.reset();
        }
    }
}
