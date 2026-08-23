use std::sync::Arc;

use axum::{
    Json, Router,
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::{DebugMcpServer, DebugRuntime};
use crate::config::RuntimeConfig;
use crate::error::SidecarError;
use crate::security::{is_loopback_host, is_loopback_origin};

/// Runs stateful Streamable HTTP on the validated loopback address.
pub async fn serve_http(
    config: RuntimeConfig,
    runtime: DebugRuntime,
    shutdown: CancellationToken,
) -> Result<(), SidecarError> {
    let service_config =
        StreamableHttpServerConfig::default().with_cancellation_token(shutdown.child_token());
    let service = StreamableHttpService::new(
        move || Ok(DebugMcpServer::new(runtime.clone())),
        Arc::new(LocalSessionManager::default()),
        service_config,
    );
    let router = protected_router(Router::new().nest_service("/mcp", service), config.port);
    let address = if config.host == "::1" {
        format!("[::1]:{}", config.port)
    } else {
        format!("{}:{}", config.host, config.port)
    };
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .map_err(|error| SidecarError::Runtime {
            message: format!("failed to bind {address}: {error}"),
        })?;
    tracing::info!(event = "mcp.listening", address, endpoint = "/mcp");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown.cancelled_owned())
        .await
        .map_err(|error| SidecarError::Runtime {
            message: error.to_string(),
        })
}

/// Applies Host, Origin, and expected-port validation before MCP routing.
fn protected_router(router: Router, port: u16) -> Router {
    router.layer(from_fn_with_state(port, validate_request))
}

/// Rejects DNS-rebinding headers with a JSON-RPC-shaped response.
async fn validate_request(State(port): State<u16>, request: Request<Body>, next: Next) -> Response {
    let headers = request.headers();
    let host = header(headers, "host");
    let origin = header(headers, "origin");
    if !is_loopback_host(host, port) || !is_loopback_origin(origin) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "jsonrpc": "2.0",
                "error": { "code": -32000, "message": "Forbidden: loopback Host, Origin, and expected port are required" },
                "id": null
            })),
        )
            .into_response();
    }
    next.run(request).await
}

/// Reads one UTF-8 request header.
fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

#[cfg(test)]
mod tests {
    use axum::{Router, body::Body, http::Request, routing::get};
    use tower::ServiceExt;

    use super::protected_router;

    /// Rejects hostile hosts before reaching an MCP route.
    #[tokio::test]
    async fn rejects_hostile_host_at_http_boundary() {
        let router = protected_router(Router::new().route("/mcp", get(|| async { "ok" })), 3001);
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/mcp")
                    .header("host", "attacker.example:3001")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), 403);
    }

    /// Accepts a loopback host on the expected port.
    #[tokio::test]
    async fn accepts_safe_http_boundary() {
        let router = protected_router(Router::new().route("/mcp", get(|| async { "ok" })), 3001);
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/mcp")
                    .header("host", "localhost:3001")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), 200);
    }
}
