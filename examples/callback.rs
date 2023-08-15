use rusty_jsc::{JSContext, JSValue, JSException};
use rusty_jsc_macros::callback;

#[callback]
fn foo(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    args: &[JSValue],
) -> Result<JSValue, JSException> {
    println!(
        "hello from Rust land! len: {}, value[0]: {}",
        args.len(),
        args[0].to_string(&ctx).unwrap()
    );
    Ok(JSValue::string(&ctx, "Returning a string to JS!"))
}

#[callback]
fn foo2<A>(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    _args: &[JSValue],
) -> Result<JSValue, JSException>
where
    A: Clone,
{
    println!("hello from Rust land!");
    Ok(JSValue::string(&ctx, "Hey"))
}

fn main() {
    let context = JSContext::default();
    let callback = JSValue::callback(&context, Some(foo));
    let global = context.get_global_object();
    global.set_property(&context, "foo", callback).unwrap();
    let foo = global
        .get_property(&context, "foo")
        .to_object(&context)
        .unwrap();
    let result = foo.call(
        &context,
        None,
        &[
            JSValue::number(&context, 5f64),
            JSValue::number(&context, 6f64),
        ],
    );
    println!(
        "direct call: {}",
        result.unwrap().to_string(&context).unwrap()
    );
    match context.evaluate_script("foo(1, 2, 3)", 1) {
        Ok(value) => {
            println!("{}", value.to_string(&context).unwrap());
        }
        Err(e) => {
            println!("Uncaught: {}", e.to_string())
        }
    }
}
