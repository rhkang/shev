use std::net::IpAddr;
use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

#[derive(Clone)]
pub struct IpFilter {
    allowed_read: Arc<Vec<IpAddr>>,
    allowed_write: Arc<Vec<IpAddr>>,
}

impl IpFilter {
    pub fn new(allowed_read: Vec<IpAddr>, allowed_write: Vec<IpAddr>) -> Self {
        Self {
            allowed_read: Arc::new(allowed_read),
            allowed_write: Arc::new(allowed_write),
        }
    }

    fn is_write_method(method: &Method) -> bool {
        matches!(method, &Method::POST | &Method::PUT | &Method::DELETE)
    }

    pub fn is_allowed(&self, ip: IpAddr, method: &Method) -> bool {
        if ip.is_loopback() {
            return true;
        }

        if self.allowed_write.contains(&ip) {
            return true;
        }

        if Self::is_write_method(method) {
            false
        } else {
            self.allowed_read.is_empty() || self.allowed_read.contains(&ip)
        }
    }
}

pub async fn ip_filter_middleware(
    State(filter): State<IpFilter>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    if filter.is_allowed(addr.ip(), &method) {
        Ok(next.run(request).await)
    } else {
        tracing::warn!("Blocked {} request from {}", method, addr.ip());
        Err(StatusCode::FORBIDDEN)
    }
}
