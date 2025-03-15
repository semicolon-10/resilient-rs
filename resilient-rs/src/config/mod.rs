use crate::strategies::RetryStrategy;
use std::error::Error;
use std::time::Duration;

#[derive(Debug)]
pub struct RetryConfig<E> {
    /// The maximum number of retry attempts.
    ///
    /// This specifies how many times the operation will be retried before
    /// giving up. For example, if `max_attempts` is set to 3, the operation
    /// will be attempted up to 3 times (1 initial attempt + 2 retries).
    pub max_attempts: usize,

    /// The delay between retry attempts.
    ///
    /// This specifies the base amount of time to wait between each retry attempt.
    /// The actual delay may vary depending on the `strategy`. For example, if
    /// `delay` is set to `Duration::from_secs(2)` and the strategy is `Linear`,
    /// the program will wait 2 seconds between retries.
    pub delay: Duration,

    /// The strategy used to calculate delays between retry attempts.
    ///
    /// This field determines how the `delay` is applied:
    /// - `Linear`: Uses a fixed delay between retries.
    /// - `ExponentialBackoff`: Increases the delay exponentially with each retry.
    pub strategy: RetryStrategy,

    /// An optional function to determine if a retry should be attempted.
    ///
    /// This field allows you to specify a custom condition for retrying based on the error type `E`.
    /// It takes a reference to the error (`&E`) and returns a `bool`:
    /// - `true` if the operation should be retried.
    /// - `false` if the operation should not be retried, causing it to fail immediately.
    ///
    /// If set to `None` (the default), all errors will trigger a retry up to `max_attempts`.
    /// If set to `Some(fn)`, only errors for which the function returns `true` will be retried.
    /// In this example, only errors containing the word "transient" will trigger retries.
    pub retry_condition: Option<fn(&E) -> bool>,
}

impl<E> Default for RetryConfig<E> {
    /// Provides a default configuration for retrying operations.
    ///
    /// The default configuration includes:
    /// - `max_attempts`: 3 retries
    /// - `delay`: 2 seconds between retries
    /// - `strategy`: `Linear`
    /// - `retry_condition`: `None`, meaning all errors trigger retries
    ///
    /// This implementation allows you to create a `RetryConfig` with sensible
    /// defaults using `RetryConfig::default()`.
    fn default() -> Self {
        RetryConfig {
            max_attempts: 3,
            delay: Duration::from_secs(2),
            strategy: RetryStrategy::Linear,
            retry_condition: None,
        }
    }
}

impl<E> RetryConfig<E> {
    /// Creates a new `RetryConfig` with the specified maximum attempts, delay, and strategy.
    ///
    /// This constructor initializes a `RetryConfig` with the given `max_attempts`, `delay`,
    /// and `strategy`, setting `retry_condition` to `None`. When `retry_condition` is `None`,
    /// all errors will trigger retries up to the specified `max_attempts`.
    ///
    /// # Arguments
    /// * `max_attempts` - The maximum number of attempts (including the initial attempt).
    /// * `delay` - The base duration to wait between retry attempts.
    /// * `strategy` - The retry strategy to use (`Linear` or `ExponentialBackoff`).
    ///
    /// # Returns
    /// A new `RetryConfig` instance with the provided settings and no retry condition.
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    /// use resilient_rs::config::{RetryConfig, RetryStrategy};
    /// let config = RetryConfig::new(3, Duration::from_secs(1), RetryStrategy::Linear);
    /// ```
    pub fn new(max_attempts: usize, delay: Duration, strategy: RetryStrategy) -> Self {
        RetryConfig {
            max_attempts,
            delay,
            strategy,
            retry_condition: None,
        }
    }

    /// Sets a custom retry condition and returns the modified `RetryConfig`.
    ///
    /// This method allows you to specify a function that determines whether an operation should
    /// be retried based on the error. It takes ownership of the `RetryConfig`, sets the
    /// `retry_condition` field to the provided function, and returns the updated instance.
    /// This enables method chaining in a builder-like pattern.
    ///
    /// # Arguments
    /// * `retry_condition` - A function that takes a reference to an error (`&E`) and returns
    ///   `true` if the operation should be retried, or `false` if it should fail immediately.
    ///
    /// # Returns
    /// The updated `RetryConfig` with the specified retry condition.
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    /// use resilient_rs::config::{RetryConfig, RetryStrategy};
    /// let config = RetryConfig::new(3, Duration::from_secs(1), RetryStrategy::Linear)
    ///     .with_retry_condition(|e: &String| e.contains("transient"));
    /// ```
    pub fn with_retry_condition(mut self, retry_condition: fn(&E) -> bool) -> Self {
        self.retry_condition = Some(retry_condition);
        self
    }

