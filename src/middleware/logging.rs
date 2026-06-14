use tower_http::trace::{self, TraceLayer};
use tracing::Level;

pub fn trace_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    trace::DefaultMakeSpan,
    trace::DefaultOnRequest,
    trace::DefaultOnResponse,
    trace::DefaultOnBodyChunk,
    trace::DefaultOnEos,
    trace::DefaultOnFailure,
> {
    TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
}
