use anyhow::Result;

use tracing::{level_filters::LevelFilter, subscriber::set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

mod websocket;

fn main() -> Result<()> {
    LogTracer::init()?;
    let subscriber = FmtSubscriber::builder()
        .with_max_level(LevelFilter::INFO)
        .finish();

    set_global_default(subscriber)?;

    let cef = huly_cef::new()?;

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
