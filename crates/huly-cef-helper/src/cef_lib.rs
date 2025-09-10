use std::env::current_exe;
use std::ffi::c_char;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::os::raw::{c_int, c_void};
use std::mem::zeroed;
use anyhow::{Result, anyhow};
use cef_ui_sys::{cef_app_t, cef_string_userfree_utf16_t, cef_string_utf16_t, char16_t};
use cef_ui_sys::cef_main_args_t;
use cef_ui_sys::cef_process_message_t;
use cef_ui_sys::cef_string_t;
use cef_ui_sys::cef_v8handler_t;
use cef_ui_sys::cef_v8value_t;
use cef_ui_sys::cef_string_userfree_t;
use cef_ui_sys::cef_v8_propertyattribute_t;
use libloading::{Library, Symbol};
use cef_ui::{try_c, CefString, ListValue, V8Value, SchemeRegistrar};

type CefExecuteProcessFn = unsafe extern "C" fn(args: *const cef_main_args_t,app: *mut cef_app_t,extra_info: *mut c_void) -> c_int;
type CefStringUtf8ToUtf16Fn = unsafe extern "C" fn(src: *const c_char,src_len: usize,output: *mut cef_string_t) -> c_int;
type CefStringUtf16SetFn = unsafe extern "C" fn(src: *const char16_t, src_len: usize, output: *mut cef_string_utf16_t, copy: c_int) -> c_int;
type CefStringUsefreeUtf16FreeFn = unsafe extern "C" fn(str_: cef_string_userfree_utf16_t);
type CefRegisterExtensionFn = unsafe extern "C" fn(name: *const cef_string_t,js_code: *const cef_string_t,handler: *mut cef_v8handler_t) -> c_int;
type CefProcessMessageCreateFn = unsafe extern "C" fn (name: *const cef_string_t) -> *mut cef_process_message_t;
type CefV8ValueCreateFunctionFn = unsafe extern "C" fn(name: *const cef_string_t, handler: *mut cef_v8handler_t) -> *mut cef_v8value_t;

pub struct CefLibrary {
    _lib: &'static Library,
    pub cef_execute_process: Symbol<'static, CefExecuteProcessFn>,
    pub cef_string_utf8_to_utf16: Symbol<'static, CefStringUtf8ToUtf16Fn>,
    pub cef_string_utf16_set: Symbol<'static, CefStringUtf16SetFn>,
    pub cef_string_userfree_utf16_free: Symbol<'static, CefStringUsefreeUtf16FreeFn>,
    pub cef_register_extension: Symbol<'static, CefRegisterExtensionFn>,
    pub cef_process_message_create: Symbol<'static, CefProcessMessageCreateFn>,
    pub cef_v8value_create_function: Symbol<'static, CefV8ValueCreateFunctionFn>,
}
const CEF_PATH: &str = "../../../Chromium Embedded Framework.framework/Chromium Embedded Framework";

pub static CEFLIB: LazyLock<CefLibrary> = LazyLock::new(|| {
    unsafe {
        let path = get_cef_path(CEF_PATH).expect("failed to get CEF path");
        let lib = Library::new(path).expect("failed to load CEF library");
        let lib = Box::leak(Box::new(lib));

        let cef_execute_process: Symbol<CefExecuteProcessFn> =
        lib.get(b"cef_execute_process\0").expect("failed to load cef_execute_process");
        let cef_string_utf8_to_utf16: Symbol<CefStringUtf8ToUtf16Fn> =
        lib.get(b"cef_string_utf8_to_utf16\0").expect("failed to load cef_string_utf8_to_utf16");
        let cef_string_utf16_set: Symbol<CefStringUtf16SetFn> =
        lib.get(b"cef_string_utf16_set\0").expect("failed to load cef_string_utf16_set");
        let cef_string_userfree_utf16_free: Symbol<CefStringUsefreeUtf16FreeFn> =
        lib.get(b"cef_string_userfree_utf16_free\0").expect("failed to load cef_string_userfree_utf16_free");
        let cef_register_extension: Symbol<CefRegisterExtensionFn> =
        lib.get(b"cef_register_extension\0").expect("failed to load cef_register_extension");
        let cef_process_message_create: Symbol<CefProcessMessageCreateFn> =
        lib.get(b"cef_process_message_create\0").expect("failed to load cef_process_message_create");
        let cef_v8value_create_function: Symbol<CefV8ValueCreateFunctionFn> =
        lib.get(b"cef_v8value_create_function\0").expect("failed to load cef_v8value_create_function");
        CefLibrary { 
            _lib: lib, 
            cef_execute_process,  
            cef_string_utf8_to_utf16,
            cef_string_utf16_set,
            cef_string_userfree_utf16_free,
            cef_register_extension,
            cef_process_message_create,
            cef_v8value_create_function, 
        }
    }
});

