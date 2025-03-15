use crate::config::{CircuitBreakerConfig, ExecConfig, RetryConfig};
use async_std::future::timeout;
use async_std::task::sleep;
use log::{debug, error, info, warn};
use std::error::Error;
use std::time::Instant;

/// Retries a given asynchronous operation based on the specified retry configuration.
///
/// # Arguments
/// * `operation` - A closure that returns a `Future` resolving to a `Result<T, E>`. The function will retry this operation if it fails.
/// * `retry_config` - A reference to `RetryConfig` specifying the maximum attempts and delay between retries.
///
/// # Returns
/// * `Ok(T)` if the operation succeeds within the allowed attempts.
/// * `Err(E)` if the operation fails after all retry attempts.
///
/// # Example
/// ```rust
/// use async_std::io::{ReadExt, WriteExt};
/// use async_std::net::TcpStream;
/// use async_std::task;
/// use resilient_rs::asynchronous::retry;
/// use resilient_rs::config::RetryConfig;
/// use std::io;
///
/// async fn fetch_url() -> Result<String, io::Error> {
///     let mut stream = TcpStream::connect("example.com:80").await?;
///     let request = "GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n";
///     stream.write_all(request.as_bytes()).await?;
///     stream.flush().await?;
///     let mut buffer = Vec::new();
///     stream.read_to_end(&mut buffer).await?;
///     let response = String::from_utf8_lossy(&buffer);
///     let is_success = response.starts_with("HTTP/1.1 200 OK");
///     Ok(is_success.to_string())
/// }
///
/// fn main() {
///     let retry_config = RetryConfig::default();
///     let result = task::block_on(async { retry(fetch_url, &retry_config).await });
///     match result {
///         Ok(output) => println!("Operation succeeded: {}", output),
///         Err(err) => println!("Operation failed: {}", err),
///     }
/// }
/// ```
///
/// # Notes
/// - The function logs warnings for failed attempts and final failure.
pub async fn retry<F, Fut, T, E>(mut operation: F, retry_config: &RetryConfig<E>) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut attempts = 0;
    let mut delay = retry_config.delay;

    loop {
        match operation().await {
            Ok(output) => {
                info!("Operation succeeded after {} attempts", attempts + 1);
                return Ok(output);
            }
            Err(err) if attempts + 1 < retry_config.max_attempts => {
                let should_retry = retry_config.retry_condition.map_or(true, |f| f(&err));
                if should_retry {
                    warn!(
                        "Operation failed (attempt {}/{}), retrying after {:?}...",
                        attempts + 1,
                        retry_config.max_attempts,
                        delay
                    );
                    sleep(delay).await;
                    delay = retry_config.strategy.calculate_delay(delay, attempts + 1);
                } else {
                    warn!(
                        "Operation failed (attempt {}/{}), not retryable, giving up.",
                        attempts + 1,
                        retry_config.max_attempts
                    );
                    return Err(err);
                }
            }
            Err(err) => {
                warn!(
                    "Operation failed after {} attempts, giving up.",
                    attempts + 1
                );
                return Err(err);
            }
        }

        attempts += 1;
    }
}

#[deprecated(
    since = "0.4.7",
    note = "use `retry` with `ExponentialBackoff` this will be removed in upcoming versions"
)]
pub async fn retry_with_exponential_backoff<F, Fut, T, E>(
    mut operation: F,
    retry_config: &RetryConfig<E>,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut attempts = 0;
    let mut delay = retry_config.delay;

    loop {
        match operation().await {
            Ok(output) => {
                info!("Operation succeeded after {} attempts", attempts + 1);
                return Ok(output);
            }
            Err(err) if attempts + 1 < retry_config.max_attempts => {
                let should_retry = retry_config.retry_condition.map_or(true, |f| f(&err));
                if should_retry {
                    warn!(
                        "Operation failed (attempt {}/{}), retrying after {:?}...",
                        attempts + 1,
                        retry_config.max_attempts,
                        delay
                    );
                    sleep(delay).await;
                    delay *= 2;
                } else {
                    warn!(
                        "Operation failed (attempt {}/{}), not retryable, giving up.",
                        attempts + 1,
                        retry_config.max_attempts
                    );
                    return Err(err);
                }
            }
            Err(err) => {
                warn!(
                    "Operation failed after {} attempts, giving up.",
                    attempts + 1
                );
                return Err(err);
            }
        }

        attempts += 1;
    }
}

