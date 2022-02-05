use std::pin::Pin;

use super::{run::CleoScript, scm::Value};
use anyhow::Result;
use boa::{
    builtins::function::NativeFunction, exec::Executable, object::FunctionBuilder,
    property::Attribute, syntax::ast::node::StatementList, Context, JsResult, JsString, JsValue,
};
use byteorder::WriteBytesExt;

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
        runtime.add_func("scmVarVal", Self::scm_var_oper);

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
                Err(e) => format!("<unable to convert to string: err = {:?}>", e),
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

    fn scm_var_oper(_func: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if args.len() != 2 && args.len() != 3 {
            return context.throw_error("Incorrect arg count.");
        }

        let index = match &args[0] {
            JsValue::Integer(val) => *val,
            other => {
                return context.throw_error(format!(
                    "Invalid variable index {:?} (expected integer)",
                    other
                ))
            }
        };

        let is_global = match &args[1] {
            JsValue::Boolean(val) => *val,
            other => {
                return context
                    .throw_error(format!("Expected boolean, but found {:?} instead", other))
            }
        };

        let exec_data = unsafe { &mut *ExecData::current().unwrap() };

        if args.len() == 3 {
            // 3 args, so we're setting the value.
            let _value =
                Self::get_scm_value(&args[2], context).map_err(|e| JsString::new(e.to_string()))?;

            // if is_global {
            //     exec_data.puppet.set_global_var(index, value);
            // } else {
            //     exec_data.puppet.set_local_var(index, value);
            // }
        } else {
            // 2 args, so we're getting the value.
            if is_global {
                todo!();
            } else {
                let value = exec_data
                    .puppet
                    .get_local_var(index as usize)
                    .ok_or_else(|| {
                        JsString::new(format!("No value found for local variable {}", index))
                    })?;

                // fixme: We need a proper solution for returning values over `i32::MAX`.
                if value > i32::MAX as u32 {
                    return Err(JsValue::String(JsString::new(format!(
                        "Value {} is too big to return as a JS integer",
                        value
                    ))));
                }

                return Ok(JsValue::Integer(value as i32));
            }
        }

        Ok(JsValue::Undefined)
    }

    fn get_scm_value(value: &JsValue, context: &mut Context) -> Result<Value> {
        Ok(match value {
            JsValue::Boolean(value) => Value::Integer(if *value { 1 } else { 0 }),
            JsValue::String(value) => Value::String(value.to_string()),
            JsValue::Rational(value) => Value::Real(*value as f32),
            JsValue::Integer(value) => Value::Integer(*value as i64),
            JsValue::Object(obj)
                if obj
                    .has_property("canBeScm", context)
                    .map_err(|e| js_val_to_err(e, context))? =>
            {
                Value::Variable(super::scm::Variable::new_local(
                    obj.get("index", context)
                        .map_err(|e| js_val_to_err(e, context))?
                        .to_index(context)
                        .map_err(|e| js_val_to_err(e, context))? as i64,
                ))
            }

            JsValue::Null
            | JsValue::Undefined
            | JsValue::BigInt(_)
            | JsValue::Object(_)
            | JsValue::Symbol(_) => {
                return Err(anyhow::format_err!(
                    "Cannot convert value of type {} to SCM value!",
                    value.type_of().to_string()
                ));
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
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let exec_data = unsafe { &mut *ExecData::current().unwrap() };
        Self::exec_instr(exec_data, args, context).map_err(|e| JsString::new(e.to_string()))?;
        Ok(JsValue::Undefined)
    }

    fn exec_instr(
        exec_data: &mut ExecData,
        js_call_args: &[JsValue],
        context: &mut Context,
    ) -> Result<()> {
        let mut args = js_call_args.iter();

        let opcode = args
            .next()
            .unwrap()
            .as_number()
            .expect("Opcode must be a number")
            .round() as u16;

        let params: Vec<Value> = args
            .map(|arg| Runtime::get_scm_value(arg, context))
            .collect::<Result<Vec<_>>>()?;

        // fixme: Much more data than just the IP should be reset for SCM calls.
        exec_data.puppet.reset_ip();

        let mut instr_data = &mut exec_data.puppet.bytes[..];
        instr_data.write_u16::<byteorder::LittleEndian>(opcode)?;

        // Write the parameter data to the puppet script's instruction space.
        for param in params {
            param.write(&mut instr_data)?;
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
                return Err(
                    js_val_to_err(err, &mut self.runtime.context).context(self.name.clone())
                );
            }

            self.next_index += 1;
        }

        self.exec_data.remove_as_current();

        Ok(())
    }

    /// Run a block of instructions.
    fn run_block(&mut self) -> Result<()> {
        if !self.exec_data.puppet.wants_update() || self.execution_ended {
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

fn js_val_to_err(value: JsValue, context: &mut Context) -> anyhow::Error {
    value
        .to_string(context)
        .map(|v| anyhow::format_err!("{}", v))
        .unwrap_or(anyhow::format_err!("Unable to convert error to string"))
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
