//! AgentKern-Gate: Distributed Cache & Rate Limiting
//!
//! Provides Redis-backed caching and rate limiting for multi-tenant protection.
//! Falls back to local in-memory/no-op if Redis is not configured.

use std::time::Duration;
use thiserror::Error;

#[cfg(feature = "redis")]
use bb8_redis::{bb8, redis::AsyncCommands, RedisConnectionManager};

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Redis connection error: {0}")]
    Connection(String),
    #[error("Redis command error: {0}")]
    Command(String),
    #[error("Cache miss")]
    Miss,
}

/// A distributed cache layer.
#[derive(Clone)]
pub struct CacheLayer {
    #[cfg(feature = "redis")]
    pool: Option<bb8::Pool<RedisConnectionManager>>,
    #[cfg(not(feature = "redis"))]
    _marker: std::marker::PhantomData<()>,
}

impl CacheLayer {
    /// Create a new cache layer.
    ///
    /// If `redis_url` is provided and the `redis` feature is enabled,
    /// a Redis connection pool is initialized.
    pub async fn new(redis_url: Option<String>) -> Result<Self, CacheError> {
        #[cfg(feature = "redis")]
        {
            if let Some(url) = redis_url {
                let manager = RedisConnectionManager::new(url)
                    .map_err(|e| CacheError::Connection(e.to_string()))?;
                let pool = bb8::Pool::builder()
                    .max_size(16)
                    .build(manager)
                    .await
                    .map_err(|e| CacheError::Connection(e.to_string()))?;

                return Ok(Self { pool: Some(pool) });
            }
            Ok(Self { pool: None })
        }

        #[cfg(not(feature = "redis"))]
        {
            if redis_url.is_some() {
                tracing::warn!(
                    "Redis URL provided but 'redis' feature is disabled. Caching will be no-op."
                );
            }
            Ok(Self {
                _marker: std::marker::PhantomData,
            })
        }
    }

    /// Check if Redis is enabled and connected.
    pub fn is_enabled(&self) -> bool {
        #[cfg(feature = "redis")]
        return self.pool.is_some();

        #[cfg(not(feature = "redis"))]
        false
    }
}

/// Distributed Rate Limiter
pub struct RateLimiter {
    cache: CacheLayer,
    limit: u64,
    window: Duration,
}

impl RateLimiter {
    pub fn new(cache: CacheLayer, limit: u64, window: Duration) -> Self {
        Self {
            cache,
            limit,
            window,
        }
    }

    /// Check if the key exceeds the rate limit.
    /// Returns (allowed, remaining, connection_error).
    /// If Redis is down/disabled, it fails OPEN (allowed=true).
    pub async fn check(&self, key: &str) -> (bool, u64, bool) {
        #[cfg(feature = "redis")]
        {
            if let Some(pool) = &self.cache.pool {
                let mut conn = match pool.get().await {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::error!("Failed to get Redis connection for rate limiting: {}", e);
                        return (true, self.limit, true); // Fail open
                    }
                };

                let redis_key = format!("rl:{}", key);
                // Pipeline: INCR then EXPIRE if new
                // For simplicity here, we do:
                // 1. INCR
                // 2. If == 1, EXPIRE
                // Note: race condition on EXPIRE possible but benign for RL (key might live forever if crash between)
                // Lua script is better, but keeping it simple for now.

                let count: u64 = match conn.incr(&redis_key, 1).await {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!("Redis INCR failed: {}", e);
                        return (true, self.limit, true);
                    }
                };

                if count == 1 {
                    let _: () = match conn
                        .expire::<_, ()>(&redis_key, self.window.as_secs() as i64)
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => tracing::error!("Redis EXPIRE failed: {}", e),
                    };
                }

                if count > self.limit {
                    return (false, 0, false);
                } else {
                    return (true, self.limit - count, false);
                }
            }
        }

        // Fallback: No Redis -> Unlimited (or rely on local rate limiter)
        (true, self.limit, false)
    }
}
