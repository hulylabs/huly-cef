use std::{ffi::c_char, mem::zeroed, ptr::null_mut};

use anyhow::Result;
use anyhow::anyhow;
use cef_ui::try_c;
use cef_ui::CefString;
use cef_ui::ListValue;
use cef_ui::{Browser, Frame, ProcessId, ProcessMessage, RenderProcessHandlerCallbacks, V8Context, V8Handler, V8HandlerCallbacks, V8Value};
use cef_ui_sys::cef_string_t;
use cef_ui_sys::cef_string_userfree_t;
use cef_ui_sys::cef_v8_propertyattribute_t;

use crate::cef_lib;
use crate::js;

unsafe fn new_cef_string(s: &str) -> cef_string_t {
    let lib = &cef_lib::CEFLIB;
    let mut ret: cef_string_t = unsafe { zeroed() };

    if s.is_empty() {
        return ret;
    }

    match (lib.cef_string_utf8_to_utf16)(s.as_ptr() as *const c_char, s.len(), &mut ret) {
        0 => panic!("Failed to convert from utf8 to utf16!"),
        _ => ret
    }
}

unsafe fn cef_string_from_userfree_ptr(ptr: cef_string_userfree_t) -> Option<cef_string_t> {
    if ptr.is_null() {
        return None;
    }

    let mut ret: cef_string_t = unsafe { zeroed() };

    unsafe {
        let lib = &cef_lib::CEFLIB;
        let worked = (lib.cef_string_utf16_set)((*ptr).str_, (*ptr).length, &mut ret, 1);
        (lib.cef_string_userfree_utf16_free)(ptr);
        match worked {
            0 => panic!("Failed to copy cef_string_userfree_t!"),
            _ => Some(ret)
        }
    }
}

trait ListValueExt {
    fn set_string_raw(&self, index: usize, value: &str) -> Result<bool>;
}

impl ListValueExt for ListValue {
    fn set_string_raw(&self, index: usize, value: &str) -> Result<bool> {
        try_c!(self, set_string, {
            Ok(set_string(self.as_ptr(), index, &new_cef_string(value)) != 0)
        })
    }
}

trait V8ValueExt {
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
                None => return Err(anyhow::anyhow!("string is empty"))?
            };

           let result = match CefString::from_ptr(&s) {
                Some(str) => str.into(),
                None => Err(anyhow::anyhow!("string is empty"))?
            };
            Ok(result)
        })
    }
}

pub struct RenderProcessCallbacks;

impl RenderProcessHandlerCallbacks for RenderProcessCallbacks {
    fn on_web_kit_initialized(&mut self) {
        let lib = &cef_lib::CEFLIB; 
        unsafe {
            (lib.cef_register_extension)(&new_cef_string("is_interactive_element"), &new_cef_string(js::INTERACTIVE_ELEMENT_FUNCTION), null_mut());
            (lib.cef_register_extension)(&new_cef_string("is_element_visible"), &new_cef_string(js::IS_ELEMENT_VISIBLE_FUNCTION), null_mut());
            (lib.cef_register_extension)(&new_cef_string("walk_dom"), &new_cef_string(js::WALK_DOM_FUNCTION), null_mut());
        }
    }

    fn on_context_created(&mut self, browser: Browser, frame: Frame, context: V8Context) {
        if !frame.is_main().unwrap() {
            return;
        }

        unsafe {
            let lib = &cef_lib::CEFLIB;

            let func = (lib.cef_v8value_create_function)(
                &new_cef_string("sendMessage"),
                V8Handler::new(SendMessageHandler::new(browser)).into_raw(),
            );
            let func = V8Value::from_ptr(func)
                .expect("failed to create func sendMessage");

            context
                .get_global()
                .expect("failed to get global context object")
                .set_value_by_key_raw("sendMessage", func)
                .expect("failed to set sendMessage function");
        }
    }
}

pub struct SendMessageHandler {
    browser: Browser,
}

impl SendMessageHandler {
    pub fn new(browser: Browser) -> Self {
        Self { browser }
    }
}

impl V8HandlerCallbacks for SendMessageHandler {
    fn execute(&mut self, _: String, _: V8Value, _: usize, arguments: Vec<V8Value>) -> Result<i32> {
        let first_arg = arguments.get(0).expect("first argument is required");

        let id = first_arg
            .get_value_by_key_raw("id")
            .expect("failed to get id")
            .get_string_value_raw()
            .expect("id must be a string");

        let message = first_arg
            .get_value_by_key_raw("message")
            .expect("failed to get message")
            .get_string_value_raw()
            .expect("message must be a string");

        let ipc_message = unsafe {
            let lib = &cef_lib::CEFLIB;
            let msg = (lib.cef_process_message_create)(&new_cef_string("javascript_message"));
            ProcessMessage::from_ptr(msg).expect("failed to create process message")
        };
        let argument_list = ipc_message
            .get_argument_list()
            .ok()
            .flatten()
            .expect("failed to get argument list");
        _ = argument_list.set_string_raw(0, &id);
        _ = argument_list.set_string_raw(1, &message);

        _ = self
            .browser
            .get_main_frame()
            .unwrap()
            .unwrap()
            .send_process_message(ProcessId::Browser, ipc_message);

        Ok(1)
    }
}