/// Executes an asynchronous operation with a timeout and an optional fallback.
///
/// This function runs the provided `operation` future with a specified timeout duration.
/// If the operation completes within the timeout, its result is returned. If it times out,
/// a fallback function (if provided) is executed synchronously to produce a result.
///
/// # Type Parameters
///
/// * `T` - The type of the successful result returned by the operation or fallback.
///
/// # Arguments
///
/// * `operation` - An asynchronous operation that returns a `Result<T, Box<dyn Error>>`.
///                 This is typically an async block or function that performs the primary task.
/// * `exec_config` - A reference to an `ExecConfig<T>` containing the timeout duration and
///                   an optional fallback function.
///
/// # Returns
///
/// * `Ok(T)` - If the operation completes successfully within the timeout, or if the
///             fallback succeeds after a timeout.
/// * `Err(Box<dyn Error>)` - If the operation times out and no fallback is provided,
///                           or if the fallback itself fails.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
/// use async_std::task::{sleep, block_on};
/// use resilient_rs::asynchronous::execute_with_fallback;
/// use resilient_rs::config::ExecConfig;
///
/// fn main() {
/// let config = ExecConfig {
///         timeout_duration: Duration::from_millis(50),
///         fallback: Some(|| Ok("fallback result".to_string())),
///     };
///
///     let operation = async {
///         sleep(Duration::from_millis(100)).await;
///         Ok("success".to_string())
///     };
///
///     let result = block_on(async { execute_with_fallback(operation, &config).await } );
///     assert_eq!(result.unwrap(), "fallback result");
/// }
/// ```
pub async fn execute_with_fallback<T>(
    operation: impl Future<Output = Result<T, Box<dyn Error>>>,
    exec_config: &ExecConfig<T>,
) -> Result<T, Box<dyn Error>> {
    match timeout(exec_config.timeout_duration, operation).await {
        Ok(result) => {
            info!("Operation completed before timeout; returning result.");
            result
        }
        Err(e) => {
            if let Some(fallback) = exec_config.fallback {
                warn!("Operation timed out; executing fallback.");
                fallback()
            } else {
                error!("Operation timed out; no fallback provided, returning error.");
                Err(Box::new(e))
            }
        }
    }
}

/// Represents the possible states of a circuit breaker.
///
/// A circuit breaker can be in one of three states, which determine how it handles operations:
/// - `Close`: Operations are allowed to proceed normally.
/// - `Open`: Operations are blocked due to repeated failures, preventing further attempts until a cooldown period elapses.
/// - `HalfOpen`: A trial state after the cooldown, where operations are tentatively allowed to test if the system has recovered.
///
/// This enum is used internally by the `CircuitBreaker` struct to manage its state machine.
#[derive(Debug, PartialEq)]
enum CircuitBreakerState {
    Close,
    Open,
    HalfOpen,
}

