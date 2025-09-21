use cef_ui::{
    BeforeDownloadCallback, Browser, DownloadHandlerCallbacks, DownloadItem, DownloadItemCallback,
};
use log::info;

use crate::{state::SharedBrowserState, TabMessage};

pub struct MyDownloadHandlerCallbacks {
    state: SharedBrowserState,
}

impl MyDownloadHandlerCallbacks {
    pub(crate) fn new(state: SharedBrowserState) -> Self {
        Self { state }
    }
}
impl DownloadHandlerCallbacks for MyDownloadHandlerCallbacks {
    fn on_before_download(
        &mut self,
        _: Browser,
        _: DownloadItem,
        suggested_name: &str,
        callback: BeforeDownloadCallback,
    ) -> bool {
        info!("Trying to get download directory 1");
        let download_dir = dirs::download_dir().expect("failed to get download directory");
        info!("Trying to get download directory 2");
        let download_path = download_dir.join(suggested_name);
        info!("Trying to get download directory 3");
        let download_path = download_path
            .to_str()
            .expect("failed to convert path to str");

        info!("Downloading to {}", download_path);

        self.state
            .notify(TabMessage::Download(download_path.to_string()));
        _ = callback.continue_download(Some(download_path), false);
        true
    }

    fn on_download_updated(
        &mut self,
        _: Browser,
        download_item: DownloadItem,
        _: DownloadItemCallback,
    ) {
        let received = download_item
            .get_received_bytes()
            .expect("failed to get received bytes") as u64;
        let total = download_item
            .get_total_bytes()
            .expect("failed to get total bytes") as u64;

        info!("Download updated: {}/{}", received, total);
        self.state
            .notify(TabMessage::DownloadProgress { received, total });
    }
}
