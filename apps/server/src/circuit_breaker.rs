use axum::{
    body::Body,
    http::Request,
    response::{Response, IntoResponse},
    BoxError,
};
use failsafe::{CircuitBreaker, Config, Error as FailsafeError, StateMachine};
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Circuit Breaker Layer for Axum/Tower
#[derive(Clone)]
pub struct CircuitBreakerLayer {
    state_machine: failsafe::SharedStateMachine<failsafe::backoff::Exponential>,
}

impl CircuitBreakerLayer {
    pub fn new() -> Self {
        let backoff = failsafe::backoff::Exponential::new(std::time::Duration::from_secs(10));
        let config = Config::new()
            .failure_rate_threshold(0.5)
            .minimum_requests(5);

        let state_machine = StateMachine::new(backoff, config);
        
        Self {
            state_machine: failsafe::SharedStateMachine::new(state_machine),
        }
    }
}

impl<S> Layer<S> for CircuitBreakerLayer {
    type Service = CircuitBreakerService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CircuitBreakerService {
            inner,
            state_machine: self.state_machine.clone(),
        }
    }
}

/// Circuit Breaker Service
#[derive(Clone)]
pub struct CircuitBreakerService<S> {
    inner: S,
    state_machine: failsafe::SharedStateMachine<failsafe::backoff::Exponential>,
}

impl<S> Service<Request<Body>> for CircuitBreakerService<S>
where
    S: Service<Request<Body>, Response = Response, Error = BoxError> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = BoxError;
    // Explicit fully qualified type
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        if let Err(_) = self.state_machine.is_call_permitted() {
            let error = axum::http::StatusCode::SERVICE_UNAVAILABLE;
            return Box::pin(async move {
                let mut resp = Response::new(Body::from("Circuit Breaker Open"));
                *resp.status_mut() = error;
                Ok(resp)
            });
        }

        let future = self.inner.call(req);
        let state_machine = self.state_machine.clone();

        Box::pin(async move {
            match future.await {
                Ok(response) => {
                    if response.status().is_server_error() {
                        state_machine.on_failure();
                    } else {
                        state_machine.on_success();
                    }
                    Ok(response)
                }
                Err(e) => {
                    state_machine.on_failure();
                    Err(e)
                }
            }
        })
    }
}