fn get_cef_path(relative_path: &str) -> Result<PathBuf, std::io::Error> {
   let cef_path = current_exe()?
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Could not get parent directory"))?;

    cef_path.join(relative_path).canonicalize()
}

// CEF abstraction functions
pub unsafe fn new_cef_string(s: &str) -> cef_string_t {
    let lib = &CEFLIB;
    let mut ret: cef_string_t = zeroed();

    if s.is_empty() {
        return ret;
    }

    match (lib.cef_string_utf8_to_utf16)(s.as_ptr() as *const c_char, s.len(), &mut ret) {
        0 => panic!("Failed to convert from utf8 to utf16!"),
        _ => ret
    }
}

pub unsafe fn cef_string_from_userfree_ptr(ptr: cef_string_userfree_t) -> Option<cef_string_t> {
    if ptr.is_null() {
        return None;
    }

    let mut ret: cef_string_t = zeroed();

    let lib = &CEFLIB;
    let worked = (lib.cef_string_utf16_set)((*ptr).str_, (*ptr).length, &mut ret, 1);
    (lib.cef_string_userfree_utf16_free)(ptr);
    match worked {
        0 => panic!("Failed to copy cef_string_userfree_t!"),
        _ => Some(ret)
    }
}

// CEF extension traits
pub trait SchemeRegistrarExt {
    fn add_custom_scheme_raw(&self, scheme_name: &str, options: i32) -> Result<bool>;
}

impl SchemeRegistrarExt for SchemeRegistrar {
    fn add_custom_scheme_raw(&self, scheme_name: &str, options: i32) -> Result<bool> {
        unsafe {
            if let Some(add_custom_scheme) = (*self.0).add_custom_scheme {
                Ok(add_custom_scheme(self.0, &new_cef_string(scheme_name), options) != 0)
            } else {
                Err(anyhow!("add_custom_scheme is None"))
            }
        }
    }
}

pub trait ListValueExt {
    fn set_string_raw(&self, index: usize, value: &str) -> Result<bool>;
}

impl ListValueExt for ListValue {
    fn set_string_raw(&self, index: usize, value: &str) -> Result<bool> {
        try_c!(self, set_string, {
            Ok(set_string(self.as_ptr(), index, &new_cef_string(value)) != 0)
        })
    }
}

pub trait V8ValueExt {
    fn get_value_by_key_raw(&self, key: &str) -> Result<V8Value>;
    fn set_value_by_key_raw(&self, key: &str, value: V8Value) -> Result<bool>;
    fn get_string_value_raw(&self) -> Result<String>;
}

impl V8ValueExt for V8Value {
    fn get_value_by_key_raw(&self, key: &str) -> Result<V8Value> {
       try_c!(self, get_value_bykey, {
            Ok(V8Value::from_ptr_unchecked(get_value_bykey(
                self.as_ptr(),
                &new_cef_string(key),
            )))
        })
    }
    
    fn set_value_by_key_raw(&self, key: &str, value: V8Value) -> Result<bool> {
        try_c!(self, set_value_bykey, {
            Ok(set_value_bykey(
                self.as_ptr(),
                &new_cef_string(key),
                value.into_raw(),
                cef_v8_propertyattribute_t::V8_PROPERTY_ATTRIBUTE_NONE
            ) != 0)
        })
    }

    fn get_string_value_raw(&self) -> Result<String> {
        try_c!(self, get_string_value, {
            let s = get_string_value(self.as_ptr());
            let s = match cef_string_from_userfree_ptr(s) {
                Some(str) => str,
                None => return Err(anyhow!("string is empty"))?
            };

           let result = match CefString::from_ptr(&s) {
                Some(str) => str.into(),
                None => Err(anyhow!("string is empty"))?
            };
            Ok(result)
        })
    }
}
