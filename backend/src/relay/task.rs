/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

//! Relay: 通用异步任务轮询网关 + 后台定时轮询器
//! 统一处理视频、图片等带有 task_id 的异步模型轮询和计费结算。
//!
//! 路由入口：
//!   - GET /v1/video/generations/{task_id} — 标准 OpenAI 视频轮询地址（主要入口）
//!   - GET /v1/tasks/{task_id}            — 兼容 apimart 等图片模型的异步轮询地址
//!   两者均执行本文件的 task_status 函数，逻辑完全一致。
//!
//! 后台定时器按 [`POLL_TICK_INTERVAL_SECS`] 自动检查未完成计费的异步任务，确保计费正确落地。

use super::cascade::{
    cascade_combine_stages, cascade_format_s2_succeeded, cascade_is_combined_resp,
    cascade_on_s2_succeeded, cascade_resolve_s2_poll, cascade_s1_upstream_task_id,
    cascade_s2_client_processing, cascade_scrub_plugin_tag_for_user, cascade_stage2_err_text,
    cascade_stage2_submit, cascade_stage_num, CascadeMk, CascadeS2SubmitOutcome,
};
use super::response_formatter::{
    force_json_task_id, format_async_task_failed, is_failed_task_status,
};
use super::url_utils::join_url;
use super::{forward, proxy};
use crate::models::ApiToken;
use crate::{
    error::{AppError, AppResult},
    AppState,
};
use axum::{
    extract::{Extension, OriginalUri, Path, Query, State},
    response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;

/// 异步任务轮询 / 后台同步共用的日志快照。
/// 用 `FromRow` 替代元组，不受 sqlx 元组 ≤16 列限制；后续加字段只改本结构 + SELECT。
#[derive(Debug, Clone, sqlx::FromRow)]
struct TaskRelayLogRow {
    id: i64,
    channel_id: i64,
    model: String,
    response_content: String,
    request_content: String,
    endpoint: String,
    action_type: String,
    plugin_tag: String,
    upstream_req_content: String,
    post_response: String,
    is_completed: i16,
    status_code: i32,
    cost: f64,
    pre_deduct_gift: f64,
    channel_config_id: Option<i32>,
    task_id: String,
    user_id: String,
    #[sqlx(default)]
    token_id: Option<i64>,
}

/// 与 [`TaskRelayLogRow`] 字段一一对应（COALESCE + AS 保证命名映射与空串语义）
const TASK_RELAY_LOG_COLS: &str = "\
id, channel_id, model, \
COALESCE(response_content, '') AS response_content, \
COALESCE(request_content, '') AS request_content, \
COALESCE(endpoint, '') AS endpoint, \
COALESCE(action_type, '') AS action_type, \
COALESCE(plugin_tag, '') AS plugin_tag, \
COALESCE(upstream_req_content, '') AS upstream_req_content, \
COALESCE(post_response, '') AS post_response, \
is_completed, status_code, cost, pre_deduct_gift, channel_config_id, \
COALESCE(task_id, '') AS task_id, \
user_id, token_id";

#[inline]
fn format_task_relay_sql(state: &AppState, where_clause: &str) -> String {
    state.db.format_query(&format!(
        "SELECT {TASK_RELAY_LOG_COLS} FROM logs WHERE {where_clause}"
    ))
}

/// 从 logs.plugin_tag 解析插件实际模型（用于轮询时模型替换）
fn resolve_plugin_model(plugin_tag: &str) -> Option<String> {
    if !plugin_tag.contains("happyhorse") {
        return None;
    }
    let tag: serde_json::Value = serde_json::from_str(plugin_tag).ok()?;
    tag["actual_model"].as_str().map(|s| s.to_string())
}

/// 级联 stage2 无任务 ID：落失败体 + 退费结案（GET / 后台共用）
async fn settle_cascade_s2_no_task_id(
    state: &AppState,
    log_id: i64,
    user_task_id: &str,
    err_text: &str,
) {
    let fail_body = super::response_formatter::async_task_failed_body(user_task_id, err_text);
    let _ = sqlx::query(
        &state
            .db
            .format_query("UPDATE logs SET response_content = ?, error_message = ? WHERE id = ?"),
    )
    .bind(&fail_body)
    .bind(err_text)
    .bind(log_id)
    .execute(&state.db.pool)
    .await;
    settle_failure(state, log_id, err_text, 500, 2).await;
}

/// 结案失败对外体：已有 `status:failed` 则保留并固定 id；否则补齐约定体
fn ensure_client_async_failed(
    raw_path: &str,
    category: &str,
    task_id: &str,
    formatted: &str,
    err_src: &str,
) -> String {
    if is_failed_task_status(formatted) {
        let mut s = formatted.to_string();
        force_json_task_id(&mut s, task_id);
        return s;
    }
    let err = proxy::extract_error_message(err_src);
    format_async_task_failed(
        raw_path,
        category,
        task_id,
        if err.is_empty() {
            "任务已失败"
        } else {
            &err
        },
    )
}

/// 选择实际轮询目标：阶段二用增强渠，否则用原渠道
fn pick_poll_target<'a>(
    s2_poll: &'a Option<(
        crate::models::Channel,
        String,
        super::forward::ResolvedForward,
        String,
    )>,
    channel: &'a crate::models::Channel,
    resolved: &'a super::forward::ResolvedForward,
    task_id: &'a str,
    model_name: &'a str,
) -> (
    &'a crate::models::Channel,
    std::borrow::Cow<'a, super::forward::ResolvedForward>,
    &'a str,
    &'a str,
) {
    if let Some((ch, s2_id, s2_resolved, fm)) = s2_poll {
        (
            ch,
            std::borrow::Cow::Borrowed(s2_resolved),
            s2_id.as_str(),
            fm.as_str(),
        )
    } else {
        (
            channel,
            std::borrow::Cow::Borrowed(resolved),
            task_id,
            model_name,
        )
    }
}

/// 构造即梦轮询上下文：优先使用日志中的原始请求内容，enable_log=0 时从 plugin_tag.jimeng_poll 恢复
fn build_jimeng_poll_ctx<'a>(
    target_type: &str,
    log_upstream_req: &'a str,
    log_request_content: &'a str,
    plugin_tag: &str,
    fallback_buf: &'a mut Option<String>,
) -> Option<(&'a str, &'a str)> {
    if !target_type.starts_with("jimeng_") {
        return None;
    }
    let req: &str = if log_request_content.is_empty() {
        *fallback_buf = serde_json::from_str::<serde_json::Value>(plugin_tag)
            .ok()
            .and_then(|pt| pt.get("jimeng_poll").map(|jp| jp.to_string()));
        fallback_buf.as_deref().unwrap_or("")
    } else {
        log_request_content
    };
    let upstream = if log_upstream_req.is_empty() {
        ""
    } else {
        log_upstream_req
    };
    Some((upstream, req))
}

/// 类别推断：优先 action_type（POST 阶段精准写入），兜底 endpoint 推断，最后查 DB
async fn infer_category(
    pool: &sqlx::PgPool,
    db: &crate::db::Database,
    action_type: &str,
    endpoint: &str,
    model: &str,
) -> String {
    if !action_type.is_empty() {
        return action_type.to_string();
    }
    if let Some(cat) = super::proxy::action_type_from_path(endpoint) {
        return cat.to_string();
    }
    sqlx::query_scalar(&db.format_query(
        "SELECT COALESCE(t.name, '') FROM models m \
             LEFT JOIN model_types t ON m.type_id = t.id \
             WHERE m.model_id = ? ORDER BY m.id LIMIT 1",
    ))
    .bind(model)
    .fetch_optional(pool)
    .await
    .unwrap_or(None)
    .unwrap_or_default()
}

/// 类别到默认入口路径的映射
fn category_to_entry_path(category: &str) -> &'static str {
    match category {
        "视频" | "视频增强" => "/v1/video/generations",
        "图片" => "/v1/images/generations",
        _ => "/v1/tasks",
    }
}

// ── GET /v1/video/generations/{task_id} | /v1/tasks/{task_id} ──

