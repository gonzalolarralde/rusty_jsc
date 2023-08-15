use anyhow::{Context, Result};
use rusty_jsc::{JSContext, JSValue};
use rusty_jsc_macros::callback;
use std::fs;
use std::path::PathBuf;

pub fn run(input: PathBuf) -> Result<()> {
    let script = fs::read_to_string(&input)
        .with_context(|| format!("Failed to load module `{}`", input.display()))?;
    let context = JSContext::new();
    setup_prelude(&context);
    let result = context.evaluate_script(&script, 1);
    if let Err(ex) = result {
        anyhow::bail!("Uncaught {}", ex.to_string(&context).unwrap());
    }
    Ok(())
}

fn setup_prelude(context: &JSContext) {
    let require_fn = JSValue::callback(&context, Some(require));
    // require()
    let global = context.get_global_object();
    let _ = global.set_property(&context, "require".to_string(), require_fn);
    // foo()
    let callback = JSValue::callback(&context, Some(foo));
    let _ = global.set_property(&context, "foo".to_string(), callback);
}

#[callback]
fn require(_context: JSContext, _function: JSObject, _this: JSObject, _args: &[JSValue]) -> Result<JSValue, JSValue> {
    println!("warning: `require` is not implemented.");
    Ok(JSValue::undefined(&_context))
}

#[callback]
fn foo(_context: JSContext, _function: JSObject, _this: JSObject, _args: &[JSValue]) -> Result<JSValue, JSValue> {
    println!("hello from Rust land!");
    Ok(JSValue::undefined(&_context))
}
