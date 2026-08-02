/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

//! 高可用（HA）failover 策略：端点只碰 [`HaAttempt`] + 少量选渠/落库辅助。
//! 语义：仅当「插件启用 AND 令牌 high_availability≠0」时可选 HA 组并做子渠切换。
//!
//! 终态对齐规则：
//! - **全部上游失败**：客户端错误 + 日志的 status/error/`channel_config_id` 保留「第一次」失败（子渠 1）
//! - **某次成功**：日志 channel_id / `channel_config_id` / 上游 URL 使用**成功那一次**的子渠
//! - 展示用 YID：读路径 JOIN `channel_configs`，日志表不落 `yid` 列
//! - 展示用 HA：写路径快照 `logs.is_ha`（见 [`channel_is_ha_flag`]），禁止 JOIN 当前 `channels.provider_type`

use crate::error::AppError;
use crate::models::Channel;
use crate::AppState;
use std::sync::Arc;
use std::time::Instant;

/// 一次解析：`(failover_on, max_attempts)`，供选渠开环前调用。
pub async fn policy(state: &AppState, token_ha: i32) -> (bool, usize) {
    let (_, plugin_on) = super::get_cached_config(state).await;
    let on = plugin_on && token_ha != 0;
    let attempts = if on {
        state
            .ha_max_retries
            .load(std::sync::atomic::Ordering::Relaxed)
            .max(1) as usize
    } else {
        1
    };
    (on, attempts)
}

/// 日志展示用 yid（空 → `-`，与后台「上游 YID」对齐）
#[inline]
pub fn yid_label(yid: Option<&str>) -> &str {
    yid.map(str::trim).filter(|s| !s.is_empty()).unwrap_or("-")
}

/// 落库用 0/1（列类型 INTEGER / INT4）
#[inline]
pub fn channel_is_ha_flag(channel: &Channel) -> i32 {
    let is_ha = channel.provider_type == "high_availability_group"
        || channel.group_aid.as_deref().is_some_and(is_ha_aid);
    i32::from(is_ha)
}

/// 落库用子配 id：运行时 aid/preset 优先，否则用内存 `Channel.yid` 反查
pub async fn resolve_log_config_id(state: &AppState, channel: &Channel) -> Option<i32> {
    if let Some(id) = resolve_config_id(channel) {
        return Some(id);
    }
    let y = channel
        .yid
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())?;
    sqlx::query_scalar::<_, i32>(
        &state
            .db
            .format_query("SELECT id FROM channel_configs WHERE yid = ? LIMIT 1"),
    )
    .bind(y)
    .fetch_optional(&state.db.pool)
    .await
    .ok()
    .flatten()
}

/// 是否仍在熔断窗口内；已过期则立即移除该键
#[inline]
pub fn is_melted_down(state: &AppState, aid: &str) -> bool {
    let Some(entry) = state.failed_channels.get(aid) else {
        return false;
    };
    if *entry.value() > Instant::now() {
        true
    } else {
        drop(entry);
        state.failed_channels.remove(aid);
        false
    }
}

/// 批量清除过期熔断；超软上限则整清
pub fn scrub_failed_channels(state: &AppState) {
    let now = Instant::now();
    state.failed_channels.retain(|_, until| *until > now);
    let n = state.failed_channels.len();
    if n > FAILED_CHANNELS_SOFT_CAP {
        tracing::warn!(
            "[HA] 熔断表数量={} 超过软上限={}，整表清空",
            n,
            FAILED_CHANNELS_SOFT_CAP
        );
        state.failed_channels.clear();
    }
}

/// HA 外环状态：各端点共用。失败续试只走 [`HaAttempt::on_spawn_result_err`]。
pub struct HaAttempt {
    pub exclude_aids: Vec<String>,
    pub attempt: usize,
    pub max_attempts: usize,
    pub failover_on: bool,
    pub first_fail: Option<FirstUpstreamFail>,
    pub pending_log_id: Option<i64>,
    pub had_upstream: bool,
    pub last_err: AppError,
}

/// 第一次上游失败快照（仅当前请求栈内）
#[derive(Debug, Clone)]
pub struct FirstUpstreamFail {
    pub status: u16,
    pub message: String,
    pub channel_id: i64,
    pub channel_config_id: Option<i32>,
    pub upstream_url: Option<String>,
}

