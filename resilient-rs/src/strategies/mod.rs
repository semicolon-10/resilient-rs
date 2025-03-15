use std::time::Duration;

/// Defines the retry strategy to use when scheduling retry attempts.
///
/// This enum specifies how delays between retries are calculated.
#[derive(Debug)]
pub enum RetryStrategy {
    /// A linear retry strategy where the delay between retries remains constant.
    ///
    /// For example, if the delay is set to 2 seconds, each retry will wait exactly 2 seconds.
    Linear,
    /// An exponential backoff strategy where the delay increases exponentially with each retry.
    ///
    /// For example, with a base delay of 2 seconds, retries might wait 2s, 4s, 8s, etc.
    ExponentialBackoff,
}

/// Configuration for retrying operations.
///
/// This struct defines the parameters for retrying an operation, including
/// the maximum number of attempts, the delay between retries, and the retry strategy.

impl RetryStrategy {
    /// Calculates the delay duration for a specific retry attempt based on the retry strategy.
    ///
    /// This method determines how long to wait before the next retry attempt, using the provided
    /// `base_delay` as a starting point. The actual delay depends on the `RetryStrategy` variant:
    /// - For `Linear`, the delay is constant and equal to `base_delay` for all attempts.
    /// - For `ExponentialBackoff`, the delay increases exponentially as `base_delay * 2^(attempt-1)`,
    ///   with the first retry (attempt = 1) using the `base_delay` directly.
    ///
    /// # Arguments
    /// * `base_delay` - The base duration to use as the starting point for delay calculations.
    ///                  This is typically provided by a configuration like `RetryConfig`.
    /// * `attempt` - The current attempt number, where:
    ///               - `0` represents the initial attempt (though typically retries start at 1).
    ///               - `1` represents the first retry, `2` the second retry, and so on.
    ///
    /// # Returns
    /// A `Duration` representing the time to wait before the next retry attempt.
    ///
    /// # Notes
    /// - For `ExponentialBackoff`, the delay grows as a power of 2 based on the attempt number.
    ///   Be cautious with large `attempt` values, as the result could exceed `Duration` limits.
    /// - The `attempt` parameter is assumed to be non-negative; negative values are not handled.
    pub(crate) fn calculate_delay(&self, base_delay: Duration, attempt: usize) -> Duration {
        match self {
            RetryStrategy::Linear => base_delay,
            RetryStrategy::ExponentialBackoff => {
                if attempt == 0 {
                    base_delay
                } else {
                    base_delay * 2u32.pow((attempt - 1) as u32)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_linear_strategy() {
        let base_delay = Duration::from_secs(2);
        let linear = RetryStrategy::Linear;

        // Test that Linear strategy returns a constant delay
        assert_eq!(
            linear.calculate_delay(base_delay, 0),
            Duration::from_secs(2)
        ); // Initial attempt
        assert_eq!(
            linear.calculate_delay(base_delay, 1),
            Duration::from_secs(2)
        ); // First retry
        assert_eq!(
            linear.calculate_delay(base_delay, 2),
            Duration::from_secs(2)
        ); // Second retry
        assert_eq!(
            linear.calculate_delay(base_delay, 3),
            Duration::from_secs(2)
        ); // Third retry
    }

    #[test]
    fn test_exponential_backoff_strategy() {
        let base_delay = Duration::from_secs(2);
        let expo = RetryStrategy::ExponentialBackoff;

        // Test that ExponentialBackoff increases delay exponentially
        assert_eq!(expo.calculate_delay(base_delay, 0), Duration::from_secs(2)); // Initial attempt
        assert_eq!(expo.calculate_delay(base_delay, 1), Duration::from_secs(2)); // First retry: 2^0 * 2s
        assert_eq!(expo.calculate_delay(base_delay, 2), Duration::from_secs(4)); // Second retry: 2^1 * 2s
        assert_eq!(expo.calculate_delay(base_delay, 3), Duration::from_secs(8)); // Third retry: 2^2 * 2s
        assert_eq!(expo.calculate_delay(base_delay, 4), Duration::from_secs(16)); // Fourth retry: 2^3 * 2s
    }
}
