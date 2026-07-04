#![deny(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! C-compatible FFI layer for embedding the Motarjim compiler in other
//! languages (Node.js, Python, Ruby, etc.).
//!
//! # Safety
//!
//! All public functions in this module are `extern "C"` and operate on
//! raw pointers. Callers must:
//!
//! * Pass a valid pointer returned by [`motarjim_compiler_new`] to every
//!   other function.
//! * Free the compiler with [`motarjim_compiler_free`] when done.
//! * Free returned strings with [`motarjim_free_string`].
//! * Not use a compiler after it has been freed.

use std::ffi::{CStr, CString};
use std::sync::Arc;

use motarjim_config::OutputFormat;
use motarjim_core::{CompileOptions, Compiler};
use motarjim_fs::RealFileSystem;

/// Opaque handle to a Motarjim compiler instance.
///
/// Wraps [`motarjim_core::Compiler`] with default configuration and the
/// real filesystem.
pub struct FfiCompiler {
    /// Wrapped Motarjim compiler instance.
    inner: Compiler,
}

/// Create a new compiler instance.
///
/// Returns a raw pointer that must be freed with [`motarjim_compiler_free`].
/// Returns a null pointer if allocation fails.
#[allow(clippy::unwrap_used)]
#[no_mangle]
pub extern "C" fn motarjim_compiler_new() -> *mut FfiCompiler {
    let config = motarjim_config::Config::new();
    let fs: Arc<dyn motarjim_fs::FileSystem> = Arc::new(RealFileSystem::new());
    let inner = Compiler::new(config, fs);
    let compiler = Box::new(FfiCompiler { inner });
    Box::into_raw(compiler)
}

/// Free a compiler instance created by [`motarjim_compiler_new`].
///
/// # Safety
///
/// `ptr` must be a valid pointer returned by [`motarjim_compiler_new`] or
/// null (which is a no-op). After calling this function the pointer is
/// dangling and must not be used again.
#[no_mangle]
pub unsafe extern "C" fn motarjim_compiler_free(ptr: *mut FfiCompiler) {
    if ptr.is_null() {
        return;
    }
    drop(Box::from_raw(ptr));
}

/// Compile HTML/CSS input and return a JSON string with the result.
///
/// On success the JSON has the shape `{"ok": true, "output": "…"}`.
/// On failure the JSON has the shape `{"ok": false, "error": "…"}`.
///
/// # Safety
///
/// * `compiler` must be a valid, non-null pointer from
///   [`motarjim_compiler_new`].
/// * `input` must be a valid null-terminated C string.
/// * `platform` must be a valid null-terminated C string
///   (`"dart"`, `"kotlin"`, or `"swift"`).
///
/// The returned string must be freed with [`motarjim_free_string`].
#[allow(clippy::unwrap_used)]
#[no_mangle]
pub unsafe extern "C" fn motarjim_compile(
    compiler: *mut FfiCompiler,
    input: *const std::ffi::c_char,
    platform: *const std::ffi::c_char,
) -> *mut std::ffi::c_char {
    let result = try_compile(compiler, input, platform);
    match result {
        Ok(json) => CString::new(json).unwrap_or_default().into_raw(),
        Err(json) => CString::new(json).unwrap_or_default().into_raw(),
    }
}

