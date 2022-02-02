use anyhow::Result;
use boa::{
    object::{FunctionBuilder, ObjectInitializer},
    property::{Attribute, PropertyDescriptor},
    Context, JsResult, JsValue,
};

struct Runtime {
    context: Context,
}

impl Runtime {
    /// Creates the runtime in which all the JS scripts are executed.
    fn new() -> Result<Runtime> {
        log::info!("Creating JS runtime");
        let cleo_script = std::str::from_utf8(include_bytes!("cleo.js"))?;

        let mut context = Context::new();

        let function = FunctionBuilder::native(&mut context, Self::js_print).build();
        context.register_global_property("print", function, Attribute::READONLY);

        if let Err(err) = context.eval(cleo_script) {
            return Err(anyhow::format_err!(
                "Error while evaluating cleo.js: {}",
                match err.to_string(&mut context) {
                    Ok(s) => s.to_string(),
                    Err(_) => "Unable to convert error message to string!".to_string(),
                }
            ));
        }

        Ok(Runtime { context })
    }

    fn js_print(func: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if args.is_empty() {
            return context.throw_error("No values passed to print!");
        }

        log::info!("Script: {}", args[0].to_string(context)?);

        Ok(JsValue::undefined())
    }
}

pub fn init() {
    let _ = Runtime::new().expect("Unable to initialise JavaScript runtime");
}

/*
    JavaScript interface for internal opcode stuff:
      - Provide JS functions in cleo.js with normal calling interfaces
      - Pass arguments and opcode to Rust function
      - Arguments are converted to appropriate C values and placed into the game's argument memory
      - The instruction implementation should be run as normal
*/
