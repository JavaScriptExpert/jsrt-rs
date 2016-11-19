#![allow(non_upper_case_globals)]
#![allow(improper_ctypes)]

#[macro_use]
extern crate bitflags;
extern crate libc;

pub use debug::*;
pub use common::*;
pub use core::*;

mod common;
mod core;
mod debug;

#[cfg(test)]
mod tests {
    use std::ptr;
    use std::str;
    use super::*;

    macro_rules! js {
        ($e: expr) => {
            let result = $e;
            if result != JsErrorCode::NoError {
                panic!("JavaScript failed error: {:?}", result);
            }
        }
    }

    #[test]
    fn it_works() {
        unsafe {
            let mut runtime = JsRuntimeHandle::new();
            js!(JsCreateRuntime(JsRuntimeAttributeNone, None, &mut runtime));

            // Create an execution context.
            let mut context = JsContextRef::new();
            js!(JsCreateContext(runtime, &mut context));

            // Now set the current execution context.
            js!(JsSetCurrentContext(context));

            let mut script = String::from("5 + 5");
            let vector = script.as_mut_vec();

            let mut script_buffer = JsValueRef::new();
            js!(JsCreateExternalArrayBuffer(vector.as_mut_ptr() as *mut _,
                                            vector.len() as usize as _,
                                            None,
                                            ptr::null_mut(),
                                            &mut script_buffer));

            let name = "test";
            let mut name_value = JsValueRef::new();
            js!(JsCreateStringUtf8(name.as_ptr(), name.len(), &mut name_value));

            // Run the script.
            let mut result = JsValueRef::new();
            let mut source_context = 1;
            js!(JsRun(script_buffer,
                      &mut source_context,
                      name_value,
                      JsParseScriptAttributeNone,
                      &mut result));

            // Convert your script result to String in JavaScript; redundant if your
            // script returns a String
            let mut result_as_string = JsValueRef::new();
            js!(JsConvertValueToString(result, &mut result_as_string));

            // Project script result back to C++.
            let mut size = 0;
            let mut buffer = vec![0; 100];
            js!(JsCopyStringUtf8(result_as_string,
                                 buffer.as_mut_ptr(),
                                 buffer.len(),
                                 &mut size));
            buffer.truncate(size);

            println!("Output: {}", str::from_utf8_unchecked(&buffer));

            // Dispose runtime
            js!(JsSetCurrentContext(JsValueRef::new()));
            js!(JsDisposeRuntime(runtime));
        }
    }
}
