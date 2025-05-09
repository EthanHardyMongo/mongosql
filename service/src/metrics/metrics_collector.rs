use log::debug;
use prometheus::{register_int_counter, register_int_counter_vec};
use prometheus::{HistogramVec, IntCounter, IntCounterVec, Registry};
use std::sync::{Arc, LazyLock};
use tonic::service::Interceptor;
use tonic::{Request, Status};

pub const METRIC_SERVER_STARTED_TOTAL: &str = "grpc_server_started_total";
pub const METRIC_SERVER_HANDLED_TOTAL: &str = "grpc_server_handled_total";
pub const METRIC_SERVER_HANDLING_SECONDS: &str = "grpc_server_handling_seconds";
pub const METRIC_ERRORS_TOTAL: &str = "grpc_errors_total";
pub const METRIC_PANICS_TOTAL: &str = "grpc_panics_total";

// Metrics for gRPC server using Prometheus
pub static SERVER_STARTED_COUNTER: LazyLock<IntCounterVec> = LazyLock::new(|| {
    IntCounterVec::new(
        prometheus::opts!(
            METRIC_SERVER_STARTED_TOTAL,
            "Total number of RPCs started on the server."
        ),
        &["grpc_service", "grpc_method"],
    )
    .expect("Failed to create SERVER_STARTED_COUNTER")
});

pub static SERVER_HANDLED_COUNTER: LazyLock<IntCounterVec> = LazyLock::new(|| {
    IntCounterVec::new(
        prometheus::opts!(
            METRIC_SERVER_HANDLED_TOTAL,
            "Total number of RPCs completed on the server, regardless of success or failure."
        ),
        &["grpc_service", "grpc_method", "grpc_code"],
    )
    .expect("Failed to create SERVER_HANDLED_COUNTER")
});

pub static SERVER_HANDLED_HISTOGRAM: LazyLock<HistogramVec> = LazyLock::new(|| {
    HistogramVec::new(
        prometheus::histogram_opts!(
            METRIC_SERVER_HANDLING_SECONDS,
            "Histogram of response latency (seconds) of gRPC that had been application-level \
            handled by the server."
        ),
        &["grpc_service", "grpc_method"],
    )
    .expect("Failed to create SERVER_HANDLED_HISTOGRAM")
});

pub static SERVER_ERRORS_TOTAL: LazyLock<IntCounterVec> = LazyLock::new(|| {
    register_int_counter_vec!(
        METRIC_ERRORS_TOTAL,
        "Total number of gRPC errors",
        &["code"]
    )
    .expect("Failed to create SERVER_ERRORS_TOTAL")
});

pub static SERVER_PANICS_TOTAL: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!(METRIC_PANICS_TOTAL, "Total number of panics in gRPC calls")
        .expect("Failed to create SERVER_PANICS_TOTAL")
});

pub fn register_metrics(registry: &Registry) {
    registry
        .register(Box::new(SERVER_STARTED_COUNTER.clone()))
        .expect("Failed to register SERVER_STARTED_COUNTER");
    registry
        .register(Box::new(SERVER_HANDLED_COUNTER.clone()))
        .expect("Failed to register SERVER_HANDLED_COUNTER");
    registry
        .register(Box::new(SERVER_HANDLED_HISTOGRAM.clone()))
        .expect("Failed to register SERVER_HANDLED_HISTOGRAM");
    registry
        .register(Box::new(SERVER_ERRORS_TOTAL.clone()))
        .expect("Failed to register SERVER_ERRORS_TOTAL");
    registry
        .register(Box::new(SERVER_PANICS_TOTAL.clone()))
        .expect("Failed to register SERVER_PANICS_TOTAL");
}

// Interceptor for recording gRPC errors
#[derive(Clone)]
pub struct ErrorInterceptor {
    error_counter: Arc<IntCounterVec>,
}

impl Default for ErrorInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorInterceptor {
    pub fn new() -> Self {
        ErrorInterceptor {
            error_counter: Arc::from(SERVER_ERRORS_TOTAL.clone()),
        }
    }

    // Records an error with the given gRPC status
    pub fn record_error(&self, status: &Status) {
        debug!(
            "Recording gRPC error metric: code={}, message='{}'",
            status.code().to_string(),
            status.message()
        );
        self.error_counter
            .with_label_values(&[status.code().to_string().as_str()])
            .inc();
    }
}

impl Interceptor for ErrorInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        request.extensions_mut().insert(self.clone());
        Ok(request)
    }
}
