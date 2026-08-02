/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

use dashmap::DashMap;
use governor::{state::InMemoryState, state::NotKeyed, Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;

pub struct GlobalRateLimiter {
    // Key: TokenID, Value: Limiter (RPS)
    token_rps_limits:
        DashMap<i64, Arc<RateLimiter<NotKeyed, InMemoryState, governor::clock::DefaultClock>>>,
    // Key: TokenID, Value: Limiter (RPM)
    token_rpm_limits:
        DashMap<i64, Arc<RateLimiter<NotKeyed, InMemoryState, governor::clock::DefaultClock>>>,
    /// 登录/管理登录按 IP 限流（每分钟）
    login_ip_limits:
        DashMap<String, Arc<RateLimiter<NotKeyed, InMemoryState, governor::clock::DefaultClock>>>,
}

impl GlobalRateLimiter {
    pub fn new() -> Self {
        Self {
            token_rps_limits: DashMap::new(),
            token_rpm_limits: DashMap::new(),
            login_ip_limits: DashMap::new(),
        }
    }

    /// 登录尝试限流：同一 IP 每分钟最多 `per_minute` 次（默认 10）。
    pub fn check_login_ip(&self, ip: &str, per_minute: u32) -> bool {
        let limit = per_minute.max(1);
        let limiter = self
            .login_ip_limits
            .entry(ip.to_string())
            .or_insert_with(|| {
                let quota = Quota::per_minute(NonZeroU32::new(limit).unwrap());
                Arc::new(RateLimiter::direct(quota))
            });
        limiter.check().is_ok()
    }

    pub fn check_rps(&self, token_id: i64, rps: i32) -> bool {
        if rps <= 0 {
            return true;
        }

        let limiter = self.token_rps_limits.entry(token_id).or_insert_with(|| {
            let quota = Quota::per_second(NonZeroU32::new(rps as u32).unwrap());
            Arc::new(RateLimiter::direct(quota))
        });

        limiter.check().is_ok()
    }

    pub fn check_rpm(&self, token_id: i64, rpm: i32) -> bool {
        if rpm <= 0 {
            return true;
        }

        let limiter = self.token_rpm_limits.entry(token_id).or_insert_with(|| {
            let quota = Quota::per_minute(NonZeroU32::new(rpm as u32).unwrap());
            Arc::new(RateLimiter::direct(quota))
        });

        limiter.check().is_ok()
    }
}