    /// Sets a custom retry strategy and returns the modified `RetryConfig`.
    ///
    /// This method allows you to specify the retry strategy (`Linear` or `ExponentialBackoff`).
    /// It takes ownership of the `RetryConfig`, sets the `strategy` field to the provided value,
    /// and returns the updated instance. This enables method chaining in a builder-like pattern.
    ///
    /// # Arguments
    /// * `strategy` - The retry strategy to use (`Linear` or `ExponentialBackoff`).
    ///
    /// # Returns
    /// The updated `RetryConfig` with the specified retry strategy.
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    /// use resilient_rs::config::{RetryConfig, RetryStrategy};
    /// let config = RetryConfig::default()
    ///     .with_strategy(RetryStrategy::ExponentialBackoff);
    /// ```
    pub fn with_strategy(mut self, strategy: RetryStrategy) -> Self {
        self.strategy = strategy;
        self
    }
}

/// Configuration for executable tasks supporting both synchronous and asynchronous operations.
///
/// This struct defines execution parameters for tasks that may run either synchronously
/// or asynchronously, including a timeout duration and an optional fallback function.
/// It's designed to be passed to functions that handle task execution with support for
/// both execution models.
///
/// # Type Parameters
/// * `T` - The type of the successful result, must implement `Clone`
/// * `E` - The type of the error that may occur during execution
///
#[derive(Debug)]
pub struct ExecConfig<T> {
    /// The maximum duration allowed for task execution before timeout.
    ///
    /// This applies to both synchronous and asynchronous operations. For async operations,
    /// this typically integrates with runtime timeout mechanisms.
    pub timeout_duration: Duration,

    /// Optional fallback function to execute if the primary task fails or times out.
    ///
    /// The fallback must be a synchronous function that returns a `Result`. For async
    /// contexts, the execution function is responsible for handling the sync-to-async
    /// transition if needed.
    pub fallback: Option<fn() -> Result<T, Box<dyn Error>>>,
}

impl<T> ExecConfig<T>
where
    T: Clone,
{
    /// Creates a new execution configuration with the specified timeout duration.
    ///
    /// Initializes the configuration without a fallback function. This is suitable
    /// for both synchronous and asynchronous task execution scenarios.
    ///
    /// # Arguments
    /// * `timeout_duration` - Maximum execution time for sync or async operations
    ///
    /// # Returns
    /// A new `ExecConfig` instance configured with the given timeout
    pub fn new(timeout_duration: Duration) -> Self {
        ExecConfig {
            timeout_duration,
            fallback: None,
        }
    }

    /// Sets a fallback function to handle task failure or timeout scenarios.
    ///
    /// The fallback is a synchronous function, but can be used in both sync and async
    /// execution contexts. When used with async operations, the executing function
    /// should handle any necessary async adaptation.
    ///
    /// # Arguments
    /// * `fallback` - Synchronous function returning a `Result` with matching types
    pub fn with_fallback(&mut self, fallback: fn() -> Result<T, Box<dyn Error>>) {
        self.fallback = Some(fallback);
    }
}

/// Configuration for a Circuit Breaker.
///
/// The `CircuitBreakerConfig` struct holds the static configuration parameters for a circuit breaker.
/// It defines how the circuit breaker behaves during different states (Closed, Open, and HalfOpen).
/// These settings determine the thresholds for failures, successes, and the cooldown period for recovery attempts.
///
/// Use this struct to configure and initialize a `CircuitBreaker` instance with specific settings.
///
/// # Fields
/// - `failure_threshold`: The maximum number of consecutive failures before the circuit breaker transitions
///   from `Close` to `Open`. This threshold determines how sensitive the circuit breaker is to failures.
/// - `success_threshold`: The number of successful operations required in the `HalfOpen` state before
///   transitioning back to `Close`. This determines how many recovery attempts the system will test before
///   considering the service restored.
/// - `cooldown_period`: The duration to wait in the `Open` state before transitioning to `HalfOpen` to test
///   if the system has recovered. This period allows the failing system time to stabilize and prevents
///   immediate retries.
///
/// # Example
/// ```
/// use std::time::Duration;
/// use resilient_rs::config::CircuitBreakerConfig;
///
/// let config = CircuitBreakerConfig::new(3, 5, Duration::from_secs(10));
/// println!("{:?}", config);
/// ```

#[derive(Debug, Clone, Copy)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: usize,
    pub success_threshold: usize,
    pub cooldown_period: Duration,
}

impl Default for CircuitBreakerConfig {
    /// # Default Configuration
    /// The default configuration sets:
    /// - `failure_threshold` to 5 (max failures before opening the circuit)
    /// - `success_threshold` to 2 (successes required to close the circuit from HalfOpen)
    /// - `cooldown_period` to 2 seconds (time to wait before testing recovery)
    fn default() -> Self {
        Self {
            success_threshold: 2,
            failure_threshold: 5,
            cooldown_period: Duration::from_secs(2),
        }
    }
}

