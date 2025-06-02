use anyhow::Result;

use tracing::{level_filters::LevelFilter, subscriber::set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

mod websocket;

#[derive(Debug)]
struct Arguments {
    port: u16,
    cache_path: String,
}

impl Default for Arguments {
    fn default() -> Self {
        let cache_path = std::env::temp_dir().join("huly_cef_cache");
        Arguments {
            port: 8080,
            cache_path: cache_path.to_str().unwrap_or_default().to_string(),
        }
    }
}

// I had to write this function because the `clap` crate doesn't work well with CEF arguments
fn parse_argument(args: &Vec<String>, flag: &str) -> Result<String> {
    for (i, arg) in args.iter().enumerate() {
        if arg.contains(flag) {
            if arg.contains("=") {
                return Ok(arg.split('=').nth(1).unwrap_or("").to_string());
            } else if i + 1 < args.len() {
                return Ok(args[i + 1].clone());
            } else {
                return Err(anyhow::anyhow!("No value provided for argument '{}'", flag));
            }
        }
    }

    Err(anyhow::anyhow!("Argument '{}' not found", flag))
}

fn parse_arguments() -> Arguments {
    let mut result = Arguments::default();
    let args: Vec<String> = std::env::args().collect();

    match parse_argument(&args, "--port") {
        Ok(port) => match port.parse::<u16>() {
            Ok(port) => result.port = port,
            Err(_) => log::error!(
                "Invalid port number provided, using default: {}",
                result.port
            ),
        },
        Err(_) => log::error!("No port argument provided, using default: {}", result.port),
    }

    match parse_argument(&args, "--cache-path") {
        Ok(path) => result.cache_path = path,
        Err(_) => log::error!(
            "No cache path argument provided, using default: {}",
            result.cache_path
        ),
    }

    result
}

fn main() -> Result<()> {
    LogTracer::init()?;
    let subscriber = FmtSubscriber::builder()
        .with_max_level(LevelFilter::INFO)
        .finish();

    set_global_default(subscriber)?;

    let args = parse_arguments();

    let cef = huly_cef::new(args.port, args.cache_path)?;

    if let Some(code) = cef.is_cef_subprocess() {
        std::process::exit(code);
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(websocket::serve(format!("127.0.0.1:{}", args.port)));

    _ = cef.initialize();

    cef.run_message_loop();
    cef.shutdown();

    Ok(())
}
