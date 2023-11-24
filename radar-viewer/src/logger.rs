use std::sync::mpsc;



pub struct Logger {
    logging_level: log::Level,
    sender: mpsc::Sender<String>,
}

impl Logger {
    pub fn initialise(level: log::Level) -> mpsc::Receiver<String> {
        let (tx, rx) = mpsc::channel();
        let logger = Logger {
            logging_level: level,
            sender: tx,
        };
        log::set_boxed_logger(Box::new(logger)).unwrap();
        log::set_max_level(log::LevelFilter::Trace);
        rx
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.logging_level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            self.sender.send(record.args().to_string()).unwrap();
        }
    }

    fn flush(&self) {
        println!("flush called");
    }
}