//! Provides a runtime, for JavaScript scripts, that integrates with the SCM runtime to allow full
//! scripting capabilities to be used in JavaScript code.

use crate::scripts::asm;

use super::{asm::Value, base, game};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use quick_js::{Context, JsValue};
use serde::{Deserialize, Serialize};
use std::{
    hash::{Hash, Hasher},
    thread::{self, JoinHandle},
};

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

/// Messages that can be sent to the control thread. Every message in `ReqMsg` should trigger the
/// control thread to send back one of the responses in `RespMsg`, or `None` if no response makes
/// sense.
pub enum ReqMsg {
    /// Execute an SCM instruction, determined by an opcode, with the given arguments. Should
    /// trigger a response containing the boolean return value of the instruction, which may or may
    /// not be relevant (depending on whether the instruction actually returns anything).
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

    /// Contains the boolean flag and the time at which the script should continue executing. Sent
    /// back after executing an instruction.
    InstrDone(bool, u32),
}

/// A bidirectional connection between a JS script and the control thread. This structure should be
/// held by the script.
#[derive(Clone)]
struct CtrlConn {
    /// Sender for sending request messages to the control module.
    sender: Sender<ReqMsg>,

    /// Receiver for receiving responses from the control module. This must be inside a `Mutex` so
    /// that this whole structure implements the `Sync` trait.
    receiver: Receiver<Option<RespMsg>>,
}

impl CtrlConn {
    /// Creates two channels for communication between the JS module and the control module.
    /// Returns the endpoint for this module as well as the one created for the control module.
    fn new() -> (CtrlConn, JsConn) {
        // Our channel to the control module must have a buffer size of zero so that it blocks when
        // we send messages. This way, we can't carry on executing JavaScript while inline SCM code
        // is being processed (which would be very bad).
        let (to_ctrl, from_js) = crossbeam_channel::bounded(0);
        let (to_js, from_ctrl) = crossbeam_channel::unbounded();

        (
            CtrlConn {
                sender: to_ctrl,
                receiver: from_ctrl,
            },
            JsConn::new(to_js, from_js),
        )
    }

    /// Sends the given message to the control thread, then waits for a response to return. A
    /// return value of `None` does not indicate failure; it simply means that there is no
    /// meaningful response.
    fn send(&self, msg: ReqMsg) -> Option<RespMsg> {
        self.sender
            .send(msg)
            .expect("Failed to send message to control thread");

        self.receiver
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("Unable to receive message from control thread (possible timeout?)")
    }
}

/// A single JavaScript CLEO script. These scripts run on a separate thread from the SCM scripts
/// used by the game itself, but SCM instruction calls are compiled to bytecode which is executed
/// on the SCM thread.
#[derive(Clone)]
struct Script {
    /// The name of the script, used to refer to it in the log.
    name: String,

    /// The script's connection to the control module.
    conn: CtrlConn,

    /// The source code of the script.
    code: String,

    /// Whether the script is currently running or not.
    running: bool,
}

