use env_logger::{Builder, Env};
use log::LevelFilter;
use std::io::Write;
use termcolor::{Color, ColorSpec, WriteColor};

pub use log::{debug, error, info, trace, warn};

pub fn init_logger() {
    Builder::from_env(Env::default().default_filter_or("lattice=info"))
        .filter_module("nannou", LevelFilter::Warn)
        .format(|_buf, record| {
            let buffer_writer =
                termcolor::BufferWriter::stdout(termcolor::ColorChoice::Auto);
            let mut buffer = buffer_writer.buffer();
            let mut spec = ColorSpec::new();

            spec.set_fg(Some(match record.level() {
                log::Level::Trace => Color::Cyan,
                log::Level::Debug => Color::Blue,
                log::Level::Info => Color::Green,
                log::Level::Warn => Color::Yellow,
                log::Level::Error => Color::Red,
            }))
            .set_bold(true);

            buffer.set_color(&spec)?;
            write!(buffer, "[{}]", record.level())?;

            buffer.reset()?;
            writeln!(buffer, " {}", record.args())?;

            buffer_writer.print(&buffer)?;
            Ok(())
        })
        .init();
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! debug_throttled {
    ($interval_ms:expr, $($arg:tt)*) => {
        {
            // Encapsulate imports within the macro
            use std::time::{Duration, Instant};
            use std::sync::Mutex;
            use log::debug;

            // Lazy initialization of throttle map
            lazy_static::lazy_static! {
                static ref DEBUG_THROTTLE: Mutex<std::collections::HashMap<&'static str, Instant>> =
                    Mutex::new(std::collections::HashMap::new());
            }

            // Throttle logic
            let key = stringify!($($arg)*);
            let interval = Duration::from_millis($interval_ms as u64);
            let mut throttle_map = DEBUG_THROTTLE.lock().unwrap();
            let now = Instant::now();

            if throttle_map.get(key).map_or(true, |&last_log_time| now.duration_since(last_log_time) >= interval) {
                throttle_map.insert(key, now);
                debug!($($arg)*);
            }
        }
    };
}