impl HaAttempt {
    /// 开环：解析 HA 策略，初始化排除列表与终态错误占位
    pub async fn begin(state: &AppState, token_ha: i32) -> Self {
        let (failover_on, max_attempts) = policy(state, token_ha).await;
        if failover_on {
            tracing::info!("[HA] 开始 最大尝试={} 令牌HA={}", max_attempts, token_ha);
        }
        Self {
            exclude_aids: vec![],
            attempt: 0,
            max_attempts,
            failover_on,
            first_fail: None,
            pending_log_id: None,
            had_upstream: false,
            last_err: AppError::UpstreamError("No available models".into()),
        }
    }

    #[inline]
    pub fn cont(&self) -> bool {
        self.attempt < self.max_attempts
    }

    /// 选渠失败：尚无上游交互时用选渠错误作为对外文案
    pub fn on_select_err(&mut self, e: AppError) {
        if !self.had_upstream {
            self.last_err = e;
        }
    }

    /// 权限/余额等不可 failover 错误
    pub fn on_access_err(&mut self, e: AppError) {
        self.last_err = e;
    }

    /// 外环失败统一入口（业务侧停环；上游失败记 first_fail / 熔断或 skip-melt / reinstate）。
    /// 返回 true → `bump(); continue`；false → `break`。
    pub async fn on_spawn_result_err(
        &mut self,
        state: &Arc<AppState>,
        channel: &Channel,
        err: AppError,
        upstream_url: Option<&str>,
    ) -> bool {
        let aid = channel.group_aid.as_deref().unwrap_or("-");
        let yid = yid_label(channel.yid.as_deref());
        let status = err.http_status();
        let n = self.attempt + 1;

        if is_access_side_err(&err) {
            tracing::info!(
                "[HA] 业务侧停止 状态码={} 上游YID={} 子渠标识={} 尝试={}/{}",
                status,
                yid,
                aid,
                n,
                self.max_attempts
            );
            self.on_access_err(err);
            return false;
        }

        self.had_upstream = true;
        let was_first = self.first_fail.is_none();
        let masked_url =
            upstream_url.map(|u| super::forward::mask_key_in_string(u, &channel.api_key));

        let (fail_status, msg) = status_msg(&err);
        if was_first {
            self.first_fail = Some(FirstUpstreamFail {
                status: fail_status,
                message: msg.clone(),
                channel_id: channel.id,
                channel_config_id: resolve_config_id(channel),
                upstream_url: masked_url,
            });
            self.last_err = err;
        } else {
            let _ = err;
        }

        let cont = try_failover(
            state,
            self.failover_on,
            channel.group_aid.as_deref(),
            channel.yid.as_deref(),
            fail_status,
            &msg,
            &mut self.exclude_aids,
        );

        if !was_first {
            if let Some(ref f) = self.first_fail {
                reinstate_first_log(state, self.pending_log_id, f).await;
            }
        }

        if cont {
            tracing::info!(
                "[HA] 切换 状态码={} 上游YID={} 子渠标识={} 尝试={}/{} 已排除={}",
                status,
                yid,
                aid,
                n,
                self.max_attempts,
                self.exclude_aids.len()
            );
        } else {
            tracing::info!(
                "[HA] 停止无切换 状态码={} 上游YID={} 子渠标识={} 尝试={}/{}（非HA子渠或HA关闭）",
                status,
                yid,
                aid,
                n,
                self.max_attempts
            );
        }
        cont
    }

    #[inline]
    pub fn bump(&mut self) {
        self.attempt += 1;
    }

    /// 环结束：业务侧 last_err 优先，否则 first_fail
    pub fn finish(self) -> AppError {
        tracing::info!(
            "[HA] 结束 已尝试={} 最大={} 已排除={:?}",
            self.attempt,
            self.max_attempts,
            self.exclude_aids
        );
        if is_access_side_err(&self.last_err) {
            return self.last_err;
        }
        match self.first_fail {
            Some(f) => super::proxy::upstream_fail(f.status, &f.message),
            None => self.last_err,
        }
    }
}

// ── 私有实现 ──────────────────────────────────────────────────

const FAILED_CHANNELS_SOFT_CAP: usize = 4096;

#[inline]
fn is_ha_aid(aid: &str) -> bool {
    aid.starts_with("ha_group_")
}

#[inline]
fn resolve_config_id(channel: &Channel) -> Option<i32> {
    if let Some(aid) = channel.group_aid.as_deref() {
        if is_ha_aid(aid) && aid.contains("_config_") {
            if let Some(id) = aid
                .rfind("_config_")
                .and_then(|pos| aid[pos + "_config_".len()..].parse().ok())
            {
                return Some(id);
            }
        }
    }
    channel.preset_id.map(|p| p as i32)
}

#[inline]
fn status_msg(err: &AppError) -> (u16, String) {
    match err {
        AppError::UpstreamHttpError(s, m) => (*s, m.clone()),
        AppError::UpstreamError(m) => (502, m.clone()),
        other => (other.http_status(), other.to_string()),
    }
}

