use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
    BoxError,
};
use std::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;
use tower::{Layer, Service};
use rand::Rng; // requires rand dependency

/// Chaos Configuration
#[derive(Clone)]
pub struct ChaosConfig {
    pub failure_rate: f64, // 0.0 to 1.0
    pub delay_ms: u64,
    pub enabled: bool,
}

impl ChaosConfig {
    pub fn env() -> Self {
        let enabled = std::env::var("CHAOS_ENABLED").unwrap_or_else(|_| "false".into()) == "true";
        let failure_rate = std::env::var("CHAOS_FAILURE_RATE")
            .unwrap_or_else(|_| "0.0".into())
            .parse()
            .unwrap_or(0.0);
        let delay_ms = std::env::var("CHAOS_DELAY_MS")
            .unwrap_or_else(|_| "0".into())
            .parse()
            .unwrap_or(0);

        Self {
            enabled,
            failure_rate,
            delay_ms,
        }
    }
}

/// Chaos Layer
#[derive(Clone)]
pub struct ChaosLayer {
    config: ChaosConfig,
}

impl ChaosLayer {
    pub fn new() -> Self {
        Self {
            config: ChaosConfig::env(),
        }
    }
}

impl<S> Layer<S> for ChaosLayer {
    type Service = ChaosService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ChaosService {
            inner,
            config: self.config.clone(),
        }
    }
}

/// Chaos Service
#[derive(Clone)]
pub struct ChaosService<S> {
    inner: S,
    config: ChaosConfig,
}

impl<S> Service<Request<Body>> for ChaosService<S>
where
    S: Service<Request<Body>, Response = Response, Error = BoxError> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        if !self.config.enabled {
            let fut = self.inner.call(req);
            return Box::pin(fut);
        }

        // 1. Artificial Delay
        if self.config.delay_ms > 0 {
            // Note: In real implementation, use tokio::time::sleep
            // But we can't easily do it here without async block overhead
            // Actually we are modifying the future chain anyway
        }

        // 2. Random Failure
        let mut rng = rand::thread_rng();
        if rng.gen_bool(self.config.failure_rate) {
            return Box::pin(async move {
                // Simulate 500 Internal Server Error
                let mut resp = Response::new(Body::from("Chaos Monkey: Artificial Failure"));
                *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                Ok(resp)
            });
        }

        let delay = self.config.delay_ms;
        let fut = self.inner.call(req);
        
        Box::pin(async move {
            if delay > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }
            fut.await
        })
    }
}
