use std::time::Duration;

use resilient_rs::config::RetryConfig;
use resilient_rs::config::RetryStrategy::{ExponentialBackoff, Linear};
use resilient_rs::synchronous::retry;

// Example 1: Using retry() with a simple failing operation
pub fn example_simple_retry() {
    // Configure retry with default
    let retry_config = RetryConfig::default();

    let mut attempt_count = 0;

    let result = retry(
        || {
            attempt_count += 1;
            println!("Attempt #{}", attempt_count);

            // Simulate an operation that fails twice before succeeding
            if attempt_count < 3 {
                Err("Temporary error")
            } else {
                Ok("Operation completed successfully")
            }
        },
        &retry_config,
    );

    match result {
        Ok(success_msg) => println!("Success: {}", success_msg),
        Err(error) => println!("Failed after retries: {}", error),
    }
}

// Example 2: Using retry_with_exponential_backoff() with a counter
pub fn example_exponential_backoff() {
    // Configure retry with 4 attempts and initial 100ms delay
    let retry_config = RetryConfig {
        max_attempts: 4,
        delay: Duration::from_millis(100),
        retry_condition: None,
        strategy: ExponentialBackoff,
    };

    let mut counter = 0;

    let result = retry(
        || {
            counter += 1;
            println!("Attempt #{} with increasing delay", counter);

            // Simulate an operation that succeeds on the 3rd attempt
            if counter < 3 { Err("Not yet") } else { Ok(42) }
        },
        &retry_config,
    );

    match result {
        Ok(value) => println!("Got value: {}", value),
        Err(error) => println!("Failed: {}", error),
    }
}

// Example 3: Using retry() with a retry condition
pub fn example_retry_with_condition() {
    // Define a retry condition that only retries on specific error messages
    let should_retry = |error: &String| error.contains("401") || error.contains("404");

    // Configure retry with condition
    let retry_config = RetryConfig {
        max_attempts: 4,
        delay: Duration::from_millis(300),
        retry_condition: Some(should_retry),
        strategy: Linear,
    };

    let mut attempt_count = 0;

    let result = retry(
        || {
            attempt_count += 1;
            println!("Attempt #{}", attempt_count);

            match attempt_count {
                1 => Err("transient failure".to_string()), // Will retry (contains "transient")
                2 => Err("timeout occurred".to_string()),  // Will retry (contains "timeout")
                3 => Err("permanent error".to_string()),   // Won't retry (no match)
                _ => Ok("Operation completed".to_string()), // Success case
            }
        },
        &retry_config,
    );

    match result {
        Ok(success_msg) => println!("Success: {}", success_msg),
        Err(error) => println!("Failed with: {}", error),
    }
}