/// Attempt to compile and return a JSON `Result`.
///
/// # Errors
///
/// Returns a JSON error string on any failure (null pointer, invalid
/// input, unsupported platform, compilation error).
#[allow(clippy::unwrap_used)]
unsafe fn try_compile(
    compiler: *mut FfiCompiler,
    input: *const std::ffi::c_char,
    platform: *const std::ffi::c_char,
) -> Result<String, String> {
    if compiler.is_null() {
        return Err(r#"{"ok":false,"error":"null compiler pointer"}"#.to_string());
    }
    if input.is_null() {
        return Err(r#"{"ok":false,"error":"null input string"}"#.to_string());
    }
    if platform.is_null() {
        return Err(r#"{"ok":false,"error":"null platform string"}"#.to_string());
    }

    let input_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => {
            return Err(r#"{"ok":false,"error":"input is not valid UTF-8"}"#.to_string());
        }
    };
    let platform_str = match CStr::from_ptr(platform).to_str() {
        Ok(s) => s,
        Err(_) => {
            return Err(r#"{"ok":false,"error":"platform is not valid UTF-8"}"#.to_string());
        }
    };

    let output_format = match platform_str {
        "dart" | "flutter" => OutputFormat::Dart,
        "kotlin" | "compose" => OutputFormat::Kotlin,
        "swift" | "swiftui" => OutputFormat::Swift,
        other => {
            return Err(format!(
                r#"{{"ok":false,"error":"unsupported platform '{other}'"}}"#
            ));
        }
    };

    let options = CompileOptions {
        platform: output_format,
        minify: false,
        source_maps: false,
        strict: false,
    };

    let ffi = &*compiler;
    match ffi.inner.compile(input_str, &options) {
        Ok(result) => {
            let payload = serde_json::json!({
                "ok": true,
                "output": result.output,
                "stats": {
                    "nodes_parsed": result.stats.nodes_parsed,
                    "css_rules": result.stats.css_rules,
                    "time_ms": result.stats.time_ms,
                }
            });
            Ok(payload.to_string())
        }
        Err(diagnostics) => {
            let errors: Vec<String> = diagnostics
                .iter()
                .map(|d| format!("[{}] {}", d.code.number, d.message))
                .collect();
            let payload = serde_json::json!({
                "ok": false,
                "error": errors.join("; "),
                "diagnostics": diagnostics.iter().map(|d| {
                    serde_json::json!({
                        "code": d.code.number,
                        "message": d.message,
                        "severity": d.severity.as_str(),
                    })
                }).collect::<Vec<_>>(),
            });
            Err(payload.to_string())
        }
    }
}

/// Free a string returned by [`motarjim_compile`].
///
/// # Safety
///
/// `ptr` must be a pointer previously returned by [`motarjim_compile`] or
/// null (which is a no-op). After calling this function the pointer is
/// dangling and must not be used again.
#[no_mangle]
pub unsafe extern "C" fn motarjim_free_string(ptr: *mut std::ffi::c_char) {
    if ptr.is_null() {
        return;
    }
    drop(CString::from_raw(ptr));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a compiler, compile, and free the returned string.
    unsafe fn compile_helper(
        compiler: *mut FfiCompiler,
        input: &std::ffi::CStr,
        platform: &std::ffi::CStr,
    ) -> String {
        let result_ptr = motarjim_compile(compiler, input.as_ptr(), platform.as_ptr());
        assert!(!result_ptr.is_null());
        let result = CStr::from_ptr(result_ptr).to_str().unwrap().to_string();
        motarjim_free_string(result_ptr);
        result
    }

    #[test]
    fn test_new_and_free() {
        let ptr = motarjim_compiler_new();
        assert!(!ptr.is_null());
        unsafe {
            motarjim_compiler_free(ptr);
        }
    }

    #[test]
    fn test_free_null_is_noop() {
        unsafe {
            motarjim_compiler_free(std::ptr::null_mut());
        }
    }

    #[test]
    fn test_compile_simple_html() {
        let compiler = motarjim_compiler_new();
        assert!(!compiler.is_null());

        unsafe {
            let input = std::ffi::CString::new("<div>Hello</div>").unwrap();
            let platform = std::ffi::CString::new("dart").unwrap();
            let json_str = compile_helper(compiler, &input, &platform);

            let value: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
            assert_eq!(value["ok"], serde_json::Value::Bool(true));
            assert!(value["output"].as_str().map_or(false, |o| !o.is_empty()));
            assert!(value["stats"]["nodes_parsed"].as_u64().unwrap_or(0) > 0);

            motarjim_compiler_free(compiler);
        }
    }

    #[test]
    fn test_compile_with_platform_swift() {
        let compiler = motarjim_compiler_new();
        assert!(!compiler.is_null());

        unsafe {
            let input = std::ffi::CString::new("<div>Hello</div>").unwrap();
            let platform = std::ffi::CString::new("swift").unwrap();
            let json_str = compile_helper(compiler, &input, &platform);

            let value: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
            assert_eq!(value["ok"], serde_json::Value::Bool(true));

            motarjim_compiler_free(compiler);
        }
    }

    #[test]
    fn test_compile_with_platform_compose() {
        let compiler = motarjim_compiler_new();
        assert!(!compiler.is_null());

        unsafe {
            let input = std::ffi::CString::new("<div>Hello</div>").unwrap();
            let platform = std::ffi::CString::new("compose").unwrap();
            let json_str = compile_helper(compiler, &input, &platform);

            let value: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
            assert_eq!(value["ok"], serde_json::Value::Bool(true));

            motarjim_compiler_free(compiler);
        }
    }

    #[test]
    fn test_compile_null_compiler() {
        unsafe {
            let input = std::ffi::CString::new("<div></div>").unwrap();
            let platform = std::ffi::CString::new("dart").unwrap();
            let result_ptr =
                motarjim_compile(std::ptr::null_mut(), input.as_ptr(), platform.as_ptr());
            assert!(!result_ptr.is_null());
            let result = CStr::from_ptr(result_ptr).to_str().unwrap().to_string();
            motarjim_free_string(result_ptr);

            let value: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
            assert_eq!(value["ok"], serde_json::Value::Bool(false));
            assert!(value["error"]
                .as_str()
                .map_or(false, |e| e.contains("null compiler")));
        }
    }

    #[test]
    fn test_compile_null_input() {
        let compiler = motarjim_compiler_new();
        assert!(!compiler.is_null());

        unsafe {
            let platform = std::ffi::CString::new("dart").unwrap();
            let result_ptr = motarjim_compile(compiler, std::ptr::null(), platform.as_ptr());
            assert!(!result_ptr.is_null());
            let result = CStr::from_ptr(result_ptr).to_str().unwrap().to_string();
            motarjim_free_string(result_ptr);

            let value: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
            assert_eq!(value["ok"], serde_json::Value::Bool(false));
            assert!(value["error"]
                .as_str()
                .map_or(false, |e| e.contains("null input")));

            motarjim_compiler_free(compiler);
        }
    }

    #[test]
    fn test_compile_invalid_platform() {
        let compiler = motarjim_compiler_new();
        assert!(!compiler.is_null());

        unsafe {
            let input = std::ffi::CString::new("<div></div>").unwrap();
            let platform = std::ffi::CString::new("invalid").unwrap();
            let json_str = compile_helper(compiler, &input, &platform);

            let value: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
            assert_eq!(value["ok"], serde_json::Value::Bool(false));
            assert!(value["error"]
                .as_str()
                .map_or(false, |e| e.contains("unsupported platform")));

            motarjim_compiler_free(compiler);
        }
    }

    #[test]
    fn test_free_string_null_is_noop() {
        unsafe {
            motarjim_free_string(std::ptr::null_mut());
        }
    }
}