/// 通用异步任务状态查询（视频/图片/其他）
/// 标准调用地址: GET /v1/video/generations/{task_id}?model=xxx
/// 兼容地址:     GET /v1/tasks/{task_id}?model=xxx（apimart 图片异步查询）
pub async fn task_status(
    State(state): State<Arc<AppState>>,
    Extension(token): Extension<ApiToken>,
    OriginalUri(uri): OriginalUri,
    Path(task_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Response> {
    let raw_path = uri.path();
    let mut model_name = params
        .get("model")
        .map(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    // 从日志中查找原始渠道信息（含 action_type 用于精准类别推断，is_completed 用于快速返回判断）
    let log_row = sqlx::query_as::<_, TaskRelayLogRow>(&format_task_relay_sql(
        &state,
        "task_id = ? ORDER BY id DESC LIMIT 1",
    ))
    .bind(&task_id)
    .fetch_optional(&state.db.pool)
    .await
    .ok()
    .flatten();

    let (
        db_log_id,
        log_channel_id,
        model_name_db,
        log_response_content,
        log_request_content,
        log_endpoint,
        log_action_type,
        log_plugin_tag,
        log_upstream_req,
        log_post_response,
        log_is_completed,
        log_status_code,
        log_cost,
        log_pre_deduct_gift,
        log_cfg_id,
    ) = match log_row {
        Some(r) => (
            Some(r.id),
            r.channel_id,
            r.model,
            r.response_content,
            r.request_content,
            r.endpoint,
            r.action_type,
            r.plugin_tag,
            r.upstream_req_content,
            r.post_response,
            r.is_completed,
            r.status_code,
            r.cost,
            r.pre_deduct_gift,
            r.channel_config_id,
        ),
        None => (
            None,
            0,
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            0i16,
            200i32,
            0.0,
            0.0,
            None,
        ),
    };
    // 日志表的 model 优先（保证数据一致性），仅日志为空时使用请求参数的 model
    if !model_name_db.is_empty() {
        model_name = model_name_db;
    }
    let log_response_content: Option<String> = if log_response_content.is_empty() {
        None
    } else {
        Some(log_response_content)
    };

    // 💡 已完成任务快速返回（前置优化：非级联场景无需查询渠道/规则，直接返回缓存）
    if log_is_completed == 1 {
        if let Some(ref content) = log_response_content {
            let cached_json: serde_json::Value =
                serde_json::from_str(content).unwrap_or(serde_json::json!({}));
            if cached_json.is_object() && !cached_json.as_object().map_or(true, |m| m.is_empty()) {
                // 通过 response_content 结构判断是否为级联（有 stage1+stage2 = 级联）
                let is_cascade_resp = cascade_is_combined_resp(&cached_json);
                let category = infer_category(
                    &state.db.pool,
                    &state.db,
                    &log_action_type,
                    &log_endpoint,
                    &model_name,
                )
                .await;
                let final_response_str = if is_cascade_resp && log_status_code == 200 {
                    let s1 = cached_json
                        .get("stage1")
                        .cloned()
                        .unwrap_or(serde_json::json!({}));
                    let s2 = cached_json
                        .get("stage2")
                        .cloned()
                        .unwrap_or(serde_json::json!({}));
                    cascade_format_s2_succeeded(
                        raw_path,
                        &category,
                        &log_plugin_tag,
                        &s1,
                        &s2,
                        &task_id,
                    )
                } else if is_cascade_resp {
                    let err_text = cascade_stage2_err_text(
                        cached_json
                            .get("stage2")
                            .unwrap_or(&serde_json::Value::Null),
                        "cascade enhance failed",
                    );
                    format_async_task_failed(raw_path, &category, &task_id, &err_text)
                } else {
                    let formatted = crate::relay::response_formatter::apply_format(
                        raw_path,
                        &category,
                        content,
                        true,
                        Some(&task_id),
                    );
                    // 业务失败结案：确保对外 status:failed（旧缓存可能只有纯 error）
                    let formatted = if log_status_code != 200 {
                        ensure_client_async_failed(
                            raw_path, &category, &task_id, &formatted, content,
                        )
                    } else {
                        formatted
                    };
                    if category.contains("图片") {
                        let rf = serde_json::from_str::<serde_json::Value>(&log_request_content)
                            .ok()
                            .and_then(|v| {
                                v.get("response_format")
                                    .and_then(|f| f.as_str())
                                    .map(|s| s.to_string())
                            });
                        super::tos_persist::align_response_format(&state, &formatted, rf.as_deref())
                            .await
                    } else {
                        formatted
                    }
                };
                tracing::info!(
                    "[TaskPoll] 任务ID={}, 已完成=1, 直接返回缓存响应, 状态码={}",
                    task_id,
                    log_status_code
                );
                return Ok(Response::builder()
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(final_response_str))
                    .unwrap());
            }
        }
        // 失败已结案且无有效缓存：仍 200 + status:failed（禁止降级上游再结算）
        if log_status_code != 200 {
            let msg = log_response_content
                .as_deref()
                .filter(|s| !s.is_empty())
                .map(proxy::extract_error_message)
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "任务已失败".to_string());
            let category = infer_category(
                &state.db.pool,
                &state.db,
                &log_action_type,
                &log_endpoint,
                &model_name,
            )
            .await;
            let body = format_async_task_failed(raw_path, &category, &task_id, &msg);
            return Ok(Response::builder()
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(body))
                .unwrap());
        }
        tracing::warn!(
            "[TaskPoll] 任务ID={}, 已完成=1 但 response_content 无效，降级轮询上游",
            task_id
        );
    }

    if model_name.is_empty() {
        return Err(AppError::BadRequest(
            "Missing model parameter and cannot infer from task_id".to_string(),
        ));
    }

    // Plugin: happyhorse_router — 从 plugin_tag 解析实际模型
    if let Some(actual) = resolve_plugin_model(&log_plugin_tag) {
        tracing::info!("[小马] 轮询模型替换: {} → {}", model_name, actual);
        model_name = actual;
    }

    // 与选渠同源水合（channel_config_id 还原 HA 子配）；无日志时 channel_id=0 → None
    let channel = super::router::fetch_channel(&state, log_channel_id, log_cfg_id)
        .await
        .ok_or_else(|| {
            AppError::BadRequest("任务对应的渠道不存在或已被删除，无法查询任务状态".to_string())
        })?;

    let category = infer_category(
        &state.db.pool,
        &state.db,
        &log_action_type,
        &log_endpoint,
        &model_name,
    )
    .await;
    let default_entry = category_to_entry_path(&category);

    // 一次性查询模型数据，供转发规则解析和计费结算共同复用（避免两次 models 表查询）
    let cat_hint = if category.is_empty() {
        None
    } else {
        Some(category.as_str())
    };
    let db_model =
        super::proxy::find_active_model_exact(&state, &model_name, cat_hint, Some(&channel)).await;

    // 根据渠道绑定的转发规则解析实际物理路径（复用已查询的 model）
    let resolved = match forward::resolve_forward_rule(
        &state,
        &model_name,
        &category,
        default_entry,
        Some(&channel),
        db_model.as_ref(),
    )
    .await
    {
        Some(r) => r,
        None => forward::infer_forward_from_base_url(&channel.base_url, &category, None),
    };

    // 级联阶段判定：cascade_stage: 0=非级联, 1=阶段一, 2=阶段二
    let post_resp_json: serde_json::Value = if resolved.is_cascade {
        serde_json::from_str(&log_post_response).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    let cascade_stage: u8 = cascade_stage_num(resolved.is_cascade, &post_resp_json);

    // 💡 级联阶段二：替换轮询目标（独立增强渠道 + stage2 任务ID）
    let s2_poll = match cascade_resolve_s2_poll(
        cascade_stage,
        &post_resp_json,
        &channel,
        &resolved,
        &log_plugin_tag,
    ) {
        Ok(v) => v,
        Err(err_text) => {
            if let Some(id) = db_log_id {
                settle_cascade_s2_no_task_id(&state, id, &task_id, &err_text).await;
            }
            let final_err = format_async_task_failed(raw_path, &category, &task_id, &err_text);
            return Ok(Response::builder()
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(final_err))
                .unwrap());
        }
    };

    let s1_poll_id = cascade_s1_upstream_task_id(&log_plugin_tag, &task_id);
    let (poll_channel, poll_resolved, poll_task_id, poll_model) =
        pick_poll_target(&s2_poll, &channel, &resolved, &s1_poll_id, &model_name);

    let is_tencent = resolved.target_type.starts_with("tencent_vod");

    // 构造轮询上下文（即梦等需要额外参数的厂商，enable_log=0 时从 plugin_tag 恢复）
    let mut jimeng_fb = None;
    let jimeng_ctx = build_jimeng_poll_ctx(
        &resolved.target_type,
        &log_upstream_req,
        &log_request_content,
        &log_plugin_tag,
        &mut jimeng_fb,
    );
    let (url, get_resp_str) = match send_poll_request(
        &state.http_client,
        poll_channel,
        &poll_resolved,
        poll_task_id,
        poll_model,
        jimeng_ctx,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            // 结构化 status，与后台同一套临时/终态判定
            let (status, retryable) = e.classify();
            if !retryable {
                if let Some(id) = db_log_id {
                    refund_poll_terminal(&state, id, cascade_stage, &e.message, status).await;
                }
            }
            return Err(proxy::upstream_fail(status, &e.message, None));
        }
    };
    if let Some(id) = db_log_id {
        clear_poll_fail(&state, id).await;
    }
    // 与自动轮询一致：腾讯云每次轮询都落原始响应，便于对照终态结算字段
    if is_tencent {
        log_tencent_poll_raw("Task Poll", &task_id, &get_resp_str);
    }
    let resp_json: serde_json::Value =
        serde_json::from_str(&get_resp_str).unwrap_or(serde_json::json!({}));

    // 提前解析任务状态，决定是否需要 TOS 替换
    let raw_status = super::response_formatter::extract_raw_status(&resp_json);
    let task_status = normalize_task_status(&raw_status);
    tracing::info!(
        "[TaskPoll] 任务ID={}, 模型={}, 类别={}, 状态={}, 级联阶段={}, 响应长度={}",
        task_id,
        model_name,
        category,
        task_status,
        cascade_stage,
        get_resp_str.len()
    );

    // 💡 级联阶段一特殊处理：succeeded 触发阶段二提交, failed/pending 继续走主流程
    if cascade_stage == 1 {
        if task_status == "succeeded" {
            let base_video_url = super::response_formatter::find_urls(&resp_json)
                .into_iter()
                .next()
                .unwrap_or_default();
            tracing::info!("[Cascade S1] 底座成功，触发阶段二 任务ID={}", task_id);
            if let Some(log_id) = db_log_id {
                match cascade_stage2_submit(
                    &state,
                    &token.user_id,
                    Some(token.id),
                    &task_id,
                    log_id,
                    &log_post_response,
                    &log_request_content,
                    &log_upstream_req,
                    log_cost,
                    log_pre_deduct_gift,
                    &channel,
                    &base_video_url,
                    &log_plugin_tag,
                    &get_resp_str,
                    resolved.crop_480p,
                )
                .await
                {
                    Ok(
                        CascadeS2SubmitOutcome::Submitted(_) | CascadeS2SubmitOutcome::InProgress,
                    ) => {
                        // 内存中仍是原始 POST ack（DB 已写为 {stage1,stage2}）；不可再取 .stage1（旧结构无此键 → {}）
                        let stage1_ack =
                            serde_json::from_str::<serde_json::Value>(&log_post_response)
                                .unwrap_or(serde_json::json!({}));
                        let final_resp = cascade_s2_client_processing(
                            raw_path,
                            &category,
                            &stage1_ack,
                            &task_id,
                        );
                        return Ok(Response::builder()
                            .header("Content-Type", "application/json")
                            .body(axum::body::Body::from(final_resp))
                            .unwrap());
                    }
                    Err(e) => {
                        // submit 内已退费结案：对外仍 200 + status:failed（勿 502 纯 error）
                        let final_resp =
                            format_async_task_failed(raw_path, &category, &task_id, &e);
                        return Ok(Response::builder()
                            .header("Content-Type", "application/json")
                            .body(axum::body::Body::from(final_resp))
                            .unwrap());
                    }
                }
            }
        }
        // pending/failed → 继续走主流程（计费/退款条件不满足会自动跳过）
    }

    // 腾讯云：统一转为 OpenAI 格式；非腾讯：保持原始格式
    let mut store_body = if is_tencent {
        super::response_formatter::format_openai(&category, &get_resp_str, true, Some(&task_id))
    } else {
        get_resp_str.clone()
    };
    // 级联 S1：落库体同步对外 cgt（防中间态 response_content 残留上游 id/task_id）
    if resolved.is_cascade && cascade_stage == 1 {
        force_json_task_id(&mut store_body, &task_id);
    }

    let rf = serde_json::from_str::<serde_json::Value>(&log_request_content)
        .ok()
        .and_then(|v| {
            v.get("response_format")
                .and_then(|f| f.as_str())
                .map(|s| s.to_string())
        });
    let rf_ref = rf.as_deref();

    // 级联阶段二：提取 stage1；成功时先抽尾帧再 TOS（抽帧需上游可访问的视频 URL）
    let mut s1_json: serde_json::Value = if cascade_stage == 2 {
        if let Some(ref content) = log_response_content {
            let parsed: serde_json::Value =
                serde_json::from_str(content).unwrap_or(serde_json::json!({}));
            parsed.get("stage1").cloned().unwrap_or(parsed)
        } else {
            serde_json::json!({})
        }
    } else {
        serde_json::json!({})
    };

    if cascade_stage == 2 && task_status == "succeeded" {
        cascade_on_s2_succeeded(
            &CascadeMk {
                http: &state.http_client,
                ch: poll_channel,
                auth_type: &poll_resolved.auth_type,
            },
            &mut s1_json,
            &mut store_body,
            &resolved.res_mul,
            &log_plugin_tag,
        )
        .await;
    }

    // 渠道 TOS 存储：仅在非级联阶段一 且 任务成功时执行
    let store_body = if task_status == "succeeded" && cascade_stage != 1 {
        if let Some(days) = channel.tos_storage() {
            let fallback_type = if category.contains("视频") {
                "video"
            } else {
                "image"
            };
            super::tos_persist::persist_response_resources(
                &state,
                &store_body,
                channel.id,
                days,
                rf_ref,
                Some(fallback_type),
            )
            .await
        } else {
            store_body
        }
    } else {
        store_body
    };

    let store_body = if cascade_stage == 2 && task_status == "succeeded" {
        cascade_combine_stages(&s1_json, &store_body)
    } else {
        store_body
    };

    if let Some(log_id) = db_log_id {
        // 清理级联 plugin_tag 中的敏感信息
        if task_status == "succeeded" || task_status == "failed" {
            let mut tag = Some(log_plugin_tag.clone());
            if cascade_scrub_plugin_tag_for_user(&mut tag) {
                if let Some(updated_tag) = tag {
                    let _ = sqlx::query(
                        &state
                            .db
                            .format_query("UPDATE logs SET plugin_tag = ? WHERE id = ?"),
                    )
                    .bind(&updated_tag)
                    .bind(log_id)
                    .execute(&state.db.pool)
                    .await;
                }
            }
        }

        if task_status == "succeeded" {
            // 先结算再落库：避免结算失败重试时对已写入的倍率 usage 再次 × res_mul
            settle_success(
                &state,
                log_id,
                &model_name,
                &store_body,
                &resp_json,
                &url,
                &category,
                &channel,
                cascade_stage,
                &log_plugin_tag,
                db_model.as_ref(),
                &resolved.res_mul,
            )
            .await;
            crate::services::notification::spawn_low_balance_check(
                Arc::clone(&state),
                token.user_id.clone(),
            );
            let _ = sqlx::query(&state.db.format_query(
                "UPDATE logs SET response_content = ?, error_message = NULL WHERE id = ?",
            ))
            .bind(&store_body)
            .bind(log_id)
            .execute(&state.db.pool)
            .await;
            tracing::info!(
                "[TaskBilling] 日志ID={}, 模型={}, 级联阶段={}, URL={}",
                log_id,
                model_name,
                cascade_stage,
                url
            );
        } else if task_status == "failed" {
            let err_text = proxy::extract_error_message(&store_body);
            if cascade_stage == 2 {
                tracing::warn!(
                    "[Cascade S2] 画质增强失败: 日志ID={}, 错误={}",
                    log_id,
                    err_text
                );
                let updated = serde_json::json!({
                    "stage1": post_resp_json["stage1"],
                    "stage2": &err_text
                })
                .to_string();
                let resp_content = cascade_combine_stages(&s1_json, &store_body);
                let _ = sqlx::query(&state.db.format_query("UPDATE logs SET response_content = ?, error_message = ?, post_response = ? WHERE id = ?"))
                    .bind(&resp_content).bind(&err_text).bind(&updated).bind(log_id)
                    .execute(&state.db.pool).await;
            } else {
                let _ = sqlx::query(&state.db.format_query(
                    "UPDATE logs SET response_content = ?, error_message = ? WHERE id = ?",
                ))
                .bind(&store_body)
                .bind(&err_text)
                .bind(log_id)
                .execute(&state.db.pool)
                .await;
            }
            let status_code = proxy::infer_error_status_code_from_str(&store_body);
            settle_failure(&state, log_id, &url, status_code, cascade_stage).await;
            tracing::info!(
                "[TaskRefund] 日志ID={}, 模型={}, 级联阶段={}, URL={}, 状态码={}",
                log_id,
                model_name,
                cascade_stage,
                url,
                status_code
            );
        } else {
            let db_store_body = if cascade_stage == 2 {
                cascade_combine_stages(&s1_json, &store_body)
            } else {
                store_body.clone()
            };
            let _ = sqlx::query(
                &state
                    .db
                    .format_query("UPDATE logs SET response_content = ? WHERE id = ?"),
            )
            .bind(&db_store_body)
            .bind(log_id)
            .execute(&state.db.pool)
            .await;
        }
    }

    // 返回格式化：
    // - 级联 S2 成功：S1 骨架 + S2 产物 URL
    // - 级联 S2 失败 / 非级联失败：HTTP 200 + status:failed
    // - 级联 S2 进行中：阶段一 POST 处理中形态
    // - 其余：腾讯已是 OpenAI；其它走 apply_format
    let final_response_str = if cascade_stage == 2 && task_status == "succeeded" {
        let resp_json: serde_json::Value =
            serde_json::from_str(&store_body).unwrap_or(serde_json::json!({}));
        let s1 = resp_json
            .get("stage1")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        let s2 = resp_json
            .get("stage2")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        cascade_format_s2_succeeded(raw_path, &category, &log_plugin_tag, &s1, &s2, &task_id)
    } else if cascade_stage == 2 && task_status == "failed" {
        // 不向客户端暴露 S2 原始失败体
        format_async_task_failed(
            raw_path,
            &category,
            &task_id,
            &proxy::extract_error_message(&store_body),
        )
    } else if cascade_stage == 2 {
        let stage1_ack = post_resp_json
            .get("stage1")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        cascade_s2_client_processing(raw_path, &category, &stage1_ack, &task_id)
    } else if task_status == "failed" {
        // live 结案失败也必须带 status:failed（与缓存路径一致）
        let formatted = if is_tencent {
            store_body
        } else {
            crate::relay::response_formatter::apply_format(
                raw_path,
                &category,
                &store_body,
                true,
                Some(&task_id),
            )
        };
        ensure_client_async_failed(raw_path, &category, &task_id, &formatted, &formatted)
    } else if is_tencent {
        store_body
    } else {
        crate::relay::response_formatter::apply_format(
            raw_path,
            &category,
            &store_body,
            true,
            Some(&task_id),
        )
    };

    // 仅对图片模型进行双向格式对齐，视频模型只返回 URL，跳过以避免大 JSON 反序列化开销
    let mut final_response_str = if category.contains("图片") {
        super::tos_persist::align_response_format(&state, &final_response_str, rf_ref).await
    } else {
        final_response_str
    };
    // 级联对外统一 cgt（防上游 poll 体 id 泄漏）
    if resolved.is_cascade {
        force_json_task_id(&mut final_response_str, &task_id);
    }

    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(final_response_str))
        .unwrap())
}

