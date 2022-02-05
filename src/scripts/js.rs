use super::scm::Value;
use anyhow::Result;
use quick_js::{Context, JsValue};

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
enum VarHandle {
    Local(usize),
    Global(isize),
}

// fixme: `ReqMsg` and `RespMsg` should be defined in the control module, not here.

/// Messages that can be sent to the SCM thread. Every message in `ReqMsg` should trigger
/// the SCM thread to send back one of the responses in `RespMsg`, or `None` if no response
/// makes sense.
enum ReqMsg {
    /// Execute an SCM instruction, determined by an opcode, with the given arguments.
    /// Should trigger a response containing the Boolean return value of the instruction,
    /// which may or may not be relevant (depending on whether the instruction actually
    /// returns anything).
    ExecBytecode(u16, Vec<Value>),

    /// Trigger a response containing the value of the specified variable.
    GetVar(VarHandle),

    /// Set the value of the specified variable.
    SetVar(VarHandle, Value),
}

enum RespMsg {
    /// Contains the value of a variable that was requested.
    Var(Value),
}

/// A single JavaScript CLEO script. These scripts run on a separate thread from the SCM scripts used
/// by the game itself, but SCM instruction calls are compiled to bytecode which is executed on the
/// SCM thread.
struct Script {
    /// The name of the script, used to refer to it in the log.
    name: String,

    /// The runtime in which this script's JavaScript code runs.
    context: Context,
}

impl Script {}

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
