use super::js;
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
}