// ── 后台定时轮询器 ──────────────────────────────────────────────

/// 后台轮询周期（秒）。过短增加上游压力，过长延迟结案/退费；客户端主动 GET 仍即时。
const POLL_TICK_INTERVAL_SECS: u64 = 30;

/// 启动后台轮询定时任务（支持优雅关闭：收到 shutdown 信号后完成当前轮询再退出）
pub fn start(
    state: Arc<AppState>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        // 启动后等待 30 秒再开始第一次轮询，让系统初始化完毕
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {},
            _ = shutdown.changed() => {
                tracing::info!("[TaskPoller] 初始化期间收到关闭信号，退出");
                return;
            }
        }
        loop {
            if let Err(e) = poll_pending_tasks(&state).await {
                tracing::error!("[TaskPoller] 轮询异常: {}", e);
            }
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(POLL_TICK_INTERVAL_SECS)) => {},
                _ = shutdown.changed() => {
                    tracing::info!("[TaskPoller] 收到关闭信号，退出轮询");
                    return;
                }
            }
        }
    })
}

/// 活跃轮询窗口与单批大小；超窗外仍冻结的任务走 `refund_stale_freezes` 兜底退款。
const POLL_ACTIVE_INTERVAL: &str = "2 days";
const POLL_BATCH: i64 = 100;
const POLL_MAX_BATCHES_PER_TICK: u32 = 5;
/// `logs.latency_ms` 为 INT4；超龄冻结直接 CAST 会 22003，先按 bigint 钳到上限。
const LATENCY_MS_SQL: &str = "LEAST(2147483647, GREATEST(0, \
    (EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - created_at)) * 1000)::bigint))::integer";

/// 连续请求失败上限（任务状态轮询 / 后台 TaskPoller 共用）
/// 取 15：上游短暂抖动时多给几次机会，避免过早退款/放弃而漏掉终态成功。
const POLL_FAIL_LIMIT: u32 = 15;

/// 查询前倒序休眠 5→4→3→2→此后 1s；若休眠会越过 `deadline` 则返回 false。
async fn poll_wait_before_query(attempt: u32, deadline: tokio::time::Instant) -> bool {
    let delay = 6u64.saturating_sub(attempt.max(1) as u64).max(1);
    if tokio::time::Instant::now() + std::time::Duration::from_secs(delay) > deadline {
        return false;
    }
    tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
    true
}

fn pending_freeze_filter(cmp: &str) -> String {
    format!(
        "is_completed = 0 AND status_code = 200 \
         AND created_at {cmp} CURRENT_TIMESTAMP - INTERVAL '{interval}'",
        cmp = cmp,
        interval = POLL_ACTIVE_INTERVAL
    )
}

