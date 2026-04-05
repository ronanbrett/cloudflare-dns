#![allow(dead_code)]

//! Caching layer for DNS records.
//!
//! This module provides a simple in-memory cache with TTL (time-to-live) support
//! to reduce redundant API calls when refreshing the UI.

use std::time::{Duration, Instant};

use crate::api::DnsRecord;

/// Cache for DNS records with automatic expiration.
///
/// The cache stores records along with their fetch timestamp and expires
/// after a configurable TTL to ensure data freshness.
pub struct DnsCache {
    /// Cached DNS records (None means not yet loaded or expired)
    records: Option<Vec<DnsRecord>>,
    /// When the cache was last populated
    last_updated: Option<Instant>,
    /// How long the cache is valid
    ttl: Duration,
}

impl DnsCache {
    /// Create a new DNS record cache.
    ///
    /// # Arguments
    /// * `ttl` - How long to keep records cached before requiring a refresh
    pub fn new(ttl: Duration) -> Self {
        Self {
            records: None,
            last_updated: None,
            ttl,
        }
    }

    /// Create a cache with default 60-second TTL.
    pub fn with_default_ttl() -> Self {
        Self::new(Duration::from_secs(60))
    }

    /// Check if the cache has valid (non-expired) records.
    pub fn is_valid(&self) -> bool {
        match self.last_updated {
            Some(updated) => updated.elapsed() < self.ttl,
            None => false,
        }
    }

    /// Get cached records if valid, otherwise returns None.
    pub fn get(&self) -> Option<&Vec<DnsRecord>> {
        if self.is_valid() {
            self.records.as_ref()
        } else {
            None
        }
    }

    /// Update the cache with fresh records.
    pub fn set(&mut self, records: Vec<DnsRecord>) {
        self.records = Some(records);
        self.last_updated = Some(Instant::now());
    }

    /// Force invalidate the cache.
    pub fn invalidate(&mut self) {
        self.records = None;
        self.last_updated = None;
    }

    /// Get the age of the cached records.
    pub fn age(&self) -> Option<Duration> {
        self.last_updated.map(|t| t.elapsed())
    }

    /// Get remaining TTL before cache expires.
    pub fn remaining_ttl(&self) -> Option<Duration> {
        self.last_updated.map(|updated| {
            let elapsed = updated.elapsed();
            if elapsed < self.ttl {
                self.ttl - elapsed
            } else {
                Duration::ZERO
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn make_record(id: &str) -> DnsRecord {
        DnsRecord {
            id: Some(id.to_string()),
            record_type: "A".to_string(),
            name: "example.com".to_string(),
            content: "127.0.0.1".to_string(),
            ttl: Some(300),
            proxied: Some(false),
            comment: None,
        }
    }

    #[test]
    fn test_cache_initially_invalid() {
        let cache = DnsCache::with_default_ttl();
        assert!(!cache.is_valid());
        assert!(cache.get().is_none());
    }

    #[test]
    fn test_cache_set_and_get() {
        let mut cache = DnsCache::with_default_ttl();
        let records = vec![make_record("1"), make_record("2")];
        cache.set(records);

        assert!(cache.is_valid());
        let cached = cache.get().unwrap();
        assert_eq!(cached.len(), 2);
        assert_eq!(cached[0].id, Some("1".to_string()));
    }

    #[test]
    fn test_cache_expires() {
        let mut cache = DnsCache::new(Duration::from_millis(100));
        let records = vec![make_record("1")];
        cache.set(records);

        assert!(cache.is_valid());

        // Wait for cache to expire
        thread::sleep(Duration::from_millis(150));

        assert!(!cache.is_valid());
        assert!(cache.get().is_none());
    }

    #[test]
    fn test_cache_invalidate() {
        let mut cache = DnsCache::with_default_ttl();
        cache.set(vec![make_record("1")]);
        assert!(cache.is_valid());

        cache.invalidate();
        assert!(!cache.is_valid());
        assert!(cache.get().is_none());
    }

    #[test]
    fn test_cache_age() {
        let mut cache = DnsCache::with_default_ttl();
        assert!(cache.age().is_none());

        cache.set(vec![make_record("1")]);
        assert!(cache.age().is_some());
        assert!(cache.age().unwrap() < Duration::from_secs(1));
    }

    #[test]
    fn test_cache_remaining_ttl() {
        let mut cache = DnsCache::new(Duration::from_secs(10));
        assert!(cache.remaining_ttl().is_none());

        cache.set(vec![make_record("1")]);
        let remaining = cache.remaining_ttl().unwrap();
        assert!(remaining <= Duration::from_secs(10));
        assert!(remaining > Duration::from_secs(9));
    }

    #[test]
    fn test_cache_remaining_ttl_zero_when_expired() {
        let mut cache = DnsCache::new(Duration::from_millis(50));
        cache.set(vec![make_record("1")]);

        thread::sleep(Duration::from_millis(100));
        assert_eq!(cache.remaining_ttl().unwrap(), Duration::ZERO);
    }
}
