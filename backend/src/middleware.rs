use std::net::IpAddr;
use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

#[derive(Clone)]
pub struct IpFilter {
    allowed: Arc<Vec<IpAddr>>,
}

impl IpFilter {
    pub fn new(allowed: Vec<IpAddr>) -> Self {
        Self {
            allowed: Arc::new(allowed),
        }
    }

    pub fn is_allowed(&self, ip: IpAddr) -> bool {
        if self.allowed.is_empty() {
            return true;
        }

        // Always allow localhost
        if ip.is_loopback() {
            return true;
        }

        self.allowed.contains(&ip)
    }
}

pub async fn ip_filter_middleware(
    State(filter): State<IpFilter>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if filter.is_allowed(addr.ip()) {
        Ok(next.run(request).await)
    } else {
        tracing::warn!("Blocked request from {}", addr.ip());
        Err(StatusCode::FORBIDDEN)
    }
}
