use super::{ctrl, scm::Value};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use quick_js::{Context, JsValue};
use std::{io, thread::JoinHandle};

fn get_scm_value(value: &JsValue) -> Result<Value> {
    Ok(match value {
        JsValue::Bool(value) => Value::Integer(if *value { 1 } else { 0 }),
        JsValue::String(value) => Value::String(value.to_string()),
        JsValue::Float(value) => Value::Real(*value as f32),
        JsValue::Int(value) => Value::Integer(*value as i64),

        _ => {
            return Err(anyhow::format_err!(
                "Cannot convert value {:?} to any SCM equivalent",
                value
            ));
        }
    })
}

/// A value that identifies a particular variable accessible from a script.
pub enum VarHandle {
    Local(usize),
    Global(isize),
}

/// Messages that can be sent to the control thread. Every message in `ReqMsg` should trigger
/// the control thread to send back one of the responses in `RespMsg`, or `None` if no response
/// makes sense.
pub enum ReqMsg {
    /// Execute an SCM instruction, determined by an opcode, with the given arguments.
    /// Should trigger a response containing the boolean return value of the instruction,
    /// which may or may not be relevant (depending on whether the instruction actually
    /// returns anything).
    ExecInstr(u16, Vec<Value>),

    /// Trigger a response containing the value of the specified variable.
    GetVar(VarHandle),

    /// Set the value of the specified variable.
    SetVar(VarHandle, Value),

    /// Report an error. This should generally be responded to with a kill message.
    ReportErr(anyhow::Error),

    /// Gives the join handle of a launched script's thread to the control module so that it may
    /// move execution to its own thread at any point.
    JoinHandle(JoinHandle<()>),
}

pub enum RespMsg {
    /// Contains the value of a variable that was requested.
    Var(Value),

    /// Tells the JS script to exit.
    Exit,

    /// Contains the boolean flag. Sent back after executing an instruction.
    BoolFlag(bool),
}

/// A bidirectional connection between a JS script and the control thread.
/// This structure should be held by the script.
#[derive(Clone)]
struct CtrlConn {
    /// Sender for sending request messages to the control module.
    sender: Sender<ReqMsg>,

    /// Receiver for receiving responses from the control module.
    /// This must be inside a `Mutex` so that this whole structure implements
    /// the `Sync` trait.
    receiver: Receiver<Option<RespMsg>>,
}

impl CtrlConn {
    /// Creates two channels for communication between the JS module and the control module.
    /// Returns the endpoint for this module as well as the one created for the control module.
    fn new() -> (CtrlConn, ctrl::JsConn) {
        // Our channel to the control module must have a buffer size of zero so that it
        // blocks when we send messages. This way, we can't carry on executing JavaScript
        // while inline SCM code is being processed (which would be very bad).
        let (to_ctrl, from_js) = crossbeam_channel::bounded(0);
        let (to_js, from_ctrl) = crossbeam_channel::unbounded();

        (
            CtrlConn {
                sender: to_ctrl,
                receiver: from_ctrl,
            },
            ctrl::JsConn::new(to_js, from_js),
        )
    }

    /// Sends the given message to the control thread, then waits for
    /// a response to return. A return value of `None` does not indicate
    /// failure; it simply means that there is no meaningful response.
    fn send(&self, msg: ReqMsg) -> Option<RespMsg> {
        self.sender
            .send(msg)
            .expect("Failed to send message to control thread");

        self.receiver
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("Unable to receive message from control thread (possible timeout?)")
    }
}

/// A single JavaScript CLEO script. These scripts run on a separate thread from the SCM scripts used
/// by the game itself, but SCM instruction calls are compiled to bytecode which is executed on the
/// SCM thread.
#[derive(Clone)]
struct Script {
    /// The name of the script, used to refer to it in the log.
    name: String,

    /// The script's connection to the control module.
    conn: CtrlConn,

    /// The source code of the script.
    code: String,
}

impl Script {
    /// Creates a new JavaScript-based script with the given name and code bytes.
    /// Also returns the communication structure to be given to the control module to
    /// allow it to manage the script.
    fn new(name: String, bytes: &mut impl io::Read) -> Result<(Script, ctrl::JsConn)> {
        let (conn, ext_conn) = CtrlConn::new();

        let mut code = String::new();
        bytes.read_to_string(&mut code)?;

        let script = Script { name, conn, code };

        Ok((script, ext_conn))
    }

    /// Start evaluating the script's code in another thread. This method will return immediately.
    fn launch(&mut self) {
        // Clone this script so we don't have to recreate it if we need to run it again after it exits.
        let script = self.clone();

        let join_handle = std::thread::spawn(move || {
            // We have to create the JS context inside the new thread because we
            // can't pass it between threads.
            let context = Context::builder()
                .console(JsConsole::new(script.name))
                .build()
                .expect("Failed to initialise JS context");

            let scm_conn = script.conn.clone();
            let scm_call = move |opcode: i32, args: Vec<JsValue>| -> Result<JsValue> {
                log::info!("SCM call");

                // This simple conversion is not anywhere near what we really need.
                let scm_args = args.iter().map(get_scm_value).collect::<Result<Vec<_>>>()?;
                let response = scm_conn.send(ReqMsg::ExecInstr(opcode as u16, scm_args));

                if let Some(RespMsg::Exit) = response {
                    return Err(anyhow::format_err!("Script thread exiting"));
                }

                Ok(JsValue::Undefined)
            };

            context
                .add_callback("scmCall", scm_call)
                .expect("Failed to add scmCall function");

            if let Err(err) = context.eval(&script.code) {
                // Report the error so the control module can handle it. We can ignore the
                // response message, because we're going to exit straight away regardless.
                script.conn.send(ReqMsg::ReportErr(err.into()));
            }
        });

        let _ = self.conn.send(ReqMsg::JoinHandle(join_handle));
    }
}

/// Backend for the `console` module that is available within JavaScript code.
struct JsConsole {
    script_name: String,
}

impl JsConsole {
    fn new(script_name: String) -> JsConsole {
        JsConsole { script_name }
    }
}

impl quick_js::console::ConsoleBackend for JsConsole {
    fn log(&self, level: quick_js::console::Level, values: Vec<JsValue>) {
        use quick_js::console::Level::*;

        // All JavaScript log messages are logged at the CLEO info level because an "error"
        // at CLEO level is much more serious than an "error" inside a JavaScript script and
        // we don't want to pollute the log file. We still show the level with a string, though.
        let level_name = match level {
            Trace => "trace",
            Debug => "debug",
            Log => "log",
            Info => "info",
            Warn => "warn",
            Error => "error",
        };

        // Convert all of the values to `String`s and join them with spaces.
        // If we can't convert a value using `into_string`, we just use the
        // debug format instead.
        let message = values
            .into_iter()
            .map(|v| {
                // We have to clone here so we can use the original value again if
                // we can't convert using `into_string`.
                v.clone()
                    .into_string()
                    .unwrap_or_else(|| format!("{:?}", v))
            })
            .collect::<Vec<_>>()
            .join(" ");

        log::info!(
            "JS script {} ({}): {}",
            self.script_name,
            level_name,
            message
        );
    }
}

pub fn init() {}

/*
    Script mode should be given using a comment (preferably at the top of the file).

    CSI:
        // cleo:mode = invoked
        Look for "//cleo:mode=invoked" after removing all spaces
    CSA:
        // cleo:mode = running
        Look for "//cleo:mode=running" after removing all spaces
*/
