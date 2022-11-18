use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::BunyanFormattingLayer;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Sync + Send
where
    Sink: for<'a> MakeWriter<'a> + Sync + Send + 'static,
{
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let fmt = BunyanFormattingLayer::new(name.into(), sink);

    Registry::default().with(filter).with(fmt)
}

pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
