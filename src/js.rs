use anyhow::Result;
use boa::{
    builtins::function::NativeFunction,
    exec::Executable,
    object::{FunctionBuilder, ObjectInitializer},
    property::{Attribute, PropertyDescriptor},
    syntax::ast::{node::StatementList, Node},
    Context, JsResult, JsString, JsValue,
};
use byteorder::{ByteOrder, WriteBytesExt};

use crate::check::Value;

// todo: Create fake scripts that use JS behind the scenes but that the game can interact with as normal.
// Three stages to running an instruction:
//  1: Setting up the fake script to allow the game to use it.
//  2: Processing a single instruction.
//  3: Taking anything the game has done with the fake script and using it to influence the JS script's state.
// These steps should be repeated for every instruction within a block.

struct Runtime {
    context: Context,
}

impl Runtime {
    /// Creates the runtime in which all the JS scripts are executed.
    fn new() -> Result<Runtime> {
        let mut runtime = Runtime {
            context: Context::new(),
        };

        runtime.context.set_strict_mode_global();

        runtime.add_func("print", Self::js_print);
        runtime.add_func("addGxtString", Self::add_gxt);

        Ok(runtime)
    }

    fn add_func(&mut self, name: impl AsRef<str>, func: NativeFunction) {
        let function = FunctionBuilder::native(&mut self.context, func).build();
        self.context
            .register_global_property(name.as_ref(), function, Attribute::READONLY);
    }

    fn js_print(_func: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if args.is_empty() {
            return context.throw_error("No values passed to print!");
        }

        log::info!("Script: {}", args[0].to_string(context)?);

        Ok(JsValue::Undefined)
    }

    fn add_gxt(_func: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if args.len() != 2 {
            return context.throw_error("addGxtString expects two arguments.");
        }

        crate::text::set_kv(
            args[0].to_string(context)?.as_str(),
            args[1].to_string(context)?.as_str(),
        );

        Ok(JsValue::Undefined)
    }

    fn get_scm_value(value: &JsValue) -> Option<Value> {
        Some(match value {
            JsValue::Boolean(value) => Value::Integer(if *value { 1 } else { 0 }),
            JsValue::String(value) => Value::String(value.to_string()),
            JsValue::Rational(value) => Value::Real(*value as f32),
            JsValue::Integer(value) => Value::Integer(*value as i64),

            JsValue::Null
            | JsValue::Undefined
            | JsValue::BigInt(_)
            | JsValue::Object(_)
            | JsValue::Symbol(_) => {
                log::error!(
                    "Cannot convert value of type {} to SCM value!",
                    value.type_of().to_string()
                );

                return None;
            }
        })
    }
}

struct Script {
    name: String,

    /// A fake script that we use when interacting with game code.
    puppet: crate::scripts::CleoScript,
    runtime: Runtime,

    statements: StatementList,
    next_index: usize,

    execution_ended: bool,

    /// Flag (set when script instructions run) that decides whether or not the next
    /// instruction should also run in this execution block.
    /// Stays as `None` until an instruction runs, after which an appropriate value
    /// will be set.
    continue_flag: Option<bool>,
}

impl Script {
    fn new(name: String, src_bytes: &[u8]) -> Result<Script> {
        // todo: Check for script mode comment.

        // Create a script containing 1K of zeros (which are just NOP instructions).
        let mut puppet = crate::scripts::CleoScript::new(vec![0u8; 1024], name.clone());
        puppet.set_active(true);

        // Parse the JavaScript bytes.
        let statements = boa::syntax::Parser::new(src_bytes, true)
            .parse_all()
            .map_err(|e| anyhow::format_err!("Syntax error: {}", e.to_string()))?;

        Ok(Script {
            name,
            puppet,
            runtime: Runtime::new()?,
            statements,
            next_index: 0,
            execution_ended: false,
            continue_flag: None,
        })
    }