/// 查询未结算异步任务并轮询上游；连续失败达到 [`POLL_FAIL_LIMIT`] 则退款终结。
/// 按 id ASC 分批，避免 DESC+LIMIT 饿死旧任务；窗口外僵死冻结单独退款。
async fn poll_pending_tasks(state: &Arc<AppState>) -> anyhow::Result<()> {
    refund_stale_freezes(state).await;

    let filter = pending_freeze_filter(">");
    for _ in 0..POLL_MAX_BATCHES_PER_TICK {
        let rows: Vec<(i64, String, Option<String>, String)> =
            sqlx::query_as(&state.db.format_query(&format!(
                "SELECT id, model, error_message, COALESCE(task_id, '') FROM logs \
             WHERE {filter} ORDER BY id ASC LIMIT ?",
                filter = filter
            )))
            .bind(POLL_BATCH)
            .fetch_all(&state.db.pool)
            .await?;

        if rows.is_empty() {
            break;
        }
        let batch_len = rows.len();
        tracing::info!("[TaskPoller] 本批待轮询 {} 条", batch_len);

        for (log_id, model, error_message, db_task_id) in rows {
            let (prev_fail_count, last_fail_status) =
                parse_poll_fail_meta(error_message.as_deref());
            if prev_fail_count >= POLL_FAIL_LIMIT {
                // 已达上限但仍在冻结队列：补退费（CAS 幂等，防上次 settle 失败后永久挂住）
                settle_failure(
                    state,
                    log_id,
                    "auto_poll_fail:limit_pending",
                    last_fail_status,
                    0,
                )
                .await;
                continue;
            }
            if db_task_id.is_empty() {
                // 正常冻结路径 task_id 非空；异常脏数据跳过，超窗由 stale 兜底
                tracing::warn!(
                    "[TaskPoller] 日志ID={}, 模型={} 日志中无 task_id，跳过本轮",
                    log_id,
                    model
                );
                continue;
            }

            tracing::info!(
                "[TaskPoller] 开始轮询 日志ID={}, 模型={}, 任务ID={}",
                log_id,
                model,
                db_task_id
            );
            if let Err(e) = sync_single_task(state, log_id).await {
                let raw = e.to_string();
                // 仅统计可重试轮询传输错；其它 anyhow（级联等）不计入，避免误退费
                let Some((status, err_msg)) = split_poll_retry_err(&raw) else {
                    tracing::warn!(
                        "[TaskPoller] 日志ID={} 轮询异常(不计入 POLL_FAIL): {}",
                        log_id,
                        raw
                    );
                    continue;
                };
                let fail_count = prev_fail_count + 1;
                tracing::warn!(
                    "[TaskPoller] 日志ID={} 自动轮询失败 ({}/{}): {}",
                    log_id,
                    fail_count,
                    POLL_FAIL_LIMIT,
                    err_msg
                );
                let _ = sqlx::query(
                    &state
                        .db
                        .format_query("UPDATE logs SET error_message = ? WHERE id = ?"),
                )
                .bind(&format_poll_fail_tag(fail_count, status, err_msg))
                .bind(log_id)
                .execute(&state.db.pool)
                .await;

                if fail_count >= POLL_FAIL_LIMIT {
                    settle_failure(
                        state,
                        log_id,
                        &format!("auto_poll_fail:{}", err_msg),
                        status,
                        0,
                    )
                    .await;
                    tracing::error!(
                        "[TaskPoller] 日志ID={} 连续 {} 次轮询失败，已终止并退款",
                        log_id,
                        fail_count
                    );
                }
            }
        }

        if (batch_len as i64) < POLL_BATCH {
            break;
        }
    }

    Ok(())
}

/// 超过活跃窗口仍未完成的冻结任务：退款终结，防止预扣永久挂住。
async fn refund_stale_freezes(state: &Arc<AppState>) {
    let filter = pending_freeze_filter("<=");
    let stale: Vec<i64> = match sqlx::query_scalar(&state.db.format_query(&format!(
        "SELECT id FROM logs WHERE {filter} ORDER BY id ASC LIMIT 50",
        filter = filter
    )))
    .fetch_all(&state.db.pool)
    .await
    {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!("[TaskPoller] 查询僵死冻结失败: {:?}", e);
            return;
        }
    };
    if stale.is_empty() {
        return;
    }
    tracing::warn!(
        "[TaskPoller] 发现 {} 条超过 {} 仍冻结，执行兜底退款",
        stale.len(),
        POLL_ACTIVE_INTERVAL
    );
    for log_id in stale {
        settle_failure(state, log_id, "stale_freeze_timeout", 408, 0).await;
    }
}

// ── sync_single_task ────────────────────────────────────────────

