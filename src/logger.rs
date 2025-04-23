use tracing::Level;
use tracing_subscriber::fmt::format::Format;

pub fn init_logging() {
    let format = Format::default()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(true)
        .with_level(true);

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .event_format(format)
        .init();
}
