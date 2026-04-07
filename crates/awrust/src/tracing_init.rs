use tracing_subscriber::{EnvFilter, fmt};

pub fn init(filter: &str) {
    let filter = EnvFilter::try_new(filter).expect("valid log filter");

    fmt()
        .json()
        .with_env_filter(filter)
        .with_current_span(true)
        .with_span_list(true)
        .init();
}