/// 执行单条任务的同步轮询（支持手动或定时调用，含完整级联支持）
pub async fn sync_single_task(state: &Arc<AppState>, log_id: i64) -> anyhow::Result<String> {
    let TaskRelayLogRow {
        channel_id,
        model: mut model_name,
        endpoint,
        action_type,
        plugin_tag,
        upstream_req_content: log_upstream_req,
        request_content: log_request_content,
        task_id,
        post_response: log_post_response,
        response_content: log_resp_content,
        is_completed,
        user_id,
        token_id,
        cost: log_cost,
        pre_deduct_gift: log_pre_deduct_gift,
        channel_config_id: log_cfg_id,
        ..
    } = sqlx::query_as::<_, TaskRelayLogRow>(&format_task_relay_sql(state, "id = ?"))
        .bind(log_id)
        .fetch_optional(&state.db.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("任务记录不存在"))?;

    // 已完成任务无需再轮询
    if is_completed == 1 {
        return Ok("任务已完成，无需轮询".to_string());
    }

    // Plugin: happyhorse_router — 若 plugin_tag 包含 happyhorse，则用实际模型替换 model_name 用于转发规则/计费
    if let Some(actual) = resolve_plugin_model(&plugin_tag) {
        tracing::info!("[小马轮询] 模型替换: {} → {}", model_name, actual);
        model_name = actual;
    }

    // task_id 直接从日志表获取，无需从 response_content 解析
    if task_id.is_empty() {
        return Err(anyhow::anyhow!("该记录无 task_id，可能不是异步任务"));
    }

    let channel = super::router::fetch_channel(state, channel_id, log_cfg_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("渠道不存在或已被删除"))?;

    let category = infer_category(
        &state.db.pool,
        &state.db,
        &action_type,
        &endpoint,
        &model_name,
    )
    .await;
    let entry_path = category_to_entry_path(&category);

    // 一次性查询模型数据，供转发规则解析和计费结算共同复用（避免两次 models 表查询）
    let cat_hint = if category.is_empty() {
        None
    } else {
        Some(category.as_str())
    };
    let db_model =
        super::proxy::find_active_model_exact(state, &model_name, cat_hint, Some(&channel)).await;

    // 根据渠道绑定的转发规则解析实际物理路径（复用已查询的 model）
    let resolved = forward::resolve_forward_rule(
        state,
        &model_name,
        &category,
        entry_path,
        Some(&channel),
        db_model.as_ref(),
    )
    .await
    .unwrap_or_else(|| forward::infer_forward_from_base_url(&channel.base_url, &category, None));

    // 💡 级联阶段判定（与 task_status 保持一致）
    let post_resp_json: serde_json::Value = if resolved.is_cascade {
        serde_json::from_str(&log_post_response).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    let cascade_stage: u8 = cascade_stage_num(resolved.is_cascade, &post_resp_json);

    // 💡 级联阶段二：替换轮询目标（增强渠道 + stage2 任务ID）
    let s2_poll = match cascade_resolve_s2_poll(
        cascade_stage,
        &post_resp_json,
        &channel,
        &resolved,
        &plugin_tag,
    ) {
        Ok(v) => v,
        Err(err_text) => {
            settle_cascade_s2_no_task_id(state, log_id, &task_id, &err_text).await;
            return Ok("级联阶段二失败: 无有效任务ID".to_string());
        }
    };

    let s1_poll_id = cascade_s1_upstream_task_id(&plugin_tag, &task_id);
    let (poll_channel, poll_resolved, poll_task_id, poll_model) =
        pick_poll_target(&s2_poll, &channel, &resolved, &s1_poll_id, &model_name);

    // 构造即梦轮询上下文（enable_log=0 时从 plugin_tag 恢复轮询参数）
    let mut jimeng_fb = None;
    let jimeng_ctx = build_jimeng_poll_ctx(
        &resolved.target_type,
        &log_upstream_req,
        &log_request_content,
        &plugin_tag,
        &mut jimeng_fb,
    );
    let (url, body) = match send_poll_request(
        &state.http_client,
        poll_channel,
        &poll_resolved,
        poll_task_id,
        poll_model,
        jimeng_ctx,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            // 结构化 status：临时错抛出计入 POLL_FAIL；终态错退费
            let (status, retryable) = e.classify();
            if retryable {
                return Err(anyhow::anyhow!("[poll:{}] {}", status, e.message));
            }
            refund_poll_terminal(state, log_id, cascade_stage, &e.message, status).await;
            return Ok(format!("任务终态失败（上游错误）: {}", e.message));
        }
    };
    // 轮询通道已通：清零 POLL_FAIL，避免恢复后仍被旧计数退费
    clear_poll_fail(state, log_id).await;

    let is_tencent = resolved.target_type.starts_with("tencent_vod");
    if is_tencent {
        log_tencent_poll_raw("TaskPoller", &task_id, &body);
    }
    let resp_json: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));

    let raw_status = super::response_formatter::extract_raw_status(&resp_json);
    let task_status = normalize_task_status(&raw_status);

    // 💡 级联阶段一：succeeded 触发阶段二提交
    if cascade_stage == 1 && task_status == "succeeded" {
        let base_video_url = super::response_formatter::find_urls(&resp_json)
            .into_iter()
            .next()
            .unwrap_or_default();
        tracing::info!("[Cascade S1 BG] 底座成功，触发阶段二 任务ID={}", task_id);
        match cascade_stage2_submit(
            state,
            &user_id,
            token_id,
            &task_id,
            log_id,
            &log_post_response,
            &log_request_content,
            &log_upstream_req,
            log_cost,
            log_pre_deduct_gift,
            &channel,
            &base_video_url,
            &plugin_tag,
            &body,
            resolved.crop_480p,
        )
        .await
        {
            Ok(CascadeS2SubmitOutcome::Submitted(stage2_id)) => {
                return Ok(format!("级联阶段二已提交，stage2_id={}", stage2_id));
            }
            Ok(CascadeS2SubmitOutcome::InProgress) => {
                return Ok("级联阶段二提交中（并发互斥）".to_string());
            }
            Err(e) => return Err(anyhow::anyhow!("{}", e)),
        }
    }

    if task_status != "succeeded" && task_status != "failed" {
        return Ok(format!("当前状态: {}", task_status));
    }

    // 腾讯云：构造 OpenAI 格式响应（复用 response_formatter 统一逻辑）
    let mut final_body = if is_tencent {
        let category_str = if category.contains("视频") {
            "视频"
        } else {
            "图片"
        };
        super::response_formatter::format_openai(&category_str, &body, true, Some(&task_id))
    } else {
        body.clone()
    };
    if resolved.is_cascade && cascade_stage == 1 {
        force_json_task_id(&mut final_body, &task_id);
    }

    // 级联阶段二：提取 stage1；成功时先抽尾帧再 TOS
    let mut s1_json: serde_json::Value = if cascade_stage == 2 && !log_resp_content.is_empty() {
        let parsed: serde_json::Value =
            serde_json::from_str(&log_resp_content).unwrap_or(serde_json::json!({}));
        parsed.get("stage1").cloned().unwrap_or(parsed)
    } else {
        serde_json::json!({})
    };

    if cascade_stage == 2 && task_status == "succeeded" {
        cascade_on_s2_succeeded(
            &CascadeMk {
                http: &state.http_client,
                ch: poll_channel,
                auth_type: &poll_resolved.auth_type,
            },
            &mut s1_json,
            &mut final_body,
            &resolved.res_mul,
            &plugin_tag,
        )
        .await;
    }

    let final_body = if task_status == "succeeded" {
        let rf = serde_json::from_str::<serde_json::Value>(&log_request_content)
            .ok()
            .and_then(|v| {
                v.get("response_format")
                    .and_then(|f| f.as_str())
                    .map(|s| s.to_string())
            });
        let rf_ref = rf.as_deref();

        // TOS 存储（级联阶段一已在上方 return，此处不会执行到）
        let body_after_tos = if let Some(days) = channel.tos_storage() {
            let fallback_type = if category.contains("视频") {
                "video"
            } else {
                "image"
            };
            super::tos_persist::persist_response_resources(
                state,
                &final_body,
                channel.id,
                days,
                rf_ref,
                Some(fallback_type),
            )
            .await
        } else {
            final_body
        };

        // 图片模型双向格式对齐
        let aligned = if category.contains("图片") {
            super::tos_persist::align_response_format(state, &body_after_tos, rf_ref).await
        } else {
            body_after_tos
        };

        if cascade_stage == 2 {
            cascade_combine_stages(&s1_json, &aligned)
        } else {
            aligned
        }
    } else {
        // 级联阶段二失败：更新 post_response.stage2 为错误文本
        if cascade_stage == 2 {
            let err_text = proxy::extract_error_message(&final_body);
            tracing::warn!(
                "[Cascade S2 BG] 画质增强失败: 日志ID={}, 错误={}",
                log_id,
                err_text
            );
            let updated = serde_json::json!({
                "stage1": post_resp_json["stage1"],
                "stage2": &err_text
            })
            .to_string();
            let resp_content = cascade_combine_stages(&s1_json, &final_body);
            let _ = sqlx::query(&state.db.format_query("UPDATE logs SET response_content = ?, error_message = ?, post_response = ? WHERE id = ?"))
                .bind(&resp_content).bind(&err_text).bind(&updated).bind(log_id)
                .execute(&state.db.pool).await;
        }
        final_body
    };

    // 失败路径：更新日志；成功路径延后到结算之后再写 response_content（防级联 usage 二次倍率）
    let inferred_status: u16 = if task_status == "succeeded" {
        200
    } else {
        let err_text = proxy::extract_error_message(&final_body);
        let status = proxy::infer_error_status_code_from_str(&final_body);
        if cascade_stage != 2 {
            let _ = sqlx::query(&state.db.format_query(
                "UPDATE logs SET response_content = ?, error_message = ? WHERE id = ?",
            ))
            .bind(&final_body)
            .bind(&err_text)
            .bind(log_id)
            .execute(&state.db.pool)
            .await;
        }
        status
    };

    // 清理级联 plugin_tag 中的敏感信息
    if task_status == "succeeded" || task_status == "failed" {
        let mut tag = Some(plugin_tag.clone());
        if cascade_scrub_plugin_tag_for_user(&mut tag) {
            if let Some(updated_tag) = tag {
                let _ = sqlx::query(
                    &state
                        .db
                        .format_query("UPDATE logs SET plugin_tag = ? WHERE id = ?"),
                )
                .bind(&updated_tag)
                .bind(log_id)
                .execute(&state.db.pool)
                .await;
            }
        }
    }

    if task_status == "succeeded" {
        settle_success(
            state,
            log_id,
            &model_name,
            &final_body,
            &resp_json,
            &url,
            &category,
            &channel,
            cascade_stage,
            &plugin_tag,
            db_model.as_ref(),
            &resolved.res_mul,
        )
        .await;
        if !user_id.is_empty() {
            crate::services::notification::spawn_low_balance_check(
                Arc::clone(state),
                user_id.clone(),
            );
        }
        let _ = sqlx::query(&state.db.format_query(
            "UPDATE logs SET response_content = ?, error_message = NULL WHERE id = ?",
        ))
        .bind(&final_body)
        .bind(log_id)
        .execute(&state.db.pool)
        .await;
        Ok("任务已成功落地并计费".to_string())
    } else {
        settle_failure(state, log_id, &url, inferred_status, cascade_stage).await;
        Ok("任务已失败，预扣费已退回".to_string())
    }
}

// ── 结算辅助函数 ────────────────────────────────────────────────

/// 任务成功：提取 token、计费、余额结算
/// cascade_stage: 级联阶段（0=非级联, 1=阶段一, 2=阶段二）
/// log_plugin_tag: 日志 plugin_tag，级联阶段二从 cascade.input_duration 获取预缓存的输入视频时长
/// res_mul: 级联分辨率倍率（stage2：有 tokens 则已乘入用量，否则乘费用）
async fn settle_success(
    state: &AppState,
    log_id: i64,
    model_name: &str,
    body: &str,
    resp_json: &serde_json::Value,
    poll_url: &str,
    category: &str,
    channel: &crate::models::Channel,
    cascade_stage: u8,
    log_plugin_tag: &str,
    caller_model: Option<&crate::models::Model>,
    res_mul: &std::collections::HashMap<String, f64>,
) {
    // 级联阶段二用量取自 stage1（成功路径 usage 已 × res_mul）
    let usage_str: String;
    let usage = if cascade_stage == 2 {
        let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::json!({}));
        let s1 = parsed.get("stage1").cloned().unwrap_or(parsed.clone());
        usage_str = s1.to_string();
        super::usage_extractor::parse_usage(&usage_str)
    } else {
        super::usage_extractor::parse_usage(body)
    };

    // 复用调用方已查询的 Model，避免重复查询 models 表
    let owned_model;
    let db_model: Option<&crate::models::Model> = if let Some(m) = caller_model {
        Some(m)
    } else {
        let cat_hint = if category.is_empty() {
            None
        } else {
            Some(category)
        };
        owned_model =
            super::proxy::find_active_model_exact(state, model_name, cat_hint, Some(channel)).await;
        owned_model.as_ref()
    };

    let mut db_rule =
        super::proxy::get_model_billing_rule(state, model_name, Some(&channel), db_model).await;

    // 获取原始预扣费、billing_detail、billing_features 及关联 ID（一次查询替代两次主键查询）
    let log_data: Option<(f64, f64, String, Option<i64>, Option<i64>, Option<String>, String)> = sqlx::query_as(
        &state.db.format_query("SELECT cost, pre_deduct_gift, user_id, token_id, channel_id, billing_detail, COALESCE(billing_features, '') FROM logs WHERE id = ?")
    ).bind(log_id).fetch_optional(&state.db.pool).await.unwrap_or(None);

    let (mut pre_deduction, mut pre_deduct_gift, uid, token_id, channel_id, b_detail, bf_str) =
        match log_data {
            Some(d) => d,
            None => (0.0, 0.0, "".to_string(), None, None, None, String::new()),
        };

    // 退款后重新成功：预扣费已退回用户，视为 0（全额从余额扣除）
    if b_detail.as_deref().map_or(false, |d| d.contains("退回")) {
        pre_deduction = 0.0;
        pre_deduct_gift = 0.0;
    } else {
        pre_deduction = crate::money::round_money(pre_deduction);
        pre_deduct_gift = crate::money::round_money(pre_deduct_gift);
    }
    let user_id = if uid.is_empty() { None } else { Some(uid) };

    // 获取用户折扣上下文（复用 get_user_context，避免重复拼装 discount 查询）
    let ctx = match user_id.as_deref() {
        Some(uid) => proxy::get_user_context(state, uid)
            .await
            .unwrap_or_else(|_| proxy::UserContext::from_discounts(1.0, 0, None)),
        None => proxy::UserContext::from_discounts(1.0, 0, None),
    };

    // 计费特征恢复：复用 build_poll_settlement_features 统一逻辑（内部已含 image_count 提取）
    let billing_features_str: Option<String> = if bf_str.is_empty() {
        None
    } else {
        Some(bf_str)
    };
    let mut features =
        build_poll_settlement_features(&billing_features_str, resp_json, body, category);

    // 级联阶段二：一次解析 plugin_tag，复用 cascade 节点（时长叠加）
    let plugin_tag_val = if cascade_stage == 2 {
        serde_json::from_str::<serde_json::Value>(log_plugin_tag).ok()
    } else {
        None
    };
    let cascade = plugin_tag_val.as_ref().and_then(|v| v.get("cascade"));
    if let Some(cascade) = cascade {
        let in_dur = cascade
            .get("input_duration")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        if in_dur > 0.0 {
            if let Some(dur) = features.duration_seconds.as_mut() {
                *dur += in_dur;
            } else {
                features.duration_seconds = Some(in_dur);
            }
        }
    }

    // 映射记录
    let (resolved_model, mapping_source) =
        crate::relay::router::resolve_model(channel, model_name, db_model);

    let (mut cost, mut detail) = super::calculate_relay_cost(
        state,
        db_model,
        db_rule.as_mut(),
        channel,
        &ctx,
        &usage,
        &features,
        mapping_source,
        model_name,
        &resolved_model,
    )
    .await;

    // 级联阶段二：有 P/C tokens 则倍率已在用量中；无则乘底座费用（分辨率优先用 cascade 原始目标）
    if cascade_stage == 2 {
        let has_tokens = usage.prompt > 0 || usage.completion > 0;
        if !has_tokens {
            let target_res = cascade
                .and_then(|c| c.get("resolution").and_then(|r| r.as_str()))
                .or(features.resolution.as_deref())
                .unwrap_or("720p");
            (cost, detail) = super::scale_cost_by_res_mul(cost, detail, res_mul, target_res);
        }
    }

    let final_uid = user_id.as_deref().unwrap_or("");
    let updated_bf = serde_json::to_string(&features).ok();
    execute_settlement_tx(
        state,
        log_id,
        final_uid,
        token_id,
        channel_id,
        usage.prompt,
        usage.completion,
        cost,
        pre_deduction,
        pre_deduct_gift,
        &detail,
        updated_bf.as_deref(),
    )
    .await;
    tracing::info!("[TaskPoller Billing] 日志ID={}, 模型={}, 费用={:.6}, 预扣={:.6}, Tokens={}+{}={}, 图片数={:?}, URL={}",
        log_id, model_name, cost, pre_deduction, usage.prompt, usage.completion, usage.total, features.image_count, poll_url);
}

