//! # Token-bucket rate limiter
//!
//! Per-capability rate limiting using a token-bucket algorithm.
//! Each capability has its own bucket; tokens refill at
//! `rate_limit_per_minute / 60` tokens per second.

use std::collections::HashMap;
use std::time::Instant;

/// A single token bucket for one capability.
#[derive(Debug, Clone)]
struct Bucket {
    /// Current number of available tokens (fractional for smooth refill).
    tokens: f64,
    /// Maximum tokens (= rate_limit_per_minute).
    max_tokens: f64,
    /// Tokens added per second.
    refill_rate: f64,
    /// Last time tokens were refilled.
    last_refill: Instant,
}

impl Bucket {
    fn new(rate_limit_per_minute: u32, now: Instant) -> Self {
        let max = rate_limit_per_minute as f64;
        Bucket {
            tokens: max, // start full
            max_tokens: max,
            refill_rate: max / 60.0,
            last_refill: now,
        }
    }

    /// Refill tokens based on elapsed time.
    fn refill(&mut self, now: Instant) {
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        if elapsed > 0.0 {
            self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
            self.last_refill = now;
        }
    }

    /// Try to consume one token. Returns `Ok(())` if allowed, or
    /// `Err(retry_after_ms)` if rate-limited.
    fn try_acquire(&mut self, now: Instant) -> Result<(), u64> {
        self.refill(now);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            Ok(())
        } else {
            // How long until 1 token is available?
            let deficit = 1.0 - self.tokens;
            let wait_secs = deficit / self.refill_rate;
            let wait_ms = (wait_secs * 1000.0).ceil() as u64;
            Err(wait_ms.max(1))
        }
    }
}

/// Rate limiter managing per-capability token buckets.
#[derive(Debug)]
pub struct RateLimiter {
    buckets: HashMap<String, Bucket>,
    /// Overridden "now" for testing. If `None`, uses `Instant::now()`.
    #[cfg(test)]
    test_now: Option<Instant>,
}

impl RateLimiter {
    pub fn new() -> Self {
        RateLimiter {
            buckets: HashMap::new(),
            #[cfg(test)]
            test_now: None,
        }
    }

    /// Register a capability's rate limit. Call this when a capability
    /// is registered. If `rate_limit_per_minute` is 0, no bucket is created
    /// (unlimited).
    pub fn configure(&mut self, capability_id: &str, rate_limit_per_minute: u32) {
        if rate_limit_per_minute == 0 {
            self.buckets.remove(capability_id);
            return;
        }
        let now = self.now();
        self.buckets
            .insert(capability_id.to_string(), Bucket::new(rate_limit_per_minute, now));
    }

    /// Try to acquire a token for the given capability.
    ///
    /// Returns `Ok(())` if allowed, `Err(retry_after_ms)` if rate-limited.
    /// If no bucket exists for this capability, it's unlimited → always OK.
    pub fn try_acquire(&mut self, capability_id: &str) -> Result<(), u64> {
        let now = self.now();
        match self.buckets.get_mut(capability_id) {
            Some(bucket) => bucket.try_acquire(now),
            None => Ok(()), // no rate limit configured
        }
    }

    /// Remove a capability's rate limit bucket.
    pub fn remove(&mut self, capability_id: &str) {
        self.buckets.remove(capability_id);
    }

    fn now(&self) -> Instant {
        #[cfg(test)]
        if let Some(t) = self.test_now {
            return t;
        }
        Instant::now()
    }

    /// For testing: set the current time.
    #[cfg(test)]
    pub fn set_now(&mut self, now: Instant) {
        self.test_now = Some(now);
        // Also update all bucket last_refill to avoid massive refill on next acquire
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_unlimited_rate() {
        let mut limiter = RateLimiter::new();
        // No bucket configured → always allowed
        for _ in 0..1000 {
            assert!(limiter.try_acquire("anything").is_ok());
        }
    }

    #[test]
    fn test_rate_limit_triggers() {
        let mut limiter = RateLimiter::new();
        // 2 calls per minute
        limiter.configure("test.cap", 2);

        assert!(limiter.try_acquire("test.cap").is_ok()); // token 1
        assert!(limiter.try_acquire("test.cap").is_ok()); // token 2
        let result = limiter.try_acquire("test.cap"); // exhausted
        assert!(result.is_err());
        let retry_ms = result.unwrap_err();
        assert!(retry_ms > 0);
    }

    #[test]
    fn test_tokens_refill() {
        let start = Instant::now();
        let mut limiter = RateLimiter::new();
        limiter.set_now(start);

        // 60 per minute = 1 per second
        limiter.configure("fast", 60);

        // Exhaust all 60 tokens
        for _ in 0..60 {
            assert!(limiter.try_acquire("fast").is_ok());
        }
        assert!(limiter.try_acquire("fast").is_err());

        // Advance 2 seconds → 2 tokens refilled
        limiter.set_now(start + Duration::from_secs(2));
        assert!(limiter.try_acquire("fast").is_ok());
        assert!(limiter.try_acquire("fast").is_ok());
        assert!(limiter.try_acquire("fast").is_err());
    }

    #[test]
    fn test_separate_capabilities() {
        let mut limiter = RateLimiter::new();
        limiter.configure("cap_a", 1);
        limiter.configure("cap_b", 1);

        assert!(limiter.try_acquire("cap_a").is_ok());
        assert!(limiter.try_acquire("cap_a").is_err()); // exhausted
        assert!(limiter.try_acquire("cap_b").is_ok()); // separate bucket
    }

    #[test]
    fn test_remove_bucket() {
        let mut limiter = RateLimiter::new();
        limiter.configure("cap", 1);
        assert!(limiter.try_acquire("cap").is_ok());
        assert!(limiter.try_acquire("cap").is_err());

        limiter.remove("cap");
        // Now unlimited
        assert!(limiter.try_acquire("cap").is_ok());
    }
}
