use std::result;

use cef_ui::{
    BrowserProcessHandlerCallbacks, Client, CommandLine, PreferenceRegistrar, PreferencesType,
};

pub struct HulyBrowserProcessHandlerCallbacks {
    cache_path: String,
    root_cache_path: String,
}

impl HulyBrowserProcessHandlerCallbacks {
    pub fn new(cache_path: String, root_cache_path: String) -> Self {
        Self {
            cache_path,
            root_cache_path,
        }
    }
}

impl BrowserProcessHandlerCallbacks for HulyBrowserProcessHandlerCallbacks {
    fn on_before_child_process_launch(&mut self, command_line: CommandLine) {
        _ = command_line.append_switch_with_value("port", Some("8080"));
        _ = command_line.append_switch_with_value("cache-path", Some(&self.cache_path));
        _ = command_line.append_switch_with_value("root-cache-path", Some(&self.root_cache_path));
    }

    fn on_register_custom_preferences(
        &mut self,
        _preferences_type: PreferencesType,
        _registrar: &mut PreferenceRegistrar,
    ) {
    }

    fn on_context_initialized(&mut self) {}

    fn on_already_running_app_relaunch(
        &mut self,
        _command_line: cef_ui::CommandLine,
        _current_directory: &str,
    ) -> bool {
        false
    }

    fn on_schedule_message_pump_work(&mut self, _delay_ms: i64) {}

    fn get_default_client(&mut self) -> Option<Client> {
        None
    }
}