/// A circuit breaker for managing fault tolerance in systems.
///
/// The `CircuitBreaker` struct implements the circuit breaker pattern to prevent cascading failures
/// by monitoring the success and failure of operations. It uses a provided `CircuitBreakerConfig`
/// to define thresholds and cooldown behavior, transitioning between states (`Closed`, `Open`, `HalfOpen`)
/// based on operation outcomes.
///
/// # Fields
/// * `config` - Configuration defining thresholds and cooldown period
/// * `state` - Current state of the circuit breaker (`Closed`, `Open`, or `HalfOpen`)
/// * `failure_count` - Number of consecutive failures since the last state change
/// * `success_count` - Number of consecutive successes in the `HalfOpen` state
/// * `last_failure_time` - Timestamp of the most recent failure (if any), used to enforce cooldown period
/// ```
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitBreakerState,
    failure_count: usize,
    success_count: usize,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    /// Creates a new `CircuitBreaker` instance with the given configuration.
    ///
    /// Initializes the circuit breaker in the `Close` state, ready to handle operations.
    ///
    /// # Parameters
    /// - `config`: A reference to a `CircuitBreakerConfig` defining the failure threshold,
    ///   success threshold, and cooldown period.
    ///
    /// # Returns
    /// A new `CircuitBreaker` instance configured with the provided `config`.
    ///
    /// # Examples
    /// ```rust
    /// use std::time::Duration;
    /// use resilient_rs::asynchronous::CircuitBreaker;
    /// use resilient_rs::config::CircuitBreakerConfig;
    ///
    /// let config = CircuitBreakerConfig::new(2, 3, Duration::from_secs(5));
    /// let cb = CircuitBreaker::new(config);
    /// ```
    pub fn new(config: CircuitBreakerConfig) -> Self {
        CircuitBreaker {
            config,
            state: CircuitBreakerState::Close,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
        }
    }

    /// Executes an operation under circuit breaker supervision.
    ///
    /// This method runs the provided async operation and updates the circuit breaker state based
    /// on the outcome. If the breaker is `Open` and the cooldown period hasn’t elapsed, it blocks
    /// the operation. In `HalfOpen`, it tests recovery, and in `Close`, it monitors for failures.
    ///
    /// # Parameters
    /// - `operation`: An async closure or function that returns a `Future` yielding a `Result`.
    ///   The closure must be `FnMut` to allow multiple calls if needed in the future.
    ///
    /// # Returns
    /// - `Ok(T)` if the operation succeeds, where `T` is the operation’s return type.
    /// - `Err(Box<dyn Error>)` if the operation fails or the breaker is `Open`.
    /// ```
    pub async fn run<F, Fut, T>(&mut self, mut operation: F) -> Result<T, Box<dyn Error>>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, Box<dyn Error>>>,
    {
        match self.state {
            CircuitBreakerState::Open => {
                if let Some(last_failure_time) = self.last_failure_time {
                    if last_failure_time.elapsed() >= self.config.cooldown_period {
                        self.state = CircuitBreakerState::HalfOpen;
                        self.success_count = 0;
                        warn!("Circuit Breaker transitioning to Half Open State");
                    } else {
                        warn!("Circuit Breaker is open.. Requests are blocked for now");
                        return Err(Box::from(String::from(
                            "Circuit Breaker is open. Please try later..!",
                        )));
                    }
                }
            }
            _ => {}
        }

        match operation().await {
            Ok(result) => {
                debug!("Request Success response");
                self.on_success();
                Ok(result)
            }
            Err(err) => {
                error!("Failed with {}", err);
                self.on_failure();
                Err(err)
            }
        }
    }

    /// Handles a successful operation outcome.
    ///
    /// Updates the circuit breaker state based on a successful operation:
    /// - In `HalfOpen`, increments `success_count` and transitions to `Close` if the success threshold is met.
    /// - In `Close`, resets `failure_count` to 0.
    /// - In `Open`, does nothing (this method is typically called only after `call`).
    fn on_success(&mut self) {
        match self.state {
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitBreakerState::Close;
                    self.failure_count = 0;
                    debug!("Circuit breaker transitioning to closed state");
                }
            }
            _ => {
                self.failure_count = 0;
            }
        }
    }

    /// Handles a failed operation outcome.
    ///
    /// Updates the circuit breaker state based on a failed operation:
    /// - Increments `failure_count`.
    /// - If `failure_count` exceeds the threshold, transitions to `Open` and records the failure time.
    fn on_failure(&mut self) {
        self.failure_count += 1;
        if self.failure_count >= self.config.failure_threshold {
            self.state = CircuitBreakerState::Open;
            self.last_failure_time = Some(Instant::now());
            error!("Circuit Breaker transitioning to open state");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task::{block_on, sleep};
    use std::error::Error;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[derive(Debug, PartialEq, Eq)]
    struct DummyError(&'static str);

    impl std::fmt::Display for DummyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl Error for DummyError {}

    // Suite for `retry` function
    mod retry_tests {
        use super::*;
        use crate::strategies::RetryStrategy::Linear;

        #[test]
        fn test_retry_success_first_try_with_block_on() {
            let config = RetryConfig {
                max_attempts: 3,
                delay: Duration::from_millis(10),
                retry_condition: None,
                strategy: Linear,
            };

            let attempts = Arc::new(Mutex::new(0));
            let op_attempts = attempts.clone();
            let operation = move || {
                let op_attempts = op_attempts.clone();
                async move {
                    let mut count = op_attempts.lock().unwrap();
                    *count += 1;
                    Ok::<_, DummyError>("success")
                }
            };

            let result = block_on(retry(operation, &config));
            assert_eq!(result, Ok("success"));
            assert_eq!(*attempts.lock().unwrap(), 1);
        }

        #[test]
        fn test_retry_success_after_failures() {
            let config = RetryConfig {
                max_attempts: 5,
                delay: Duration::from_millis(10),
                retry_condition: None,
                strategy: Linear,
            };

            let attempts = Arc::new(Mutex::new(0));
            let op_attempts = attempts.clone();
            let operation = move || {
                let op_attempts = op_attempts.clone();
                async move {
                    let mut count = op_attempts.lock().unwrap();
                    *count += 1;
                    if *count < 4 {
                        Err(DummyError("temporary failure"))
                    } else {
                        Ok("eventual success")
                    }
                }
            };

            let result = block_on(retry(operation, &config));
            assert_eq!(result, Ok("eventual success"));
            assert_eq!(*attempts.lock().unwrap(), 4);
        }

        #[test]
        fn test_retry_failure_all_attempts() {
            let config = RetryConfig {
                max_attempts: 3,
                delay: Duration::from_millis(10),
                retry_condition: None,
                strategy: Linear,
            };

            let attempts = Arc::new(Mutex::new(0));
            let op_attempts = attempts.clone();
            let operation = move || {
                let op_attempts = op_attempts.clone();
                async move {
                    let mut count = op_attempts.lock().unwrap();
                    *count += 1;
                    Err(DummyError("permanent failure"))
                }
            };

            let result: Result<(), DummyError> = block_on(retry(operation, &config));
            assert_eq!(result, Err(DummyError("permanent failure")));
            assert_eq!(*attempts.lock().unwrap(), config.max_attempts);
        }

        #[test]
        fn test_retry_fail_first_try_retry_condition_un_match() {
            let config = RetryConfig {
                max_attempts: 3,
                delay: Duration::from_millis(10),
                retry_condition: Some(|e: &DummyError| e.0.contains("transient")),
                strategy: Linear,
            };

            let attempts = Arc::new(Mutex::new(0));
            let op_attempts = attempts.clone();
            let operation = move || {
                let op_attempts = op_attempts.clone();
                async move {
                    let mut count = op_attempts.lock().unwrap();
                    *count += 1;
                    Err(DummyError("always fail"))
                }
            };

            let result: Result<(), DummyError> = block_on(retry(operation, &config));
            assert_eq!(result, Err(DummyError("always fail")));
            assert_eq!(*attempts.lock().unwrap(), 1);
        }

        #[test]
        fn test_retry_fail_first_try_retry_condition_match() {
            let config = RetryConfig {
                max_attempts: 3,
                delay: Duration::from_millis(10),
                retry_condition: Some(|e: &DummyError| e.0.contains("transient")),
                strategy: Linear,
            };

            let attempts = Arc::new(Mutex::new(0));
            let op_attempts = attempts.clone();
            let operation = move || {
                let op_attempts = op_attempts.clone();
                async move {
                    let mut count = op_attempts.lock().unwrap();
                    *count += 1;
                    Err(DummyError("transient"))
                }
            };

            let result: Result<(), DummyError> = block_on(retry(operation, &config));
            assert_eq!(result, Err(DummyError("transient")));
            assert_eq!(*attempts.lock().unwrap(), 3);
        }
    }

    // Suite for `retry_with_exponential_backoff` function
    mod retry_with_exponential_backoff_tests {
        use super::*;
        use crate::strategies::RetryStrategy::ExponentialBackoff;

        #[test]
        fn test_retry_with_exponential_backoff_success_first_try() {
            let config = RetryConfig {
                max_attempts: 3,
                delay: Duration::from_millis(10),
                retry_condition: None,
                strategy: ExponentialBackoff,
            };

            let attempts = Arc::new(Mutex::new(0));
            let op_attempts = attempts.clone();
            let operation = move || {
                let op_attempts = op_attempts.clone();
                async move {
                    let mut count = op_attempts.lock().unwrap();
                    *count += 1;
                    Ok::<_, DummyError>("successful")
                }
            };

            let result = block_on(retry(operation, &config));
            assert_eq!(result, Ok("successful"));
            assert_eq!(*attempts.lock().unwrap(), 1);
        }

        #[test]
        fn test_retry_with_exponential_backoff_success_after_failures() {
            let config = RetryConfig {
                max_attempts: 5,
                delay: Duration::from_millis(10),
                retry_condition: None,
                strategy: ExponentialBackoff,
            };

            let attempts = Arc::new(Mutex::new(0));
            let op_attempts = attempts.clone();
            let operation = move || {
                let op_attempts = op_attempts.clone();
                async move {
                    let mut count = op_attempts.lock().unwrap();
                    *count += 1;
                    if *count < 4 {
                        Err(DummyError("temporary fail"))
                    } else {
                        Ok("eventual success")
                    }
                }
            };

            let result = block_on(retry(operation, &config));
            assert_eq!(result, Ok("eventual success"));
            assert_eq!(*attempts.lock().unwrap(), 4);
        }

        #[test]
        fn test_retry_with_exponential_backoff_failure_all_attempts() {
            let config = RetryConfig {
                max_attempts: 3,
                delay: Duration::from_millis(10),
                retry_condition: None,
                strategy: ExponentialBackoff,
            };

            let attempts = Arc::new(Mutex::new(0));
            let op_attempts = attempts.clone();
            let operation = move || {
                let op_attempts = op_attempts.clone();
                async move {
                    let mut count = op_attempts.lock().unwrap();
                    *count += 1;
                    Err(DummyError("always fail"))
                }
            };

            let result: Result<(), DummyError> = block_on(retry(operation, &config));
            assert_eq!(result, Err(DummyError("always fail")));
            assert_eq!(*attempts.lock().unwrap(), config.max_attempts);
        }

        #[test]
        fn test_retry_with_exponential_backoff_success_after_failures_with_condition() {
            let config = RetryConfig {
                max_attempts: 5,
                delay: Duration::from_millis(10),
                retry_condition: Some(|e: &DummyError| e.0.contains("405")),
                strategy: ExponentialBackoff,
            };

            let attempts = Arc::new(Mutex::new(0));
            let op_attempts = attempts.clone();
            let operation = move || {
                let op_attempts = op_attempts.clone();
                async move {
                    let mut count = op_attempts.lock().unwrap();
                    *count += 1;
                    if *count < 2 {
                        Err(DummyError("temporary fail"))
                    } else {
                        Ok("eventual success")
                    }
                }
            };

            let result = block_on(retry(operation, &config));
            assert_eq!(result, Err(DummyError("temporary fail")));
            assert_eq!(*attempts.lock().unwrap(), 1);
        }
    }

    // Suite for `execute_with_timeout` function
    mod execute_with_timeout_tests {
        use super::*;

        #[test]
        fn test_execute_with_timeout_success() {
            let config: ExecConfig<String> = ExecConfig {
                timeout_duration: Duration::from_millis(100),
                fallback: None,
            };

            let operation = || async { Ok("success".to_string()) };
            let result = block_on(execute_with_fallback(operation(), &config));
            assert_eq!(result.unwrap(), "success");
        }

        #[test]
        fn test_execute_with_timeout_immediate_failure() {
            let config: ExecConfig<String> = ExecConfig {
                timeout_duration: Duration::from_millis(100),
                fallback: None,
            };

            let operation =
                || async { Err(Box::new(DummyError("immediate failure")) as Box<dyn Error>) };
            let result = block_on(execute_with_fallback(operation(), &config));
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "immediate failure");
        }

        #[test]
        fn test_execute_with_timeout_timeout_no_fallback() {
            let config: ExecConfig<String> = ExecConfig {
                timeout_duration: Duration::from_millis(10),
                fallback: None,
            };

            let operation = || async {
                sleep(Duration::from_millis(50)).await;
                Ok("too slow".to_string())
            };
            let result = block_on(execute_with_fallback(operation(), &config));
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "future has timed out");
        }

        #[test]
        fn test_execute_with_timeout_timeout_with_fallback_success() {
            let mut config: ExecConfig<String> = ExecConfig::new(Duration::from_millis(10));
            config.with_fallback(|| Ok("fallback success".to_string()));

            let operation = || async {
                sleep(Duration::from_millis(50)).await;
                Ok("too slow".to_string())
            };
            let result = block_on(execute_with_fallback(operation(), &config));
            assert_eq!(result.unwrap(), "fallback success");
        }

        #[test]
        fn test_execute_with_timeout_timeout_with_fallback_failure() {
            let mut config: ExecConfig<String> = ExecConfig::new(Duration::from_millis(10));
            config.with_fallback(|| Err(Box::new(DummyError("fallback failed")) as Box<dyn Error>));

            let operation = || async {
                sleep(Duration::from_millis(50)).await;
                Ok("too slow".to_string())
            };
            let result = block_on(execute_with_fallback(operation(), &config));
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "fallback failed");
        }

        #[test]
        fn test_execute_with_timeout_success_near_timeout() {
            let config: ExecConfig<String> = ExecConfig {
                timeout_duration: Duration::from_millis(50),
                fallback: None,
            };

            let operation = || async {
                sleep(Duration::from_millis(40)).await;
                Ok("just in time".to_string())
            };
            let result = block_on(execute_with_fallback(operation(), &config));
            assert_eq!(result.unwrap(), "just in time");
        }
    }

    mod circuit_breaker_tests {
        use super::*;

        #[test]
        fn test_success_keeps_closed() {
            let config = CircuitBreakerConfig::new(2, 3, Duration::from_secs(1));
            let mut cb = CircuitBreaker::new(config);
            let result = block_on(async {
                cb.run(|| async { Ok::<_, Box<dyn Error>>("Success") })
                    .await
            });
            assert!(result.is_ok());
            assert_eq!(cb.state, CircuitBreakerState::Close);
            assert_eq!(cb.failure_count, 0);
        }

        #[test]
        fn test_half_open_to_close() {
            let config = CircuitBreakerConfig::new(2, 3, Duration::from_millis(100));
            let mut cb = CircuitBreaker::new(config);
            // Trigger Open state
            for _ in 0..3 {
                let _ =
                    block_on(async { cb.run(|| async { Err::<(), _>(Box::from("Fail")) }).await });
            }
            assert_eq!(cb.state, CircuitBreakerState::Open);
            // Wait for cooldown
            block_on(sleep(Duration::from_millis(150)));
            // Transition to HalfOpen and succeed twice
            for _ in 0..2 {
                let result = block_on(async {
                    cb.run(|| async { Ok::<_, Box<dyn Error>>("Success") })
                        .await
                });
                assert!(result.is_ok());
            }
            assert_eq!(cb.state, CircuitBreakerState::Close);
            assert_eq!(cb.success_count, 2);
        }
    }
}
