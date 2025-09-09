use std::{fs, path::PathBuf};

use cef_ui::{
    register_scheme_handler_factory, Browser, BrowserProcessHandlerCallbacks, Callback,
    CommandLine, Frame, Request, ResourceHandler, ResourceHandlerCallbacks, Response,
    SchemeHandlerFactory, SchemeHandlerFactoryCallbacks,
};
use log::info;

struct HulyResourceHandlerCallbacks {
    file_content: Option<Vec<u8>>,
    position: usize,
}

impl HulyResourceHandlerCallbacks {
    fn new() -> Self {
        Self {
            file_content: None,
            position: 0,
        }
    }
}

impl ResourceHandlerCallbacks for HulyResourceHandlerCallbacks {
    fn open(&mut self, request: Request, handle_request: &mut bool, _callback: Callback) -> bool {
        let url = request.get_url().expect("failed to get request URL");
        if url == "huly://newtab" {
            let file_path = std::env::current_exe()
                .expect("failed to get current exe path")
                .parent()
                .expect("failed to get parent directory of a current exe")
                .join("cef/huly-cef-resources/new-tab.html");

            let content = fs::read(file_path).expect("failed to read new-tab.html");

            self.file_content = Some(content);
            self.position = 0;
            *handle_request = true;
            return true;
        };

        false
    }

    fn get_response_headers(
        &mut self,
        response: Response,
        response_length: &mut i64,
        _redirect_url: &mut String,
    ) {
        if let Some(content) = &self.file_content {
            response
                .set_mime_type("text/html")
                .expect("failed to set mime type");
            response.set_status(200).expect("failed to set status");
            *response_length = content.len() as i64;
        }
    }

    fn read_response(
        &mut self,
        data_out: *mut std::os::raw::c_void,
        bytes_to_read: std::ffi::c_int,
        bytes_read: &mut std::ffi::c_int,
        _callback: cef_ui::Callback,
    ) -> bool {
        if let Some(content) = &self.file_content {
            let remaining = content.len() - self.position;
            let to_read = std::cmp::min(bytes_to_read as usize, remaining);

            if to_read > 0 {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        content.as_ptr().add(self.position),
                        data_out as *mut u8,
                        to_read,
                    );
                }
                self.position += to_read;
                *bytes_read = to_read as std::ffi::c_int;
                true
            } else {
                *bytes_read = 0;
                false
            }
        } else {
            *bytes_read = 0;
            false
        }
    }

    fn cancel(&mut self) {
        self.file_content = None;
        self.position = 0;
    }
}

struct HulySchemeHandlerFactoryCallbacks;

impl SchemeHandlerFactoryCallbacks for HulySchemeHandlerFactoryCallbacks {
    fn create(
        &mut self,
        _browser: Browser,
        _frame: Frame,
        _scheme_name: &str,
        _request: Request,
    ) -> Option<ResourceHandler> {
        Some(ResourceHandler::new(HulyResourceHandlerCallbacks::new()))
    }
}

pub struct BrowserProcessCallbacks {
    port: u16,
    cache_path: String,
}

impl BrowserProcessCallbacks {
    pub fn new(port: u16, cache_path: String) -> Self {
        Self { port, cache_path }
    }
}

impl BrowserProcessHandlerCallbacks for BrowserProcessCallbacks {
    fn on_before_child_process_launch(&mut self, command_line: CommandLine) {
        _ = command_line.append_switch_with_value("port", Some(&self.port.to_string()));
        _ = command_line.append_switch_with_value("cache-path", Some(&self.cache_path));
    }

    fn on_context_initialized(&mut self) {
        register_scheme_handler_factory(
            "huly",
            "",
            SchemeHandlerFactory::new(HulySchemeHandlerFactoryCallbacks {}),
        );
    }
}