/// 任务失败：按预扣费钱包来源精准退还
/// cascade_stage: 级联阶段（0=非级联, 2=阶段二）
async fn settle_failure(
    state: &AppState,
    log_id: i64,
    poll_url: &str,
    status_code: u16,
    cascade_stage: u8,
) {
    let log_data: Option<(f64, f64, String, Option<i64>, Option<i64>)> =
        sqlx::query_as(&state.db.format_query(
            "SELECT cost, pre_deduct_gift, user_id, token_id, channel_id FROM logs WHERE id = ?",
        ))
        .bind(log_id)
        .fetch_optional(&state.db.pool)
        .await
        .unwrap_or(None);

    let (pre_deduction, pre_deduct_gift, uid, token_id, channel_id) = match log_data {
        Some(d) => d,
        None => (0.0, 0.0, "".to_string(), None, None),
    };

    let detail = if cascade_stage == 2 {
        if pre_deduction > 0.0 {
            "画质增强失败，预扣费已退回"
        } else {
            "画质增强失败"
        }
    } else {
        if pre_deduction > 0.0 {
            "任务失败，预扣费已退回"
        } else {
            "任务失败，该请求无冻结费用"
        }
    };

    execute_refund_tx(
        state,
        log_id,
        &uid,
        token_id,
        channel_id,
        pre_deduction,
        pre_deduct_gift,
        detail,
        status_code,
    )
    .await;
    tracing::info!(
        "[TaskPoller Failure] 日志ID={}, 级联阶段={}, 退款={:.6}, URL={}, 状态码={}",
        log_id,
        cascade_stage,
        pre_deduction,
        poll_url,
        status_code
    );
}

// ── 公共结算工具函数 ────────────────────────────────────────────

