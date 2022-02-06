use anyhow::Result;

use super::{base, js};
use std::sync::mpsc::{Receiver, Sender};

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
    // We need a puppet here, but the game script structures haven't been rewritten to
    // implement the new base traits yet.
}

impl JsScript {
    /// Create a new script using the given communication structure.
    fn new(conn: JsConn) -> JsScript {
        JsScript { conn }
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
                self.conn.send(Some(js::RespMsg::Kill));
                return Err(err);
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
        // 1. Tell the JS script to reset its context
        // 2. Reset the puppet
        // 3. Launch the JS script again

        todo!()
    }

    fn identity(&self) -> base::Identity {
        todo!()
    }
}
