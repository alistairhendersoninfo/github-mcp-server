use prometheus::{Counter, Histogram, Gauge, Registry, Encoder, TextEncoder, Opts, HistogramOpts};
use std::sync::Arc;
use axum::{
    extract::State,
    response::{Response, IntoResponse},
    http::{StatusCode, header},
};

#[derive(Clone)]
pub struct Metrics {
    pub registry: Arc<Registry>,
    pub http_requests_total: Counter,
    pub http_request_duration: Histogram,
    pub github_api_requests_total: Counter,
    pub github_api_request_duration: Histogram,
    pub github_api_rate_limit_remaining: Gauge,
    pub mcp_commands_total: Counter,
    pub mcp_command_duration: Histogram,
    pub active_connections: Gauge,
    pub database_connections: Gauge,
}

impl Metrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Arc::new(Registry::new());

        // HTTP metrics
        let http_requests_total = Counter::with_opts(Opts::new(
            "http_requests_total",
            "Total number of HTTP requests"
        ).const_labels([("service", "github-mcp-server")].iter().cloned().collect()))?;

        let http_request_duration = Histogram::with_opts(HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds"
        ).const_labels([("service", "github-mcp-server")].iter().cloned().collect()))?;

        // GitHub API metrics
        let github_api_requests_total = Counter::with_opts(Opts::new(
            "github_api_requests_total",
            "Total number of GitHub API requests"
        ))?;

        let github_api_request_duration = Histogram::with_opts(HistogramOpts::new(
            "github_api_request_duration_seconds",
            "GitHub API request duration in seconds"
        ))?;

        let github_api_rate_limit_remaining = Gauge::with_opts(Opts::new(
            "github_api_rate_limit_remaining",
            "GitHub API rate limit remaining"
        ))?;

        // MCP command metrics
        let mcp_commands_total = Counter::with_opts(Opts::new(
            "mcp_commands_total",
            "Total number of MCP commands executed"
        ))?;

        let mcp_command_duration = Histogram::with_opts(HistogramOpts::new(
            "mcp_command_duration_seconds",
            "MCP command execution duration in seconds"
        ))?;

        // Connection metrics
        let active_connections = Gauge::with_opts(Opts::new(
            "active_connections",
            "Number of active connections"
        ))?;

        let database_connections = Gauge::with_opts(Opts::new(
            "database_connections",
            "Number of active database connections"
        ))?;

        // Register all metrics
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration.clone()))?;
        registry.register(Box::new(github_api_requests_total.clone()))?;
        registry.register(Box::new(github_api_request_duration.clone()))?;
        registry.register(Box::new(github_api_rate_limit_remaining.clone()))?;
        registry.register(Box::new(mcp_commands_total.clone()))?;
        registry.register(Box::new(mcp_command_duration.clone()))?;
        registry.register(Box::new(active_connections.clone()))?;
        registry.register(Box::new(database_connections.clone()))?;

        Ok(Metrics {
            registry,
            http_requests_total,
            http_request_duration,
            github_api_requests_total,
            github_api_request_duration,
            github_api_rate_limit_remaining,
            mcp_commands_total,
            mcp_command_duration,
            active_connections,
            database_connections,
        })
    }

    pub fn record_http_request(&self, method: &str, path: &str, status_code: u16, duration: f64) {
        self.http_requests_total
            .with_label_values(&[method, path, &status_code.to_string()])
            .inc();
        self.http_request_duration.observe(duration);
    }

    pub fn record_github_api_request(&self, endpoint: &str, method: &str, duration: f64) {
        self.github_api_requests_total
            .with_label_values(&[endpoint, method])
            .inc();
        self.github_api_request_duration.observe(duration);
    }

    pub fn update_github_rate_limit(&self, remaining: f64) {
        self.github_api_rate_limit_remaining.set(remaining);
    }

    pub fn record_mcp_command(&self, command: &str, status: &str, duration: f64) {
        self.mcp_commands_total
            .with_label_values(&[command, status])
            .inc();
        self.mcp_command_duration.observe(duration);
    }

    pub fn set_active_connections(&self, count: f64) {
        self.active_connections.set(count);
    }

    pub fn set_database_connections(&self, count: f64) {
        self.database_connections.set(count);
    }
}

pub async fn metrics_handler(State(metrics): State<Arc<Metrics>>) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = metrics.registry.gather();
    
    match encoder.encode_to_string(&metric_families) {
        Ok(output) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, encoder.format_type())
            .body(output)
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("Failed to encode metrics: {}", e))
            .unwrap(),
    }
}