    fn exec_instr(&mut self, js_call_args: &[JsValue]) -> Result<()> {
        let mut args = js_call_args.iter();

        let opcode = args
            .next()
            .unwrap()
            .as_number()
            .expect("Opcode must be a number")
            .round() as u16;

        let params: Vec<Value> = match args.map(Runtime::get_scm_value).collect::<Option<Vec<_>>>()
        {
            Some(p) => p,
            None => {
                return Err(anyhow::format_err!(
                    "Unable to convert JS argument data to SCM format for instruction call"
                ));
            }
        };

        // fixme: Much more data than just the IP should be reset for SCM calls.
        self.puppet.reset_ip();

        let mut instr_data = &mut self.puppet.bytes[..];
        instr_data.write_u16::<byteorder::LittleEndian>(opcode)?;

        // Write the parameter data to the puppet script's instruction space.
        for param in params {
            match param {
                Value::Integer(val) => {
                    // i32 type code.
                    instr_data.write_u8(0x01)?;
                    instr_data.write_i32::<byteorder::LittleEndian>(val as i32)?;
                }
                Value::Real(_) => todo!(),
                Value::String(string) => {
                    // Variable-length string type code.
                    instr_data.write_u8(0x0e)?;
                    instr_data.write_u8(string.len() as u8)?;

                    for c in string.chars() {
                        instr_data.write_u8(c as u8)?;
                    }
                }
                Value::Model(_) => todo!(),
                // Value::Pointer(_) => todo!(),
                Value::VarArgs(_) => todo!(),
                Value::Buffer(_) => todo!(),
                // Value::Variable(_) => todo!(),
                // Value::Array(_) => todo!(),
                _ => todo!(),
            }
        }

        let should_continue = !self.puppet.update_once();
        self.continue_flag = Some(should_continue);

        Ok(())
    }

    /// Executes one instruction. This includes executing any JavaScript statements leading
    /// up to the instruction function call.
    fn run_single(&mut self) -> Result<()> {
        while self.continue_flag.is_none() && self.next_index < self.statements.items().len() {
            match &self.statements.items()[self.next_index] {
                Node::Call(call) if matches!(call.expr(), Node::Identifier(ident) if ident.as_ref() == "scmCall") =>
                {
                    log::info!("Found SCM call!");

                    let converted_args = call
                        .args()
                        .iter()
                        .map(|node| node.run(&mut self.runtime.context))
                        .collect::<Result<Vec<_>, _>>();

                    let args = match converted_args {
                        Ok(args) => args,
                        Err(e) => {
                            let err_str = match e.to_string(&mut self.runtime.context) {
                                Ok(s) => s.to_string(),
                                Err(_) => "Unable to convert error string to String".to_string(),
                            };

                            return Err(anyhow::format_err!(
                                "Error while converting SCM call arguments: {}",
                                err_str
                            ));
                        }
                    };

                    self.exec_instr(&args)?;

                    self.next_index += 1;
                    continue;
                }
                _ => (),
            };

            let run_result =
                self.statements.items()[self.next_index].run(&mut self.runtime.context);

            // Execute the next statement.
            if let Err(err) = run_result {
                return Err(anyhow::format_err!(
                    "Runtime error in script '{}': {:?}",
                    self.name,
                    err.to_string(&mut self.runtime.context)
                ));
            }

            self.next_index += 1;
        }

        Ok(())
    }

    /// Run a block of instructions.
    fn run_block(&mut self) -> Result<()> {
        if !self.puppet.wants_update() {
            return Ok(());
        }

        if self.execution_ended {
            log::info!("Execution already ended in {}.", self.name);
            return Ok(());
        }

        loop {
            self.run_single()?;

            if let Some(false) = self.continue_flag.take() {
                break;
            }

            if self.next_index >= self.statements.items().len() {
                log::info!("Script {} finished executing.", self.name);

                // No more statements left, so we're done executing.
                self.execution_ended = true;
                break;
            }
        }

        Ok(())
    }
}

pub struct ScriptManager {
    scripts: Vec<Script>,
}

impl ScriptManager {
    pub fn load() -> Result<ScriptManager> {
        let cleo_script = include_bytes!("cleo.js");

        Ok(ScriptManager {
            scripts: vec![Script::new("cleo.js".into(), cleo_script)?],
        })
    }

    pub fn update_all(&mut self) -> Result<()> {
        for script in self.scripts.iter_mut() {
            script.run_block()?;
        }

        Ok(())
    }
}

pub fn init() {

    // let _ = Runtime::new().expect("Unable to initialise JavaScript runtime");
}

/*
    Script mode should be given using a comment (preferably at the top of the file).

    CSI:
        // cleo:mode = invoked
        Look for "//cleo:mode=invoked" after removing all spaces
    CSA:
        // cleo:mode = running
        Look for "//cleo:mode=running" after removing all spaces
*/
