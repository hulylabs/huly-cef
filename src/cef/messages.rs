pub enum CefMessage {
    Render(Vec<u8>),
    IsLoading,
    Loaded,
    LoadError,
}
