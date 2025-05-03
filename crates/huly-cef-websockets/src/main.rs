use anyhow::Result;

use clap::Parser;
use tracing::{level_filters::LevelFilter, subscriber::set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

mod websocket;

#[derive(Parser)]
struct Arguments {
    #[arg(short, long, default_value = "8080")]
    port: u16,
}

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

    let args = Arguments::parse();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(websocket::serve(format!("127.0.0.1:{}", args.port)));

    _ = cef.initialize();

    cef.run_message_loop();
    cef.shutdown();

    Ok(())
}
