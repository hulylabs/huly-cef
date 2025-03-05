use anyhow::Result;

mod cef;
mod websocket;

fn main() -> Result<()> {
    let cef = cef::new()?;

    if let Some(code) = cef.is_cef_subprocess() {
        std::process::exit(code);
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(websocket::serve());

    _ = cef.initialize();

    cef.run_message_loop();
    cef.shutdown();

    Ok(())
}
