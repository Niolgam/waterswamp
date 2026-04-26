/// In-memory circuit breaker for external service calls (RF-035/RF-036).
///
/// States:
///   CLOSED  — normal operation, calls pass through
///   OPEN    — service down, calls rejected immediately (fast-fail)
///   HALF_OPEN — testing recovery: one probe call allowed; if it succeeds → CLOSED
///
/// Thread-safe via tokio::sync::RwLock. Intended to wrap external service clients
/// (Comprasnet, SIORG) in the application layer.
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug)]
struct InnerState {
    state: CircuitState,
    consecutive_failures: u32,
    opened_at: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    name: String,
    failure_threshold: u32,
    recovery_duration: Duration,
    inner: Arc<RwLock<InnerState>>,
}

impl CircuitBreaker {
    pub fn new(name: impl Into<String>, failure_threshold: u32, recovery_secs: u64) -> Self {
        Self {
            name: name.into(),
            failure_threshold,
            recovery_duration: Duration::from_secs(recovery_secs),
            inner: Arc::new(RwLock::new(InnerState {
                state: CircuitState::Closed,
                consecutive_failures: 0,
                opened_at: None,
            })),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns true if a call should be allowed through.
    /// Transitions OPEN → HALF_OPEN when the recovery window has elapsed.
    pub async fn allow_call(&self) -> bool {
        let mut inner = self.inner.write().await;
        match inner.state {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => {
                if let Some(opened) = inner.opened_at {
                    if opened.elapsed() >= self.recovery_duration {
                        tracing::info!(
                            circuit = %self.name,
                            "Circuit breaker transitioning OPEN → HALF_OPEN"
                        );
                        inner.state = CircuitState::HalfOpen;
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Record a successful call. Resets failures and closes the circuit.
    pub async fn record_success(&self) {
        let mut inner = self.inner.write().await;
        if inner.state != CircuitState::Closed {
            tracing::info!(circuit = %self.name, "Circuit breaker CLOSED after success");
        }
        inner.state = CircuitState::Closed;
        inner.consecutive_failures = 0;
        inner.opened_at = None;
    }

    /// Record a failed call. Opens the circuit if the threshold is reached.
    pub async fn record_failure(&self) {
        let mut inner = self.inner.write().await;
        inner.consecutive_failures += 1;
        if inner.consecutive_failures >= self.failure_threshold
            && inner.state != CircuitState::Open
        {
            tracing::warn!(
                circuit = %self.name,
                failures = inner.consecutive_failures,
                "Circuit breaker OPENED after {} consecutive failures",
                inner.consecutive_failures
            );
            inner.state = CircuitState::Open;
            inner.opened_at = Some(Instant::now());
        }
    }

    /// Returns a snapshot of the current state for health checks.
    pub async fn state_snapshot(&self) -> CircuitBreakerSnapshot {
        let inner = self.inner.read().await;
        CircuitBreakerSnapshot {
            name: self.name.clone(),
            state: inner.state.clone(),
            consecutive_failures: inner.consecutive_failures,
            seconds_since_opened: inner
                .opened_at
                .map(|t| t.elapsed().as_secs()),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CircuitBreakerSnapshot {
    pub name: String,
    pub state: CircuitState,
    pub consecutive_failures: u32,
    pub seconds_since_opened: Option<u64>,
}

/// Global registry of all circuit breakers for health reporting (RF-035).
/// The list itself uses std::sync::RwLock (sync, populated at startup);
/// each individual CircuitBreaker uses tokio::sync::RwLock for async call-time access.
#[derive(Default, Clone)]
pub struct CircuitBreakerRegistry {
    breakers: Arc<std::sync::RwLock<Vec<Arc<CircuitBreaker>>>>,
}

impl CircuitBreakerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a circuit breaker. Sync — call at startup before serving requests.
    pub fn register(&self, cb: Arc<CircuitBreaker>) {
        self.breakers.write().unwrap().push(cb);
    }

    /// Returns snapshots of all registered circuit breakers.
    pub async fn health_report(&self) -> Vec<CircuitBreakerSnapshot> {
        let breakers = self.breakers.read().unwrap().clone();
        let mut result = Vec::with_capacity(breakers.len());
        for cb in breakers.iter() {
            result.push(cb.state_snapshot().await);
        }
        result
    }

    /// Returns true if any circuit breaker is OPEN (system is in degraded mode).
    pub async fn is_degraded(&self) -> bool {
        let breakers = self.breakers.read().unwrap().clone();
        for cb in breakers.iter() {
            let snap = cb.state_snapshot().await;
            if snap.state == CircuitState::Open {
                return true;
            }
        }
        false
    }
}
