/// The `asynchronous` module provides utilities for handling retries and resilience
/// in asynchronous contexts. This includes retry logic and other resilience patterns
/// that are compatible with async/await.
pub mod asynchronous;

/// The `config` module provides configuration structures for retry logic and other
/// resilience patterns. This includes settings like the maximum number of attempts
/// and delay between retries.
pub mod config;

/// The `strategies` module defines different retry strategies used for handling
/// transient failures. It provides mechanisms to calculate appropriate delay
/// durations between retry attempts, supporting both linear and exponential backoff approaches.
///
/// This module is utilized by both synchronous and asynchronous retry mechanisms.
pub mod strategies;
/// The `synchronous` module provides utilities for handling retries and resilience
/// in synchronous contexts. This includes retry logic and other resilience patterns
/// for blocking operations.
pub mod synchronous;