/// 业务侧错误：禁止 HA failover / reinstate 覆盖
#[inline]
fn is_access_side_err(err: &AppError) -> bool {
    match err {
        AppError::PaymentRequired(_)
        | AppError::Forbidden(_)
        | AppError::BadRequest(_)
        | AppError::Unauthorized
        | AppError::AuthFailed(_) => true,
        AppError::Internal(m) if m.contains("预扣费") => true,
        _ => false,
    }
}

/// 上游客户端侧 HTTP：仍切换，但不熔断
#[inline]
fn is_client_side_http_status(status: u16) -> bool {
    matches!(status, 400 | 402 | 403 | 422)
}

async fn reinstate_first_log(
    state: &Arc<AppState>,
    pending_log_id: Option<i64>,
    first: &FirstUpstreamFail,
) {
    let Some(log_id) = pending_log_id else { return };
    let err_msg = super::proxy::extract_error_message(&first.message);
    if let Err(e) = sqlx::query(&state.db.format_query(
        "UPDATE logs SET channel_id = ?, status_code = ?, error_message = ?, \
         channel_config_id = ?, upstream_url = COALESCE(?, upstream_url) WHERE id = ?",
    ))
    .bind(first.channel_id)
    .bind(first.status as i32)
    .bind(&err_msg)
    .bind(first.channel_config_id)
    .bind(&first.upstream_url)
    .bind(log_id)
    .execute(&state.db.pool)
    .await
    {
        tracing::warn!("[HA] 还原首败日志失败 日志id={} 错误: {:?}", log_id, e);
    }
}

/// 仅 HA 子渠且策略开启时 exclude（并可能熔断）；400/402/403/422 只切换不熔断
fn try_failover(
    state: &Arc<AppState>,
    failover_on: bool,
    group_aid: Option<&str>,
    yid: Option<&str>,
    status: u16,
    err_msg: &str,
    exclude_aids: &mut Vec<String>,
) -> bool {
    if !failover_on {
        return false;
    }
    let Some(aid) = group_aid.filter(|a| is_ha_aid(a)) else {
        return false;
    };
    let yid_disp = yid_label(yid);
    if is_client_side_http_status(status) {
        tracing::info!(
            "[HA] 切换跳过熔断 状态码={} 上游YID={} 子渠标识={}（上游客户端侧错误）",
            status,
            yid_disp,
            aid
        );
    } else {
        trigger_ha_meltdown(state, aid, status, err_msg, yid);
    }
    if !exclude_aids.iter().any(|a| a == aid) {
        exclude_aids.push(aid.to_string());
    }
    true
}

/// 写入 HA 子渠熔断冷却；白名单命中则跳过。`yid` 仅日志对照。
fn trigger_ha_meltdown(
    state: &AppState,
    group_aid: &str,
    status_code: u16,
    error_message: &str,
    yid: Option<&str>,
) {
    if !is_ha_aid(group_aid) {
        return;
    }

    use std::sync::atomic::Ordering;
    use std::time::Duration;

    let yid = yid_label(yid);

    if !error_message.is_empty() {
        if let Ok(whitelist) = state.ha_meltdown_whitelist.read() {
            let err_lower = error_message.to_lowercase();
            for pattern in whitelist.iter() {
                if !pattern.is_empty() && err_lower.contains(pattern.as_str()) {
                    tracing::info!(
                        "[HA] 白名单跳过熔断 上游YID={} 子渠标识={} 关键词={}",
                        yid,
                        group_aid,
                        pattern
                    );
                    return;
                }
            }
        }
    }

    let cooldown = match status_code {
        429 => state.ha_cooldown_429.load(Ordering::Relaxed),
        401 | 402 => state.ha_cooldown_auth.load(Ordering::Relaxed),
        404 => state.ha_cooldown_404.load(Ordering::Relaxed),
        _ => state.ha_cooldown_network.load(Ordering::Relaxed),
    };

    if cooldown > 0 {
        let block_until = Instant::now() + Duration::from_secs(cooldown.max(0) as u64);
        state
            .failed_channels
            .insert(group_aid.to_string(), block_until);
        tracing::info!(
            "[HA] 熔断 上游YID={} 子渠标识={} 状态码={} 冷却={}秒",
            yid,
            group_aid,
            status_code,
            cooldown
        );
    }

    static CLEANUP_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = CLEANUP_COUNTER.fetch_add(1, Ordering::Relaxed);
    if n % 32 == 0 || state.failed_channels.len() > 2048 {
        scrub_failed_channels(state);
    }
}
