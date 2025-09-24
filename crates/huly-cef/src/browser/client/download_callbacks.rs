use crate::{state::SharedBrowserState, TabMessage};
use cef_ui::{Browser, DownloadHandlerCallbacks, DownloadItem, DownloadItemCallback};

pub struct MyDownloadHandlerCallbacks {
    state: SharedBrowserState,
}

impl MyDownloadHandlerCallbacks {
    pub(crate) fn new(state: SharedBrowserState) -> Self {
        Self { state }
    }
}
impl DownloadHandlerCallbacks for MyDownloadHandlerCallbacks {
    fn on_download_updated(
        &mut self,
        _: Browser,
        download_item: DownloadItem,
        _: DownloadItemCallback,
    ) {
        let id = download_item.get_id().expect("failed to get download id");
        let path = download_item
            .get_full_path()
            .expect("failed to get full path");
        let received = download_item
            .get_received_bytes()
            .expect("failed to get received bytes") as u64;
        let total = download_item
            .get_total_bytes()
            .expect("failed to get total bytes") as u64;

        let is_complete = download_item
            .is_complete()
            .expect("failed to check if download is complete");

        let is_aborted = download_item
            .is_canceled()
            .expect("failed to check if download is canceled");

        self.state.notify(TabMessage::DownloadProgress {
            id,
            path,
            received,
            total,
            is_complete,
            is_aborted,
        });
    }
}