/// 从提交响应 JSON 字符串中提取 task_id（复用 response_formatter::extract_async_task_id 统一搜索路径）
pub fn extract_task_id(response: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(response).ok()?;
    let id = super::response_formatter::extract_async_task_id(&v);
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

// ── 状态归一化 ──────────────────────────────────────────────────

/// 统一任务状态归一化：将各厂商返回的状态字符串映射为 succeeded / failed / 原值
pub(super) fn normalize_task_status(raw: &str) -> &str {
    let standard = super::response_formatter::parse_raw_status_to_standard(raw);
    match standard {
        "completed" => "succeeded",
        "failed" => "failed",
        "pending" => "pending",
        "in_progress" => "in_progress",
        _ => {
            if raw.is_empty() {
                "pending"
            } else {
                raw
            }
        }
    }
}

// ── 腾讯云转换函数 ──────────────────────────────────────────────

/// 腾讯云轮询原始响应日志（手动 / 自动 / 同步轮询共用，不影响业务路径）
fn log_tencent_poll_raw(source: &str, task_id: &str, body: &str) {
    tracing::info!(
        "[{}] 腾讯云原始响应 任务ID={}, 响应体={}",
        source,
        task_id,
        body
    );
}

/// 腾讯云 POST 响应统一转换：原始响应 → OpenAI 格式（复用 response_formatter::format_openai 统一逻辑）
/// 返回 (转换后的响应字符串, 是否为错误)
pub fn convert_tencent_post_response(raw_response: &str, category: &str) -> (String, bool) {
    let formatted = super::response_formatter::format_openai(category, raw_response, false, None);
    let is_error = formatted.contains("\"error\":");
    (formatted, is_error)
}

// ── 用户请求轮询共用的结算辅助函数 ──

/// 构建异步任务终态结算所需的计费特征
/// billing_features 快照为优先（POST 阶段保存），然后叠加终态响应中的实际数据
pub(super) fn build_poll_settlement_features(
    billing_features_str: &Option<String>,
    resp_json: &serde_json::Value,
    store_body: &str,
    category: &str,
) -> super::usage_extractor::ExtractedFeatures {
    let mut features = if let Some(ref bf_str) = billing_features_str {
        serde_json::from_str::<super::usage_extractor::ExtractedFeatures>(bf_str)
            .unwrap_or_default()
    } else {
        super::usage_extractor::ExtractedFeatures::default()
    };
    features.merge_settlement_response(resp_json, store_body, category);
    features
}

// ── 成功结算事务 ────────────────────────────────────────────────

/// 成功结算事务：更新日志计费、用户余额差额、令牌配额、渠道配额
pub(super) async fn execute_settlement_tx(
    state: &crate::AppState,
    log_id: i64,
    user_id: &str,
    token_id: Option<i64>,
    channel_id: Option<i64>,
    prompt_tokens: i32,
    completion_tokens: i32,
    cost: f64,
    pre_deduction: f64,
    pre_deduct_gift: f64,
    detail: &str,
    billing_features: Option<&str>, // 新增参数：更新后的计费特征快照JSON
) {
    match state.db.pool.begin().await {
        Ok(mut tx) => {
            let (settled_cost, apply_balance) = crate::money::settlement_delta(cost, pre_deduction);

            // 原子 CAS：仅当 billing_detail 含"冻结"（首次结算）或"退回"（退款后重新结算）时才更新，
            // 且排除用户已取消(499)的记录，防止取消后轮询到 succeeded 覆盖状态码
            // 同时将 status_code 恢复为 200（退款后重新成功场景需要从 400 恢复）
            // 使用 COALESCE(?, billing_features) 绑定，如传入 None 则不修改原有特征快照值
            let result = sqlx::query(&state.db.format_query(&format!(
                "UPDATE logs SET status_code = 200, prompt_tokens = ?, completion_tokens = ?, cached_tokens = 0, cost = ?, billing_detail = ?, billing_features = COALESCE(?, billing_features), error_message = NULL, is_completed = 1, latency_ms = {latency} WHERE id = ? AND (billing_detail LIKE '%冻结%' OR billing_detail LIKE '%退回%') AND status_code != 499",
                latency = LATENCY_MS_SQL
            ))).bind(prompt_tokens).bind(completion_tokens).bind(settled_cost).bind(detail).bind(billing_features).bind(log_id)
            .execute(&mut *tx).await;

            let affected = match &result {
                Ok(r) => r.rows_affected(),
                Err(e) => {
                    tracing::error!("[Settlement] 更新日志错误，事务回滚: {:?}", e);
                    let _ = tx.rollback().await;
                    return;
                }
            };

            if affected == 0 {
                // 另一方已抢先结算，回滚事务避免重复扣费
                let _ = tx.rollback().await;
                tracing::info!("[Settlement] log_id={} 已被其他线程结算，跳过", log_id);
                return;
            }

            // 更新用户账户余额、令牌配额和渠道配额，任一步骤失败都会导致整个事务回滚，确保计费落地一致性
            let res: Result<(), sqlx::Error> = async {
                if apply_balance > 0.0 {
                    sqlx::query(&state.db.format_query(
                        "UPDATE users SET \
                         balance = CASE WHEN gift_balance >= ? THEN balance ELSE balance - (? - gift_balance) END, \
                         gift_used_quota = gift_used_quota + ? + CASE WHEN gift_balance >= ? THEN ? ELSE gift_balance END, \
                         gift_balance = CASE WHEN gift_balance >= ? THEN gift_balance - ? ELSE 0 END, \
                         used_quota = used_quota + ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
                    ))
                    .bind(apply_balance).bind(apply_balance)
                    .bind(pre_deduct_gift).bind(apply_balance).bind(apply_balance)
                    .bind(apply_balance).bind(apply_balance)
                    .bind(settled_cost)
                    .bind(user_id)
                    .execute(&mut *tx).await?;
                } else if apply_balance < 0.0 {
                    let refund = -apply_balance;
                    let gift_cost = settled_cost.min(pre_deduct_gift);
                    let gift_refund = pre_deduct_gift - gift_cost;
                    let balance_refund = refund - gift_refund;
                    sqlx::query(&state.db.format_query(
                        "UPDATE users SET balance = balance + ?, gift_balance = gift_balance + ?, \
                         used_quota = used_quota + ?, gift_used_quota = gift_used_quota + ?, \
                         updated_at = CURRENT_TIMESTAMP WHERE id = ?"
                    )).bind(balance_refund).bind(gift_refund)
                    .bind(settled_cost).bind(gift_cost).bind(user_id)
                    .execute(&mut *tx).await?;
                } else {
                    sqlx::query(&state.db.format_query(
                        "UPDATE users SET used_quota = used_quota + ?, gift_used_quota = gift_used_quota + ?, \
                         updated_at = CURRENT_TIMESTAMP WHERE id = ?"
                    ))
                    .bind(settled_cost)
                    .bind(settled_cost.min(pre_deduct_gift))
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await?;
                }

                if apply_balance != 0.0 {
                    // 渠道/预设：站点时区；令牌：所属用户 timedisplay（与 proxy/中间件一致）
                    let (site_tz, _) = crate::relay::get_cached_config(state).await;
                    if let Some(tid) = token_id {
                        let user_td = crate::api::date_helper::resolve_user_timedisplay_name(
                            &state.db, user_id, &site_tz,
                        )
                        .await;
                        crate::relay::token_quota::apply_delta_with_memory(
                            state, &mut tx, tid, apply_balance, &user_td,
                        )
                        .await?;
                        sqlx::query(&state.db.format_query(
                            "UPDATE api_tokens SET last_used_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
                        ))
                        .bind(tid)
                        .execute(&mut *tx)
                        .await?;
                    }
                    if let Some(cid) = channel_id {
                        if apply_balance > 0.0 {
                            crate::relay::channel_quota::consume_channel(
                                &state.db, &mut tx, cid, apply_balance, &site_tz,
                            )
                            .await?;
                        } else {
                            crate::relay::channel_quota::refund_channel(
                                &state.db, &mut tx, cid, -apply_balance, &site_tz,
                            )
                            .await?;
                        }
                    }
                    // 从日志取上游预设 ID 以便同步累加/退回预设额度
                    let cfg_id: Option<i32> = sqlx::query_scalar(
                        &state.db.format_query("SELECT channel_config_id FROM logs WHERE id = ?")
                    )
                    .bind(log_id)
                    .fetch_optional(&mut *tx)
                    .await?
                    .flatten();
                    if let Some(cfg_id) = cfg_id {
                        if cfg_id > 0 {
                            if apply_balance > 0.0 {
                                crate::relay::channel_quota::consume_config(
                                    &state.db, &mut tx, cfg_id as i64, apply_balance, &site_tz,
                                )
                                .await?;
                            } else {
                                crate::relay::channel_quota::refund_config(
                                    &state.db, &mut tx, cfg_id as i64, -apply_balance, &site_tz,
                                )
                                .await?;
                            }
                        }
                    }
                }
                Ok(())
            }.await;

            if let Err(e) = res {
                tracing::error!("[Settlement] 更新余额或配额失败，事务回滚: {:?}", e);
                let _ = tx.rollback().await;
            } else {
                if let Err(e) = tx.commit().await {
                    tracing::error!("[Settlement] 提交事务失败: {:?}", e);
                } else {
                    tracing::info!(
                        "[Settlement] log_id={}, cost={:.6}, applied={:.6}",
                        log_id,
                        settled_cost,
                        apply_balance
                    );
                    // 异步任务结算成功：补记实时 TPM（该路径不经 record_and_bill_inner）
                    if let Some(tid) = token_id {
                        let live_total = (prompt_tokens.max(0) as u64)
                            .saturating_add(completion_tokens.max(0) as u64);
                        crate::middleware::live_metrics::record_tokens(user_id, tid, live_total);
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("[Settlement] 启动事务失败: {:?}", e);
        }
    }
}

// ── 失败退款事务 ────────────────────────────────────────

/// 失败退款事务：按预扣费钱包来源精准退还余额、令牌配额、渠道配额
pub(crate) async fn execute_refund_tx(
    state: &crate::AppState,
    log_id: i64,
    user_id: &str,
    token_id: Option<i64>,
    channel_id: Option<i64>,
    pre_deduction: f64,
    pre_deduct_gift: f64,
    detail: &str,
    status_code: u16,
) {
    match state.db.pool.begin().await {
        Ok(mut tx) => {
            // 原子 CAS：仅当 billing_detail 仍含"冻结"且未被用户取消(499)时才更新，防止并发双重退款
            let result = sqlx::query(&state.db.format_query(&format!(
                "UPDATE logs SET status_code = ?, cost = 0.0, pre_deduct_gift = 0.0, billing_detail = ?, is_completed = 1, latency_ms = {latency} WHERE id = ? AND billing_detail LIKE '%冻结%' AND status_code != 499",
                latency = LATENCY_MS_SQL
            ))).bind(status_code as i32).bind(detail).bind(log_id)
            .execute(&mut *tx).await;

            let affected = match &result {
                Ok(r) => r.rows_affected(),
                Err(e) => {
                    tracing::error!("[Refund] 更新日志错误，事务回滚: {:?}", e);
                    let _ = tx.rollback().await;
                    return;
                }
            };

            if affected == 0 {
                let _ = tx.rollback().await;
                tracing::info!("[Refund] log_id={} 已被其他线程处理，跳过", log_id);
                return;
            }

            // 更新用户退款余额、令牌配额和渠道已用额度，任何异常都会触发事务安全回滚
            let res: Result<(), sqlx::Error> =
                async {
                    if pre_deduction > 0.0 {
                        let gift_refund =
                            crate::money::round_money(pre_deduct_gift.min(pre_deduction).max(0.0));
                        let balance_refund = crate::money::round_money(pre_deduction - gift_refund);
                        sqlx::query(&state.db.format_query(
                        "UPDATE users SET balance = balance + ?, gift_balance = gift_balance + ?, \
                         updated_at = CURRENT_TIMESTAMP WHERE id = ?"
                    )).bind(balance_refund).bind(gift_refund).bind(user_id)
                    .execute(&mut *tx).await?;

                        let (site_tz, _) = crate::relay::get_cached_config(state).await;
                        if let Some(tid) = token_id {
                            let user_td = crate::api::date_helper::resolve_user_timedisplay_name(
                                &state.db, user_id, &site_tz,
                            )
                            .await;
                            crate::relay::token_quota::refund(
                                &state.db,
                                &mut tx,
                                tid,
                                pre_deduction,
                                &user_td,
                            )
                            .await?;
                            state
                                .quota_memory
                                .apply_refund_ensured(&state.db, tid, &user_td, pre_deduction)
                                .await;
                        }
                        if let Some(cid) = channel_id {
                            crate::relay::channel_quota::refund_channel(
                                &state.db,
                                &mut tx,
                                cid,
                                pre_deduction,
                                &site_tz,
                            )
                            .await?;
                        }
                        let cfg_id: Option<i32> = sqlx::query_scalar(
                            &state
                                .db
                                .format_query("SELECT channel_config_id FROM logs WHERE id = ?"),
                        )
                        .bind(log_id)
                        .fetch_optional(&mut *tx)
                        .await?
                        .flatten();
                        if let Some(cfg_id) = cfg_id {
                            if cfg_id > 0 {
                                crate::relay::channel_quota::refund_config(
                                    &state.db,
                                    &mut tx,
                                    cfg_id as i64,
                                    pre_deduction,
                                    &site_tz,
                                )
                                .await?;
                            }
                        }
                    }
                    Ok(())
                }
                .await;

            if let Err(e) = res {
                tracing::error!("[Refund] 更新退款余额或配额失败，事务回滚: {:?}", e);
                let _ = tx.rollback().await;
            } else {
                if let Err(e) = tx.commit().await {
                    tracing::error!("[Refund] 提交事务失败: {:?}", e);
                } else {
                    tracing::info!("[Refund] 日志ID={}, 退款={:.6}", log_id, pre_deduction);
                }
            }
        }
        Err(e) => {
            tracing::error!("[Refund] 启动事务失败: {:?}", e);
        }
    }
}

/// 轮询业务终态：写 error_message（保留 response_content）并退费。GET / 后台共用。
async fn refund_poll_terminal(
    state: &AppState,
    log_id: i64,
    cascade_stage: u8,
    err: &str,
    status: u16,
) {
    let _ = sqlx::query(
        &state
            .db
            .format_query("UPDATE logs SET error_message = ? WHERE id = ?"),
    )
    .bind(err)
    .bind(log_id)
    .execute(&state.db.pool)
    .await;
    settle_failure(
        state,
        log_id,
        &format!("poll_upstream_error:{}", err),
        status,
        cascade_stage,
    )
    .await;
}

/// 轮询 HTTP 已成功：清掉 POLL_FAIL 累计，避免恢复后仍被旧计数退费。
async fn clear_poll_fail(state: &AppState, log_id: i64) {
    let _ = sqlx::query(&state.db.format_query(
        "UPDATE logs SET error_message = NULL WHERE id = ? AND error_message LIKE '[POLL_FAIL:%'",
    ))
    .bind(log_id)
    .execute(&state.db.pool)
    .await;
}

/// 解析 `[poll:status] msg`；非该前缀返回 None（不计入 POLL_FAIL）。
fn split_poll_retry_err(raw: &str) -> Option<(u16, &str)> {
    let rest = raw.strip_prefix("[poll:")?;
    let (code, msg) = rest.split_once(']')?;
    let status = proxy::normalize_error_http_status(code.parse().ok()?);
    Some((status, msg.trim_start()))
}

/// 解析 `[POLL_FAIL:count]` / `[POLL_FAIL:count:status]` → (次数, 状态码)；旧格式无 status 时默认 502。
fn parse_poll_fail_meta(error_message: Option<&str>) -> (u32, u16) {
    let Some(m) = error_message.and_then(|m| m.strip_prefix("[POLL_FAIL:")) else {
        return (0, 502);
    };
    let Some((tag, _)) = m.split_once(']') else {
        return (0, 502);
    };
    let mut parts = tag.split(':');
    let count = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0);
    let status = parts
        .next()
        .and_then(|n| n.parse().ok())
        .map(proxy::normalize_error_http_status)
        .unwrap_or(502);
    (count, status)
}

fn format_poll_fail_tag(count: u32, status: u16, msg: &str) -> String {
    format!("[POLL_FAIL:{}:{}] {}", count, status, msg)
}

// ── 轮询请求 ────────────────────────────────────────────────────

/// 轮询请求失败：HTTP 非 2xx 带真实 status；连接失败无 status（一律可重试）。
struct PollReqErr {
    http_status: Option<u16>,
    message: String,
}

impl PollReqErr {
    fn transport(message: String) -> Self {
        Self {
            http_status: None,
            message,
        }
    }

    async fn from_http(resp: reqwest::Response) -> Self {
        let status = resp.status();
        let err_body = resp.text().await.unwrap_or_default();
        let detail = serde_json::from_str::<serde_json::Value>(&err_body)
            .map(|v| super::response_formatter::extract_error_message(&v))
            .unwrap_or_default();
        let message = if detail.is_empty() || detail == "generation failed" {
            format!("渠道返回错误状态码: {}", status)
        } else {
            format!("渠道返回错误状态码: {} - {}", status, detail)
        };
        Self {
            http_status: Some(status.as_u16()),
            message,
        }
    }

    /// (规范化状态码, 是否可重试) — 直接用 HTTP 码，不反解析文案；500 与 429/5xx 网关同属可重试。
    fn classify(&self) -> (u16, bool) {
        match self.http_status {
            Some(s) => {
                let s = proxy::normalize_error_http_status(s);
                (s, proxy::is_poll_transport_retryable(s))
            }
            None => (proxy::infer_error_status_code_from_str(&self.message), true),
        }
    }
}

/// 构建轮询 URL + 鉴权 + 发送（GET/后台/`poll_task_result` 共用）。
/// 成功返回 (poll_url, body)；失败返回结构化 [`PollReqErr`]。
async fn send_poll_request(
    http_client: &reqwest::Client,
    channel: &crate::models::Channel,
    resolved: &super::forward::ResolvedForward,
    task_id: &str,
    model: &str,
    jimeng_ctx: Option<(&str, &str)>, // (upstream_req_content, request_content)
) -> Result<(String, String), PollReqErr> {
    let is_tencent = resolved.target_type.starts_with("tencent_vod");
    let is_jimeng = resolved.target_type.starts_with("jimeng_");

    // 即梦AI：POST 轮询（req_key + task_id + 可选 req_json）
    if is_jimeng {
        let (ak, sk) = forward::parse_jimeng_key(&channel.api_key);
        // req_key 从 upstream_req_content 提取
        let (upstream_req_str, request_content_str) = jimeng_ctx.unwrap_or(("", ""));
        let jimeng_req_key = serde_json::from_str::<serde_json::Value>(upstream_req_str)
            .ok()
            .and_then(|v| {
                v.get("req_key")
                    .and_then(|r| r.as_str().map(|s| s.to_string()))
            })
            .unwrap_or_else(|| model.to_string());
        let mut poll_body = serde_json::json!({
            "req_key": jimeng_req_key,
            "task_id": task_id
        });
        // req_json 组装：优先用户原始请求(request_content)中的 req_json > OpenAI 参数转换 + 兜底
        // 注意：火山引擎 CV API 要求 req_json 为 JSON 字符串格式，非嵌套对象
        if let Ok(req) = serde_json::from_str::<serde_json::Value>(request_content_str) {
            if let Some(rj) = req.get("req_json") {
                // 用户直接传入的 req_json：确保为字符串格式
                poll_body["req_json"] = if rj.is_string() {
                    rj.clone()
                } else {
                    serde_json::json!(serde_json::to_string(rj).unwrap_or_default())
                };
            } else {
                let mut assembled = serde_json::Map::new();
                // return_url：有 response_format 按参数定义，没有则兜底为 true
                let return_url =
                    if let Some(rf) = req.get("response_format").and_then(|v| v.as_str()) {
                        rf != "b64_json" // b64_json 返回 base64，其他（url 等）返回 URL
                    } else {
                        true // 未指定时默认返回 URL
                    };
                assembled.insert("return_url".to_string(), serde_json::json!(return_url));
                if req
                    .get("watermark")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    assembled.insert(
                        "logo_info".to_string(),
                        serde_json::json!({"add_logo": true}),
                    );
                }
                poll_body["req_json"] = serde_json::json!(serde_json::to_string(
                    &serde_json::json!(assembled)
                )
                .unwrap_or_default());
            }
        } else {
            // request_content 解析失败时仍兜底 return_url
            poll_body["req_json"] = serde_json::json!("{\"return_url\":true}");
        }
        let body_str = serde_json::to_string(&poll_body).unwrap_or_default();
        let headers = forward::build_jimeng_headers(
            ak,
            sk,
            "CVSync2AsyncGetResult",
            &body_str,
            &channel.base_url,
        );
        let poll_url = format!(
            "{}/?Action=CVSync2AsyncGetResult&Version=2022-08-31",
            channel.base_url.trim_end_matches('/')
        );
        tracing::info!("[PollTask] 轮询 URL={}", poll_url);
        let mut builder = http_client
            .post(&poll_url)
            .header("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(30));
        for (k, v) in headers {
            builder = builder.header(k, v);
        }
        // 使用已签名的 body_str 发送，避免 .json() 重新序列化导致签名不匹配
        let resp = builder.body(body_str).send().await.map_err(|e| {
            PollReqErr::transport(proxy::sanitize_error_message(&format!(
                "请求渠道失败: {}",
                e
            )))
        })?;
        if !resp.status().is_success() {
            return Err(PollReqErr::from_http(resp).await);
        }
        let body = resp.text().await.unwrap_or_default();
        return Ok((poll_url, body));
    }

    let poll_path = if is_tencent {
        "/".to_string()
    } else if let Some(ref custom_path) = resolved.poll_path {
        custom_path
            .replace("${task_id}", task_id)
            .replace("${model}", model)
    } else if resolved.target_type.is_empty()
        || resolved.target_type == "openai"
        || resolved.target_type == "apimart"
    {
        format!("/v1/tasks/{}", task_id)
    } else {
        let path = resolved.upstream_path.replace("${model}", model);
        format!("{}/{}", path.trim_end_matches('/'), task_id)
    };

    let url = if is_tencent {
        "https://vod.tencentcloudapi.com".to_string()
    } else {
        join_url(&channel.base_url, &poll_path)
    };
    tracing::info!("[PollTask] 轮询 URL={}", url);

    let resp = if is_tencent {
        let (ak, sk, sub_app_id) = forward::parse_tencent_vod_key(&channel.api_key);
        let tc_body = serde_json::json!({ "TaskId": task_id, "SubAppId": sub_app_id });
        let body_str = serde_json::to_string(&tc_body).unwrap_or_default();
        let tc_headers =
            forward::build_tencent_vod_headers(ak, sk, "DescribeTaskDetail", &body_str);
        let mut builder = http_client
            .post(&url)
            .timeout(std::time::Duration::from_secs(30));
        for (k, v) in tc_headers {
            builder = builder.header(k, v);
        }
        // 使用已签名的 body_str 发送，避免 .json() 重新序列化导致签名不匹配
        builder.body(body_str).send().await
    } else {
        let auth = forward::build_auth_headers(resolved, &channel.api_key);
        let mut builder = http_client
            .get(&url)
            .timeout(std::time::Duration::from_secs(30));
        for (k, v) in auth {
            builder = builder.header(k, v);
        }
        builder.send().await
    };

    let resp = resp.map_err(|e| {
        PollReqErr::transport(proxy::sanitize_error_message(&format!(
            "请求渠道失败: {}",
            e
        )))
    })?;
    if !resp.status().is_success() {
        return Err(PollReqErr::from_http(resp).await);
    }
    let body = resp.text().await.unwrap_or_default();
    Ok((url, body))
}

// ── 通用轮询 ────────────────────────────────────────────────────

/// [`poll_task_result`] 可选参数；缺省适合 MediaKit 等无 model/即梦上下文场景。
#[derive(Clone, Copy)]
pub struct PollTaskOpts<'a> {
    pub model: &'a str,
    pub category: &'a str,
    pub timeout_secs: u64,
    /// `(upstream_req_content, request_content)`，即梦等厂商需要
    pub jimeng_ctx: Option<(&'a str, &'a str)>,
}

impl Default for PollTaskOpts<'_> {
    fn default() -> Self {
        Self {
            model: "",
            category: "",
            timeout_secs: 300,
            jimeng_ctx: None,
        }
    }
}

/// 异步任务通用轮询（供通道测试 / 同步图 / 裁剪等场景复用）。
/// 仅轮询上游获取终态响应，不执行计费结算。
/// 返回 Some((终态响应字符串, "succeeded"|"failed")) 或 None（超时/请求失败）。
///
/// 策略：每次查询前倒序休眠 5→4→3→2→1s；连续失败达 [`POLL_FAIL_LIMIT`] 终止。
pub async fn poll_task_result(
    http_client: &reqwest::Client,
    channel: &crate::models::Channel,
    resolved: &super::forward::ResolvedForward,
    task_id: &str,
    opts: PollTaskOpts<'_>,
) -> Option<(String, String)> {
    let is_tencent = resolved.target_type.starts_with("tencent_vod");
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(opts.timeout_secs);
    let mut consecutive_errors: u32 = 0;
    let mut attempt: u32 = 0;

    tracing::info!(
        "[PollTask] 开始轮询 任务ID={}, 超时={}秒",
        task_id,
        opts.timeout_secs
    );

    loop {
        if tokio::time::Instant::now() >= deadline {
            break;
        }
        attempt += 1;
        if !poll_wait_before_query(attempt, deadline).await {
            break;
        }

        match send_poll_request(
            http_client,
            channel,
            resolved,
            task_id,
            opts.model,
            opts.jimeng_ctx,
        )
        .await
        {
            Ok((_url, body)) => {
                consecutive_errors = 0;
                let resp_json: serde_json::Value =
                    serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
                let raw_status = super::response_formatter::extract_raw_status(&resp_json);
                let task_status = normalize_task_status(&raw_status).to_string();

                tracing::info!(
                    "[PollTask] 轮询第 {} 次, 任务ID={}, 状态={}",
                    attempt,
                    task_id,
                    task_status
                );

                if task_status == "succeeded" || task_status == "failed" {
                    tracing::info!(
                        "[PollTask] 终态 任务ID={}, 状态={}, 响应长度={}",
                        task_id,
                        task_status,
                        body.len()
                    );
                    let store_body = if is_tencent {
                        log_tencent_poll_raw("PollTask", task_id, &body);
                        super::response_formatter::format_openai(
                            opts.category,
                            &body,
                            true,
                            Some(task_id),
                        )
                    } else {
                        body
                    };
                    return Some((store_body, task_status));
                }
            }
            Err(e) => {
                consecutive_errors += 1;
                tracing::warn!(
                    "[PollTask] 轮询请求失败 ({}/{}): {} (任务ID={})",
                    consecutive_errors,
                    POLL_FAIL_LIMIT,
                    e.message,
                    task_id
                );
                if consecutive_errors >= POLL_FAIL_LIMIT {
                    tracing::error!(
                        "[PollTask] 连续 {} 次请求失败，放弃轮询 任务ID={}",
                        POLL_FAIL_LIMIT,
                        task_id
                    );
                    return None;
                }
            }
        }
    }

    tracing::warn!(
        "[PollTask] 轮询超时 任务ID={}, 已尝试 {} 次",
        task_id,
        attempt
    );
    None
}
