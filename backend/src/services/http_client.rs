/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

//! 全局出站 HTTP 客户端：连接层容错 + 连接池复用。
//! 不设 Client 级总超时，避免切断长流式；非流式在请求级使用 [`with_upstream_timeout`]。

use std::sync::OnceLock;
use std::time::Duration;

fn env_secs(key: &str, default: u64) -> Duration {
    Duration::from_secs(
        std::env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .filter(|&n| n > 0)
            .unwrap_or(default),
    )
}

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n| n > 0)
        .unwrap_or(default)
}

fn upstream_timeout() -> Duration {
    static TIMEOUT: OnceLock<Duration> = OnceLock::new();
    *TIMEOUT.get_or_init(|| env_secs("HTTP_UPSTREAM_TIMEOUT_SECS", 1800))
}

/// 出站 Client 公共基线（低延迟、建连超时、keepalive、空闲回收）。
pub fn outbound_client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .tcp_nodelay(true)
        .tcp_keepalive(env_secs("HTTP_TCP_KEEPALIVE_SECS", 60))
        .connect_timeout(env_secs("HTTP_CONNECT_TIMEOUT_SECS", 10))
        .pool_idle_timeout(env_secs("HTTP_POOL_IDLE_TIMEOUT_SECS", 90))
        .pool_max_idle_per_host(env_usize("HTTP_POOL_MAX_IDLE_PER_HOST", 100))
}

/// AppState 共享客户端：无全局 request timeout（流式安全）。
pub fn build_outbound_client() -> reqwest::Client {
    outbound_client_builder()
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

/// 为非流式 RequestBuilder 挂上防挂死总超时（默认 1800s，可用 HTTP_UPSTREAM_TIMEOUT_SECS 覆盖）。
#[inline]
pub fn with_upstream_timeout(builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    builder.timeout(upstream_timeout())
}
