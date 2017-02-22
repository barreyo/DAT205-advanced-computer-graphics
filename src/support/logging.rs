
extern crate alewife;
extern crate log;

use core::event;

use log::{LogRecord, LogLevel, LogMetadata, SetLoggerError, LogLevelFilter};

pub struct LogBuilder {
    publisher: Option<alewife::Publisher<event::EventID, event::Event>>,
}

impl LogBuilder {
    pub fn new() -> LogBuilder {
        LogBuilder {
            publisher: None,
        }
    }

    pub fn publisher(&mut self, p: alewife::Publisher<event::EventID, event::Event>) -> &mut Self {
        self.publisher = Some(p);
        self
    }

    pub fn init(&mut self) -> Result<(), SetLoggerError> {
        log::set_logger(|max_level| {
            let logger = self.build();
            max_level.set(LogLevelFilter::Info);
            Box::new(logger)
        })
    }

    pub fn build(&mut self) -> Logger {
        Logger {
            publisher: self.publisher.clone(),
        }
    }
}

pub struct Logger {
    publisher: Option<alewife::Publisher<event::EventID, event::Event>>,
}

unsafe impl Sync for Logger {}

impl log::Log for Logger {

    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {

        use ui::console::ConsoleLogLevel;

        if self.enabled(record.metadata()) {
            let s = format!("{}: {}", record.level(), record.args());
            println!("{}", s);
            if let Some(ref p) = self.publisher {
                match record.level() {
                    LogLevel::Error => p.publish(event::EventID::UIEvent, event::Event::ConsoleMessage(s, ConsoleLogLevel::ERROR)),
                    LogLevel::Warn  => p.publish(event::EventID::UIEvent, event::Event::ConsoleMessage(s, ConsoleLogLevel::WARNING)),
                    LogLevel::Info  => p.publish(event::EventID::UIEvent, event::Event::ConsoleMessage(s, ConsoleLogLevel::INFO)),
                    _               => p.publish(event::EventID::UIEvent, event::Event::ConsoleMessage(s, ConsoleLogLevel::INFO))
                }
            }
        }
    }
}
