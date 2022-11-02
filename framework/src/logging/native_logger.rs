use super::LogRecord;

pub struct DefaultLogger {
    listeners: Vec<Box<dyn Fn(LogRecord) + Send + Sync>>,
    default_level_filter: log::LevelFilter,
    crate_level_filters: Vec<(String, log::LevelFilter)>,
}

impl DefaultLogger {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
            default_level_filter: log::LevelFilter::Trace,
            crate_level_filters: Vec::new(),
        }
    }

    pub fn with_global_level_filter(mut self, level: log::LevelFilter) -> Self {
        self.default_level_filter = level;

        self
    }

    pub fn with_crate_level_filter(mut self, crate_name: &str, level: log::LevelFilter) -> Self {
        self.crate_level_filters
            .push((crate_name.replace('-', "_"), level));

        self
    }

    pub fn with_listener(mut self, listener: impl Fn(LogRecord) + Send + Sync + 'static) -> Self {
        self.listeners.push(Box::new(listener));
        self
    }

    pub fn init(self) -> Result<(), log::SetLoggerError> {
        // allow everything, we'll filter logs within the logger
        log::set_max_level(log::LevelFilter::Trace);
        log::set_boxed_logger(Box::new(self))
    }
}

impl Default for DefaultLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl log::Log for DefaultLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        let filter_tuple = self
            .crate_level_filters
            .iter()
            .find(|(name, _)| metadata.target().starts_with(name));

        if let Some((_, level)) = filter_tuple {
            metadata.level() <= *level
        } else {
            metadata.level() <= self.default_level_filter
        }
    }

    fn log(&self, record: &log::Record) {
        if !self.listeners.is_empty() {
            let record = LogRecord {
                target: record.target().to_string(),
                level: record.level(),
                message: format!("{}", record.args()),
            };

            for listener in &self.listeners {
                listener(record.clone());
            }
        }

        use std::io::Write;
        use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

        if !self.enabled(record.metadata()) {
            return;
        }

        let mut color_spec = ColorSpec::new();

        match record.level() {
            log::Level::Error => {
                color_spec.set_fg(Some(Color::Red));
            }
            log::Level::Warn => {
                color_spec.set_fg(Some(Color::Yellow));
            }
            log::Level::Info => {}
            log::Level::Trace | log::Level::Debug => {
                color_spec.set_dimmed(true);
            }
        };

        let mut stderr = StandardStream::stderr(ColorChoice::Always);

        stderr.set_color(&color_spec).unwrap();
        write!(&mut stderr, "{} ", record.level()).unwrap();

        stderr.reset().unwrap();
        writeln!(&mut stderr, "[{}] {}", record.target(), record.args()).unwrap();
    }

    fn flush(&self) {}
}
