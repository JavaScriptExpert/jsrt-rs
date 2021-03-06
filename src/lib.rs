#[macro_use]
extern crate error_chain;
extern crate jsrt_sys;
extern crate libc;

pub use context::Context;
pub use runtime::Runtime;
pub use script::Script;
pub use property::PropertyId;

#[macro_use]
mod macros;
mod property;
mod script;
mod runtime;
mod util;
pub mod context;
pub mod error;
pub mod value;

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_env() -> (Runtime, Context) {
        let runtime = Runtime::new().unwrap();
        let context = Context::new(&runtime).unwrap();
        (runtime, context)
    }

    #[test]
    fn basic_runtime() {
        let (_runtime, context) = setup_env();
        let guard = context.make_current().unwrap();

        // TODO: Wrap input in parantheses?
        let result = Script::eval(&guard, "(5 + 5)").unwrap();
        assert_eq!(result.to_integer(&guard), 10);
    }

    #[test]
    fn basic_exception() {
        let (_runtime, context) = setup_env();
        let guard = context.make_current().unwrap();

        let error = Script::eval(&guard, "throw 5;");
        let result = Script::eval(&guard, "(5 + 5)").unwrap();

        assert_eq!(result.to_integer(&guard), 10);
        assert!(error.is_err());
    }

    #[test]
    fn global_context() {
        let (_runtime, context) = setup_env();
        let guard = context.make_current().unwrap();

        let global = guard.global().unwrap();
        let dirname = PropertyId::from_str(&guard, "__dirname");

        global.set(&guard, &dirname, &value::String::from_str(&guard, "FooBar"));
        global.set_index(&guard, 2, &value::Number::new(&guard, 1337));

        let result1 = Script::eval(&guard, "__dirname").unwrap();
        let result2 = Script::eval(&guard, "this[2]").unwrap();

        assert_eq!(result1.to_string(&guard), "FooBar");
        assert_eq!(result2.to_integer(&guard), 1337);
    }

    #[test]
    fn function_call() {
        let (_runtime, context) = setup_env();
        let guard = context.make_current().unwrap();
        let captured_variable = 5.0;

        let function = value::Function::new(&guard, Box::new(move |guard, info| {
            // Ensure the defaults are sensible
            assert!(info.this.is_null());
            assert!(info.is_construct_call == false);
            assert_eq!(info.arguments.len(), 2);
            assert_eq!(captured_variable, 5.0);

            let result = info.arguments[0].to_double(guard) +
                         info.arguments[1].to_double(guard) +
                         captured_variable;
            Ok(value::Number::from_double(guard, result).into())
        }));

        let result = function.call(&guard, &value::null(&guard), &[
            value::Number::new(&guard, 5).into(),
            value::Number::from_double(&guard, 10.5).into()
        ]).unwrap();

        assert_eq!(result.to_integer(&guard), 20);
        assert_eq!(result.to_double(&guard), 20.5);
    }

    #[test]
    fn external_data_drop() {
        static mut called: bool = false;
        {
            struct Foo(i32);
            impl Drop for Foo {
                fn drop(&mut self) {
                    assert_eq!(self.0, 10);
                    unsafe { called = true };
                }
            }

            let (_runtime, context) = setup_env();
            let guard = context.make_current().unwrap();
            let _external = value::Object::with_external(&guard, Box::new(Foo(10)));
        }
        assert!(unsafe { called });
    }
}