impl Script {
    /// Start evaluating the script's code in another thread. This method will return immediately.
    fn launch(&mut self) -> JoinHandle<()> {
        self.running = true;

        // Clone this script so we don't have to recreate it if we need to run it again after it
        // exits.
        // hack: Cloning the whole script is expensive and probably not necessary.
        let script = self.clone();

        thread::spawn(move || {
            // We have to create the JS context inside the new thread because we can't pass it
            // between threads.
            let context = Context::builder()
                .console(JsConsole::new(script.name))
                .build()
                .expect("Failed to initialise JS context");

            context
                .add_callback("setGxtKeyValue", |key: String, value: String| {
                    crate::text::set_kv(&key, &value);
                    JsValue::Undefined
                })
                .expect("Failed to add GXT callback");

            let scm_conn = script.conn.clone();
            let scm_call = move |opcode: i32, args: Vec<JsValue>| -> Result<JsValue> {
                log::info!("SCM call");

                // This simple conversion is not anywhere near what we really need.
                let scm_args = args.iter().map(get_scm_value).collect::<Result<Vec<_>>>()?;
                let response = scm_conn.send(ReqMsg::ExecInstr(opcode as u16, scm_args));

                match response {
                    Some(RespMsg::Exit) => {
                        return Err(anyhow::format_err!("Script thread exiting"))
                    }
                    Some(RespMsg::InstrDone(flag, continue_time)) => {
                        let game_time = game::time();

                        if continue_time > game_time {
                            let wait_len = continue_time - game::time();

                            // Wait until we're scheduled to run again.
                            thread::sleep(std::time::Duration::from_millis(wait_len as u64));
                        }

                        Ok(JsValue::Bool(flag))
                    }
                    _ => Ok(JsValue::Undefined),
                }
            };

            context
                .add_callback("scmCall", scm_call)
                .expect("Failed to add scmCall function");

            if let Err(err) = context.eval(&script.code) {
                // Report the error so the control module can handle it. We can ignore the response
                // message, because we're going to exit straight away regardless.
                script.conn.send(ReqMsg::ReportErr(err.into()));
            }
        })
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

        // All JavaScript log messages are logged at the CLEO info level because an "error" at CLEO
        // level is much more serious than an "error" inside a JavaScript script and we don't want
        // to pollute the log file. We still show the level with a string, though.
        let level_name = match level {
            Trace => "trace",
            Debug => "debug",
            Log => "log",
            Info => "info",
            Warn => "warn",
            Error => "error",
        };

        // Convert all of the values to `String`s and join them with spaces. If we can't convert a
        // value using `into_string`, we just use the debug format instead.
        let message = values
            .into_iter()
            .map(|v| {
                // We have to clone here so we can use the original value again if we can't convert
                // using `into_string`.
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

/// A connection between this module and a JavaScript-based script that allows us to receive
/// requests and send back responses.
pub struct JsConn {
    sender: Sender<Option<RespMsg>>,
    receiver: Receiver<ReqMsg>,
}

impl JsConn {
    /// Create a new connection using a sender and receiver.
    pub fn new(sender: Sender<Option<RespMsg>>, receiver: Receiver<ReqMsg>) -> JsConn {
        JsConn { sender, receiver }
    }

    fn send(&self, msg: Option<RespMsg>) {
        self.sender
            .send(msg)
            .expect("Failed to send response message");
    }

    fn next(&mut self) -> Option<ReqMsg> {
        match self.receiver.try_recv() {
            Ok(msg) => Some(msg),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => panic!("JS connection disconnected"),
        }
    }
}

/// A proxy for a JavaScript-based script that behaves like a full script.
pub struct ScriptUnit {
    /// The script that this controller is in charge of. We need to communicate with it through
    /// channels because even though it might be owned by the controlling thread, it will actually
    /// execute on another thread.
    script: Script,

    /// The connection through which we communicate with the script's thread of execution.
    conn: JsConn,

    /// A skeleton script that we use to run the JIT-compiled instructions requested by the
    /// JavaScript thread.
    puppet: game::CleoScript,

    /// A handle with which we can move the execution of JavaScript code to the controlling thread.
    join_handle: Option<thread::JoinHandle<()>>,
}

impl ScriptUnit {
    pub fn load(path: std::path::PathBuf) -> Result<ScriptUnit> {
        let name = match path.file_name().and_then(|os| os.to_str()) {
            Some(s) => s.to_string(),
            None => "JS Script".to_string(),
        };

        let bytes = std::fs::read(path)?;
        ScriptUnit::from_bytes(name, &bytes)
    }

    pub fn from_bytes(name: String, bytes: &[u8]) -> Result<ScriptUnit> {
        // Create a connection to use between the script and control threads.
        let (conn, ext_conn) = CtrlConn::new();

        let code = std::str::from_utf8(bytes)?.to_string();
        let script = Script {
            name,
            conn,
            code,
            running: false,
        };

        Ok(Self::new(script, ext_conn))
    }

    /// Create a new script using the given communication structure.
    fn new(script: Script, conn: JsConn) -> ScriptUnit {
        let puppet = game::CleoScript::new(
            // No name required.
            String::new(),
            // A JS-based script should never have more than one instruction in it, so 1000
            // bytes is plenty of space.
            &mut &vec![0; 1000][..],
        )
        // Safety: `CleoScript::new` only fails when it can't read the bytes, but we know that
        // we've just created the vector, so it won't fail.
        .unwrap();

        ScriptUnit {
            script,
            conn,
            puppet,
            join_handle: None,
        }
    }
}

impl base::Script for ScriptUnit {
    fn exec_single(&mut self) -> Result<base::FocusWish> {
        let request = match self.conn.next() {
            Some(r) => r,

            // If there's no request to handle, just move on to the next script.
            None => return Ok(base::FocusWish::MoveOn),
        };

        use ReqMsg::*;
        let response = match request {
            ExecInstr(opcode, args) => {
                // Clear anything that could affect instruction behaviour and return the
                // instruction pointer to the beginning of the script.
                self.puppet.reset();
                self.puppet.set_state(base::State::Auto(true));

                self.puppet.bytecode_mut().clear();

                // Create a new instruction to compile and execute.
                let instr = asm::Instr::new(opcode, args);
                log::trace!("instruction: {}", instr);

                let byte_count = instr.write(&mut self.puppet.bytecode_mut())?;

                log::trace!("Wrote {} bytes of compiled code", byte_count);

                let byte_str = self.puppet.bytecode_mut()[..byte_count]
                    .iter()
                    .map(|b| format!("{:x}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                log::trace!("bytes: {}", byte_str);

                // Execute the instruction we just assembled.
                let focus_wish = self.puppet.exec_single()?;

                // Clear the bytecode we created so that the next instruction is not mixed with old
                // bytes.
                self.puppet.bytecode_mut().clear();

                // Send back the boolean flag of the script. This could be considered the return
                // value of the SCM instruction. We also send the time at which the script should
                // continue executing, because the waiting needs to be done on the script's thread.
                self.conn.send(Some(RespMsg::InstrDone(
                    self.puppet.bool_flag(),
                    self.puppet.wakeup_time(),
                )));

                return Ok(focus_wish);
            }
            GetVar(_) => todo!(),
            SetVar(_, _) => todo!(),
            ReportErr(err) => {
                self.conn.send(Some(RespMsg::Exit));
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
        self.conn.send(Some(RespMsg::Exit));

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

        self.script.running = false;
    }

    fn identity(&self) -> base::Identity {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.script.code.hash(&mut hasher);
        base::Identity::Js(hasher.finish())
    }

    fn set_state(&mut self, state: base::State) {
        if state.is_on() && !self.script.running {
            self.join_handle = Some(self.script.launch());
        }

        // todo: Handle other script states here.

        self.puppet.set_state(state);
    }

    fn state(&self) -> base::State {
        self.puppet.state()
    }

    fn name(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed(&self.script.name)
    }

    fn add_flag(&mut self, flag: base::Flag) {
        self.puppet.add_flag(flag);
    }

    fn flags(&self) -> &std::collections::BTreeSet<base::Flag> {
        self.puppet.flags()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    pub meta: Meta,
    pub extensions: Vec<Extension>,
    pub classes: Vec<Class>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    pub last_update: i64,
    pub version: String,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Extension {
    pub name: String,
    pub commands: Vec<Command>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub num_params: i64,
    pub short_desc: Option<String>,
    pub attrs: Option<Attrs>,
    #[serde(default)]
    pub input: Vec<IoValue>,
    pub class: Option<String>,
    pub member: Option<String>,
    #[serde(default)]
    pub output: Vec<IoValue>,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub versions: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attrs {
    pub is_static: Option<bool>,
    pub is_condition: Option<bool>,
    pub is_overload: Option<bool>,
    pub is_keyword: Option<bool>,
    pub is_variadic: Option<bool>,
    pub is_nop: Option<bool>,
    pub is_destructor: Option<bool>,
    pub is_constructor: Option<bool>,
    pub is_unsupported: Option<bool>,
    pub is_branch: Option<bool>,
    pub is_segment: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IoValue {
    pub name: String,
    pub r#type: String,
    pub source: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Class {
    pub name: String,
    pub constructable: bool,
    pub desc: String,
    pub extends: Option<String>,
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
