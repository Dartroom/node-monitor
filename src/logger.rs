pub use slog::{info, o, trace, warn, Drain, Logger};

pub fn configure_log() -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let console_drain = slog_term::FullFormat::new(decorator).build().fuse();

    // It is used for Synchronization
    let console_drain = slog_async::Async::new(console_drain).build().fuse();

    // Root logger
    slog::Logger::root(console_drain, o!("v"=>env!("CARGO_PKG_VERSION")))
}

lazy_static! {
    pub static ref LOGGER: Logger = configure_log();
}
