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
    fn on_before_download(
        &mut self,
        _: Browser,
        _: DownloadItem,
        suggested_name: &str,
        callback: cef_ui::BeforeDownloadCallback,
    ) -> bool {
        let full_path = get_file_path(suggested_name);
        callback
            .continue_download(Some(&full_path), false)
            .expect("failed to continue download");
        true
    }
    fn on_download_updated(
        &mut self,
        _: Browser,
        download_item: DownloadItem,
        callback: DownloadItemCallback,
    ) {
        let id = download_item.get_id().expect("failed to get download id");
        self.state.update(|s| {
            if s.downloads.contains_key(&id) {
                return;
            }
            s.downloads.insert(id, callback);
        });

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

fn get_file_path(name: &str) -> String {
    let download_dir = dirs::download_dir().expect("failed to get download dir");
    let mut full_path = download_dir.join(name);

    let mut counter = 1;
    while full_path.exists() {
        let stem = std::path::Path::new(name)
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("invalid file name");
        let extension = std::path::Path::new(name)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| format!(".{}", s))
            .expect("invalid file extension");

        let new_name = format!("{} ({}){}", stem, counter, extension);
        full_path = download_dir.join(new_name);
        counter += 1;
    }

    return full_path.to_str().unwrap_or(name).to_string();
}
