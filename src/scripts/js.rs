use std::pin::Pin;

use super::{run::CleoScript, scm::Value};
use anyhow::Result;
use boa::{
    builtins::function::NativeFunction, exec::Executable, object::FunctionBuilder,
    property::Attribute, syntax::ast::node::StatementList, Context, JsResult, JsString, JsValue,
};
use byteorder::WriteBytesExt;

/*

    Current system limitations:
        - Just about everything
        - `scmCall` is only useable from top-level code because it's implemented as a special execution case and not a real function
        - Loading and updating JS scripts requires unsafe code in the `scripts` module
        - Internals from `scripts` and `check` have been exposed simply for use in the `js` module
            todo: Adapt (or rewrite) the current `scripts` module to allow different types of script that all implement a trait

*/

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

        let out_str = args
            .iter()
            .map(|a| match a.to_string(context) {
                Ok(s) => s.to_string(),
                Err(e) => format!("<Unable to convert to string: err = {:?}", e),
            })
            .collect::<Vec<_>>()
            .join(" ");

        log::info!("Script: {}", out_str);

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

struct ExecData {
    /// A fake script that we use when interacting with game code.
    puppet: CleoScript,

    /// Flag (set when script instructions run) that decides whether or not the next
    /// instruction should also run in this execution block.
    /// Stays as `None` until an instruction runs, after which an appropriate value
    /// will be set.
    continue_flag: Option<bool>,
}

impl ExecData {
    fn make_current(&mut self) {
        *Self::current() = Some(self);
    }

    fn remove_as_current(&mut self) {
        if let Some(prev_cur) = Self::current().take() {
            if prev_cur != self {
                panic!("Incorrect ExecData set as current");
            }
        } else {
            panic!("`remove_as_current` called with no current ExecData set");
        }
    }

    fn current() -> &'static mut Option<*mut ExecData> {
        static mut CURRENT: Option<*mut ExecData> = None;

        unsafe { &mut CURRENT }
    }
}

struct Script {
    name: String,

    exec_data: Pin<Box<ExecData>>,

    runtime: Runtime,

    statements: StatementList,
    next_index: usize,

    execution_ended: bool,
}

impl Script {
    fn new(name: String, src_bytes: &[u8]) -> Result<Script> {
        // todo: Check for script mode comment.

        // Create a script containing 1K of zeros (which are just NOP instructions).
        let mut puppet = CleoScript::new(vec![0u8; 1024], name.clone());
        puppet.set_active(true);

        let exec_data = Box::pin(ExecData {
            puppet,
            continue_flag: None,
        });

        // Parse the JavaScript bytes.
        let statements = boa::syntax::Parser::new(src_bytes, true)
            .parse_all()
            .map_err(|e| anyhow::format_err!("Syntax error: {}", e.to_string()))?;

        let mut runtime = Runtime::new()?;
        runtime.add_func("scmCall", Self::exec_instr_js);

        Ok(Script {
            name,
            exec_data,
            runtime,
            statements,
            next_index: 0,
            execution_ended: false,
        })
    }

    fn exec_instr_js(
        _func: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let exec_data = unsafe { &mut *ExecData::current().unwrap() };
        Self::exec_instr(exec_data, args).map_err(|e| JsString::new(e.to_string()))?;
        Ok(JsValue::Undefined)
    }

    fn exec_instr(exec_data: &mut ExecData, js_call_args: &[JsValue]) -> Result<()> {
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
        exec_data.puppet.reset_ip();

        let mut instr_data = &mut exec_data.puppet.bytes[..];
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

        let should_continue = !exec_data.puppet.update_once();
        exec_data.continue_flag = Some(should_continue);

        Ok(())
    }

    /// Executes one instruction. This includes executing any JavaScript statements leading
    /// up to the instruction function call.
    fn run_single(&mut self) -> Result<()> {
        self.exec_data.make_current();

        let statements = self.statements.items();

        while self.exec_data.continue_flag.is_none() && self.next_index < statements.len() {
            let run_result = statements[self.next_index].run(&mut self.runtime.context);

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

        self.exec_data.remove_as_current();

        Ok(())
    }

    /// Run a block of instructions.
    fn run_block(&mut self) -> Result<()> {
        if !self.exec_data.puppet.wants_update() {
            return Ok(());
        }

        if self.execution_ended {
            log::info!("Execution already ended in {}.", self.name);
            return Ok(());
        }

        loop {
            self.run_single()?;

            if let Some(false) = self.exec_data.continue_flag.take() {
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