impl CircuitBreakerConfig {
    /// Creates a new `CircuitBreakerConfig` instance with the specified settings.
    ///
    /// This method constructs a configuration object for a circuit breaker based on the provided thresholds
    /// and cooldown period. The configuration defines how the circuit breaker will behave during operation.
    ///
    /// # Parameters
    /// - `success_threshold`: The number of successful operations required in the `HalfOpen` state
    ///   to transition back to `Close`. This must be greater than 0 for meaningful recovery.
    /// - `failure_threshold`: The number of consecutive failures in the `Close` state that will trigger
    ///   a transition to `Open`. This must be greater than 0.
    /// - `cooldown_period`: The duration to wait in the `Open` state before moving to `HalfOpen` to test
    ///   recovery. Should be long enough to allow the system to stabilize and prevent immediate retries.
    ///
    /// # Returns
    /// A new `CircuitBreakerConfig` instance with the provided parameters.
    ///
    /// # Panics
    /// This function will panic if any parameter is invalid (e.g., zero or negative values for thresholds).
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use resilient_rs::config::CircuitBreakerConfig;
    /// let config = CircuitBreakerConfig::new(3, 5, Duration::from_secs(10));
    /// assert_eq!(config.failure_threshold, 5);
    /// ```
    pub fn new(
        success_threshold: usize,
        failure_threshold: usize,
        cooldown_period: Duration,
    ) -> Self {
        assert!(
            success_threshold > 0,
            "success_threshold must be greater than 0"
        );
        assert!(
            failure_threshold > 0,
            "failure_threshold must be greater than 0"
        );
        assert!(
            cooldown_period > Duration::ZERO,
            "cooldown_period must be non-zero"
        );

        Self {
            failure_threshold,
            success_threshold,
            cooldown_period,
        }
    }

    /// Builder-style setter for `failure_threshold`.
    ///
    /// This method allows you to modify the `failure_threshold` value after the initial configuration.
    /// It enables more flexible configuration using a builder pattern.
    ///
    /// # Parameters
    /// - `threshold`: The number of failures required to trigger a transition to `Open`. Must be greater than 0.
    ///
    /// # Returns
    /// A new `CircuitBreakerConfig` instance with the updated `failure_threshold`.
    ///
    /// # Example
    /// ```
    /// use resilient_rs::config::CircuitBreakerConfig;
    /// let config = CircuitBreakerConfig::default().with_failure_threshold(3);
    /// assert_eq!(config.failure_threshold, 3);
    /// ```
    pub fn with_failure_threshold(mut self, threshold: usize) -> Self {
        assert!(threshold > 0, "failure_threshold must be greater than 0");
        self.failure_threshold = threshold;
        self
    }

    /// Builder-style setter for `success_threshold`.
    ///
    /// This method allows you to modify the `success_threshold` value after the initial configuration.
    /// It enables more flexible configuration using a builder pattern.
    ///
    /// # Parameters
    /// - `threshold`: The number of successes required to close the circuit from the `HalfOpen` state. Must be greater than 0.
    ///
    /// # Returns
    /// A new `CircuitBreakerConfig` instance with the updated `success_threshold`.
    ///
    /// # Example
    /// ```
    /// use resilient_rs::config::CircuitBreakerConfig;
    /// let config = CircuitBreakerConfig::default().with_success_threshold(4);
    /// assert_eq!(config.success_threshold, 4);
    /// ```
    pub fn with_success_threshold(mut self, threshold: usize) -> Self {
        assert!(threshold > 0, "success_threshold must be greater than 0");
        self.success_threshold = threshold;
        self
    }

    /// Builder-style setter for `cooldown_period`.
    ///
    /// This method allows you to modify the `cooldown_period` value after the initial configuration.
    /// It enables more flexible configuration using a builder pattern.
    ///
    /// # Parameters
    /// - `period`: The duration to wait before testing the recovery of the system. Must be greater than 0.
    ///
    /// # Returns
    /// A new `CircuitBreakerConfig` instance with the updated `cooldown_period`.
    ///
    /// # Example
    /// ```
    /// use resilient_rs::config::CircuitBreakerConfig;
    /// let config = CircuitBreakerConfig::default().with_cooldown_period(std::time::Duration::from_secs(5));
    /// assert_eq!(config.cooldown_period, std::time::Duration::from_secs(5));
    /// ```
    pub fn with_cooldown_period(mut self, period: Duration) -> Self {
        assert!(period > Duration::ZERO, "cooldown_period must be non-zero");
        self.cooldown_period = period;
        self
    }
}
