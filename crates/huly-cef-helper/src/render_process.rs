// use std::ptr::null_mut;

// use anyhow::Result;
// use cef_ui::{
//     Browser, Frame, ProcessId, ProcessMessage, RenderProcessHandlerCallbacks, V8Context, V8Handler,
//     V8HandlerCallbacks, V8Value,
// };

// use crate::cef_lib::{self, new_cef_string, ListValueExt, V8ValueExt};
// use crate::js;

// pub struct RenderProcessCallbacks;

// impl RenderProcessHandlerCallbacks for RenderProcessCallbacks {
//     fn on_web_kit_initialized(&mut self) {
//         let lib = &cef_lib::CEFLIB;
//         unsafe {
//             (lib.cef_register_extension)(
//                 &new_cef_string("is_interactive_element"),
//                 &new_cef_string(js::INTERACTIVE_ELEMENT_FUNCTION),
//                 null_mut(),
//             );
//             (lib.cef_register_extension)(
//                 &new_cef_string("is_element_visible"),
//                 &new_cef_string(js::IS_ELEMENT_VISIBLE_FUNCTION),
//                 null_mut(),
//             );
//             (lib.cef_register_extension)(
//                 &new_cef_string("walk_dom"),
//                 &new_cef_string(js::WALK_DOM_FUNCTION),
//                 null_mut(),
//             );
//         }
//     }

//     fn on_context_created(&mut self, browser: Browser, frame: Frame, context: V8Context) {
//         if !frame.is_main().unwrap() {
//             return;
//         }

//         unsafe {
//             let lib = &cef_lib::CEFLIB;

//             let func = (lib.cef_v8value_create_function)(
//                 &new_cef_string("sendMessage"),
//                 V8Handler::new(SendMessageHandler::new(browser)).into_raw(),
//             );
//             let func = V8Value::from_ptr(func).expect("failed to create func sendMessage");

//             context
//                 .get_global()
//                 .expect("failed to get global context object")
//                 .set_value_by_key_raw("sendMessage", func)
//                 .expect("failed to set sendMessage function");
//         }
//     }
// }

// pub struct SendMessageHandler {
//     browser: Browser,
// }

// impl SendMessageHandler {
//     pub fn new(browser: Browser) -> Self {
//         Self { browser }
//     }
// }

// impl V8HandlerCallbacks for SendMessageHandler {
//     fn execute(&mut self, _: String, _: V8Value, _: usize, arguments: Vec<V8Value>) -> Result<i32> {
//         let first_arg = arguments.get(0).expect("first argument is required");

//         let id = first_arg
//             .get_value_by_key_raw("id")
//             .expect("failed to get id")
//             .get_string_value_raw()
//             .expect("id must be a string");

//         let message = first_arg
//             .get_value_by_key_raw("message")
//             .expect("failed to get message")
//             .get_string_value_raw()
//             .expect("message must be a string");

//         let ipc_message = unsafe {
//             let lib = &cef_lib::CEFLIB;
//             let msg = (lib.cef_process_message_create)(&new_cef_string("javascript_message"));
//             ProcessMessage::from_ptr(msg).expect("failed to create process message")
//         };
//         let argument_list = ipc_message
//             .get_argument_list()
//             .ok()
//             .flatten()
//             .expect("failed to get argument list");
//         _ = argument_list.set_string_raw(0, &id);
//         _ = argument_list.set_string_raw(1, &message);

//         _ = self
//             .browser
//             .get_main_frame()
//             .unwrap()
//             .unwrap()
//             .send_process_message(ProcessId::Browser, ipc_message);

//         Ok(1)
//     }
// }
