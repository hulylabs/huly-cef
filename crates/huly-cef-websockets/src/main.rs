use anyhow::Result;

use log::SetLoggerError;
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};

mod server;

#[derive(Debug)]
struct Arguments {
    port: u16,
    cache_path: String,
    use_server_size: bool,
}

impl Default for Arguments {
    fn default() -> Self {
        let cache_path = std::env::temp_dir().join("huly_cef_cache");
        Arguments {
            port: 8080,
            cache_path: cache_path.to_str().unwrap_or_default().to_string(),
            use_server_size: false,
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

fn parse_argument_without_value(args: &Vec<String>, flag: &str) -> bool {
    for arg in args {
        if arg == flag {
            return true;
        }
    }
    false
}

fn parse_arguments() -> Arguments {
    let mut result = Arguments::default();
    let args: Vec<String> = std::env::args().collect();

    match parse_argument(&args, "--port") {
        Ok(port) => match port.parse::<u16>() {
            Ok(port) => result.port = port,
            Err(_) => log::warn!(
                "Invalid port number provided, using default: {}",
                result.port
            ),
        },
        Err(_) => log::warn!("No port argument provided, using default: {}", result.port),
    }

    match parse_argument(&args, "--cache-path") {
        Ok(path) => result.cache_path = path,
        Err(_) => log::warn!(
            "No cache path argument provided, using default: {}",
            result.cache_path
        ),
    }

    result.use_server_size = parse_argument_without_value(&args, "--use-server-size");
    result
}

fn setup_logging(cache_dir: &str) -> Result<log4rs::Handle, SetLoggerError> {
    let stdout_pattern = "\x1b[90m{d(%H:%M:%S%.3f)} \x1b[0m{h({l})} \x1b[90m{f}:{L} \x1b[0m{m}{n}";
    let file_pattern = "{d(%H:%M:%S%.3f)} {h({l})} {f}:{L} {m}{n}";
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(stdout_pattern)))
        .build();
    let file = FileAppender::builder()
        .append(false)
        .encoder(Box::new(PatternEncoder::new(file_pattern)))
        .build(format!("{}/huly-cef.log", cache_dir))
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("file")
                .build(log::LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config)
}

fn main() {
    let args = parse_arguments();

    setup_logging(&args.cache_path).expect("failed to set up logging");

    let cef =
        huly_cef::new(args.port, args.cache_path.clone()).expect("failed to create CEF instance");
    if let Some(code) = cef.is_cef_subprocess() {
        std::process::exit(code);
    }

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.spawn(server::serve(
        format!("0.0.0.0:{}", args.port),
        args.cache_path,
        args.use_server_size,
    ));

    _ = cef.initialize();

    cef.run_message_loop();
    cef.shutdown();
}
