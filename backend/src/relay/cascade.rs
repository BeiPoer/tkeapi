/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

use super::proxy;
use super::task::{execute_refund_tx, normalize_task_status, poll_task_result, PollTaskOpts};
use crate::error::{AppError, AppResult};
use crate::models::{BillingRule, Channel};
use crate::relay::{forward, response_formatter};
use crate::AppState;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// 增强版本 → (billing version, 火山 mid)；非法返回 None
fn cascade_enhance_pair(version: &str) -> Option<(&'static str, &'static str)> {
    match version.trim().to_ascii_lowercase().as_str() {
        "fast" => Some(("fast", "vve-ft")),
        "standard" => Some(("standard", "vve-sd")),
        "pro" => Some(("pro", "vve-pf")),
        "ai" => Some(("ai", "vve-gt")),
        _ => None,
    }
}

/// 阶段二超分分辨率入参：非大模型 + 目标 480p → 整型 `resolution_limit=480`（锁标准 480p）；否则字符串 `resolution`
fn cascade_s2_apply_resolution_param(payload: &mut serde_json::Value, target_res: &str, mid: &str) {
    let Some(obj) = payload.as_object_mut() else {
        return;
    };
    obj.remove("resolution");
    obj.remove("resolution_limit");
    let res = target_res.trim().to_ascii_lowercase();
    // 非 vve-gt：MediaKit resolution_limit 为像素上限整型，与字符串 resolution 互斥
    if !mid.trim().eq_ignore_ascii_case("vve-gt") && res == "480p" {
        obj.insert("resolution_limit".to_string(), serde_json::json!(496));
    } else {
        obj.insert("resolution".to_string(), serde_json::json!(res));
    }
}

/// 优先用转发规则 res_enhance[目标分辨率]；缺省/非法/分辨率不支持的 ai → 标准版
pub(crate) fn cascade_resolve_enhance(
    target_res: &str,
    res_enhance: &HashMap<String, String>,
) -> (&'static str, &'static str) {
    let key = target_res.trim().to_ascii_lowercase();
    res_enhance
        .get(&key)
        .and_then(|ver| {
            let pair = cascade_enhance_pair(ver)?;
            // 大模型增强（ai）仅 720p / 1080p / 2k
            if pair.0 == "ai" && !matches!(key.as_str(), "720p" | "1080p" | "2k") {
                None
            } else {
                Some(pair)
            }
        })
        .unwrap_or(("standard", "vve-sd"))
}

/// 标准版增强场景枚举（仅 tool_version=standard 生效）
fn cascade_scene_pair(scene: &str) -> Option<&'static str> {
    match scene.trim().to_ascii_lowercase().as_str() {
        "common" => Some("common"),
        "ugc" => Some("ugc"),
        "short_series" => Some("short_series"),
        "aigc" => Some("aigc"),
        "old_film" => Some("old_film"),
        _ => None,
    }
}

/// 仅标准增强返回场景；配置合法则用配置，否则 common；非标准 → None
pub(crate) fn cascade_resolve_scene(
    cascade_version: &str,
    target_res: &str,
    res_scene: &HashMap<String, String>,
) -> Option<&'static str> {
    if cascade_version != "standard" {
        return None;
    }
    Some(
        res_scene
            .get(&target_res.trim().to_ascii_lowercase())
            .and_then(|s| cascade_scene_pair(s))
            .unwrap_or("common"),
    )
}

fn cascade_is_res(s: &str) -> bool {
    matches!(
        s.trim().to_ascii_lowercase().as_str(),
        "480p" | "720p" | "768p" | "1080p" | "2k" | "4k"
    )
}

/// 与 cascade_enhance_pair 同源，避免版本枚举双份维护
fn cascade_is_version(s: &str) -> bool {
    cascade_enhance_pair(s).is_some()
}

/// 目标分辨率允许的底座列表（首项为默认一级；单元素即锁定不可改）
fn cascade_allowed_bases(target: &str) -> &'static [&'static str] {
    match target.trim().to_ascii_lowercase().as_str() {
        "480p" => &["480p"],
        "720p" => &["480p", "720p"],
        "1080p" => &["720p", "480p", "1080p"],
        "2k" | "4k" => &["1080p", "720p", "480p"],
        _ => &["720p"],
    }
}

/// 有分辨率计费时返回已启用集合；非分辨率计费或无配置则返回 None
fn cascade_billing_enabled_resolutions(
    rule: &BillingRule,
    cascade_version: &str,
) -> Option<HashSet<String>> {
    // requests (按次计费) 不参与分辨率拦截
    if rule.billing_type.eq_ignore_ascii_case("requests") {
        return None;
    }
    if (rule.extended_config.is_empty() || rule.extended_config == "{}")
        && (rule.pricing_tiers.is_empty() || rule.pricing_tiers == "[]")
    {
        return None;
    }

    let ext: serde_json::Value = serde_json::from_str(&rule.extended_config).unwrap_or_default();
    let mut has_res_billing = false;
    let mut enabled = HashSet::new();

    if let Some(rates) = ext.get("resolution_rates").and_then(|v| v.as_object()) {
        has_res_billing = true;
        for k in rates.keys().filter(|k| cascade_is_res(k)) {
            enabled.insert(k.to_ascii_lowercase());
        }
    }

    if let Some(pt) = ext.get("price_table").and_then(|v| v.as_object()) {
        let disabled: HashSet<String> = ext
            .get("price_table_disabled")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_ascii_lowercase()))
                    .collect()
            })
            .unwrap_or_default();
        for key in pt.keys() {
            let lower = key.to_ascii_lowercase();
            let parts: Vec<&str> = lower.split('|').collect();
            let res = match parts.as_slice() {
                [ver, res, ..] if cascade_is_version(ver) && cascade_is_res(res) => {
                    has_res_billing = true;
                    if cascade_version.is_empty() || ver.eq_ignore_ascii_case(cascade_version) {
                        Some(*res)
                    } else {
                        None
                    }
                }
                [attr, res] if !cascade_is_version(attr) && cascade_is_res(res) => {
                    has_res_billing = true;
                    Some(*res)
                }
                _ => None,
            };
            if let Some(res) = res {
                if !disabled.contains(&lower) {
                    enabled.insert(res.to_string());
                }
            }
        }
    }

    if !rule.pricing_tiers.is_empty() && rule.pricing_tiers != "[]" {
        if let Ok(tiers) = serde_json::from_str::<Vec<serde_json::Value>>(&rule.pricing_tiers) {
            for tier in tiers {
                let Some(res) = tier.get("resolution").and_then(|v| v.as_str()) else {
                    continue;
                };
                if !cascade_is_res(res) {
                    continue;
                }
                has_res_billing = true;
                if tier
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true)
                {
                    enabled.insert(res.trim().to_ascii_lowercase());
                }
            }
        }
    }

    has_res_billing.then_some(enabled)
}

/// 格式与计费启用校验：仅当模型存在生效的分辨率计费配置时才进行开启拦截；非分辨率计费则直接放行。
pub(crate) fn cascade_check_resolution(
    db_rule: Option<&BillingRule>,
    cascade_version: &str,
    res_str: &str,
) -> AppResult<()> {
    let Some(rule) = db_rule else {
        return Ok(());
    };
    let Some(enabled) = cascade_billing_enabled_resolutions(rule, cascade_version) else {
        return Ok(());
    };
    let key = res_str.trim().to_ascii_lowercase();
    if enabled.contains(&key) {
        return Ok(());
    }
    Err(AppError::BadRequest(format!(
        "当前分辨率 {} 不支持",
        res_str.trim()
    )))
}

/// 目标分辨率 → 默认一级底座（未配置 res_base 时）
fn cascade_clamp_base_resolution(target: &str) -> &'static str {
    cascade_allowed_bases(target)
        .first()
        .copied()
        .unwrap_or("720p")
}

/// 优先用转发规则 res_base[目标]；缺省/非法回退默认一级底座
pub(crate) fn cascade_resolve_base(
    target: &str,
    res_base: &HashMap<String, String>,
) -> &'static str {
    let key = target.trim().to_ascii_lowercase();
    let allowed = cascade_allowed_bases(&key);
    res_base
        .get(&key)
        .and_then(|configured| {
            let b = configured.trim().to_ascii_lowercase();
            allowed.iter().copied().find(|a| a.eq_ignore_ascii_case(&b))
        })
        .unwrap_or_else(|| cascade_clamp_base_resolution(&key))
}

/// MediaKit 共用上下文（http + 增强渠道鉴权），避免裁剪/抽帧重复传参。
pub(crate) struct CascadeMk<'a> {
    pub http: &'a reqwest::Client,
    pub ch: &'a Channel,
    pub auth_type: &'a str,
}

/// MediaKit 异步工具：POST → `poll_task_result`（5→1s）→ 取 `out_ptr`。
/// POST 提交失败重试仍用短退避，与任务状态轮询分离。
async fn cascade_mk_url(
    mk: &CascadeMk<'_>,
    path: &str,
    payload: serde_json::Value,
    out_ptr: &str,
) -> Option<String> {
    let resolved = forward::ResolvedForward {
        auth_type: mk.auth_type.to_string(),
        upstream_path: path.to_string(),
        poll_path: Some("/api/v1/tasks/${task_id}".to_string()),
        ..Default::default()
    };
    let url = forward::build_upstream_url(&mk.ch.base_url, &resolved, "", &mk.ch.api_key);

    let mut attempt = 0u32;
    let task_id = loop {
        attempt += 1;
        let mut body = payload.clone();
        let builder =
            crate::services::http_client::with_upstream_timeout(forward::apply_request_auth(
                mk.http
                    .post(&url)
                    .header("Content-Type", "application/json"),
                &resolved,
                &mk.ch.api_key,
                &mut body,
                &mk.ch.base_url,
            ));
        let retry = match builder.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if status != 200 {
                    crate::relay::proxy::is_poll_transport_retryable(status)
                } else {
                    let text = resp.text().await.unwrap_or_default();
                    let post: serde_json::Value =
                        serde_json::from_str(&text).unwrap_or(serde_json::json!({}));
                    let id = response_formatter::find_id(&post);
                    if !id.is_empty() && !response_formatter::is_upstream_error_response(&post) {
                        break id;
                    }
                    false
                }
            }
            Err(_) => true,
        };
        if retry && attempt < 5 {
            let delay = (2u64 << (attempt - 1)).min(10);
            tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
            continue;
        }
        return None;
    };

    let (body, status) =
        poll_task_result(mk.http, mk.ch, &resolved, &task_id, PollTaskOpts::default()).await?;
    if status != "succeeded" {
        return None;
    }
    let v: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
    v.pointer(out_ptr)
        .and_then(|u| u.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// 仅 480→720 且 ratio∈{16:9,9:16}：MediaKit 居中裁成标准 480p；否则/失败返回原 URL。
/// 角点：16:9→(2,6,862,490)；9:16→(6,2,490,862)。S1 明确非 480p 时跳过；ratio 优先 S1，缺则 hints。
async fn cascade_ensure_standard_480p_video(
    mk: &CascadeMk<'_>,
    video_url: &str,
    stage1_resp: &serde_json::Value,
    target_resolution: &str,
    field_hints: &[&serde_json::Value],
) -> String {
    fn root<'a>(v: &'a serde_json::Value, key: &str) -> Option<&'a str> {
        v.get(key)
            .and_then(|x| x.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }
    if !target_resolution.eq_ignore_ascii_case("720p") {
        return video_url.to_string();
    }
    // 勿用用户入参 resolution=720p 误判；仅信 S1 回显
    if root(stage1_resp, "resolution").is_some_and(|r| !r.eq_ignore_ascii_case("480p")) {
        return video_url.to_string();
    }
    let Some(ratio) =
        root(stage1_resp, "ratio").or_else(|| field_hints.iter().find_map(|h| root(h, "ratio")))
    else {
        return video_url.to_string();
    };
    let (tlx, tly, brx, bry) = match ratio {
        "16:9" => (2, 6, 862, 490),
        "9:16" => (6, 2, 490, 862),
        _ => return video_url.to_string(),
    };

    cascade_mk_url(
        mk,
        "/api/v1/tools/crop-video",
        serde_json::json!({
            "video_url": video_url,
            "top_left_x": tlx,
            "top_left_y": tly,
            "bottom_right_x": brx,
            "bottom_right_y": bry,
        }),
        "/result/video_url",
    )
    .await
    .unwrap_or_else(|| video_url.to_string())
}

/// S2 成功落库前：stage1 usage×res_mul；S1 有尾帧则对 S2 视频抽帧写入 `s2.last_frame_url`（不改 stage1）。
pub(crate) async fn cascade_on_s2_succeeded(
    mk: &CascadeMk<'_>,
    s1: &mut serde_json::Value,
    s2_raw: &mut String,
    res_mul: &HashMap<String, f64>,
    plugin_tag: &str,
) {
    apply_cascade_res_mul_to_stage1(s1, res_mul, plugin_tag);

    if response_formatter::find_last_frame_url(s1).is_none() {
        return;
    }
    let mut s2: serde_json::Value = serde_json::from_str(s2_raw).unwrap_or(serde_json::json!({}));
    let Some(video_url) = s2
        .pointer("/result/video_url")
        .and_then(|u| u.as_str())
        .filter(|s| !s.is_empty())
    else {
        return;
    };
    let Some(frame) = cascade_mk_url(
        mk,
        "/api/v1/tools/extract-frames",
        serde_json::json!({
            "video_url": video_url,
            "snapshot_type": "SpecifiedFrames",
            "specified_frames": [-1],
        }),
        "/result/snapshots/0/image_url",
    )
    .await
    else {
        tracing::warn!("[Cascade S2] 尾帧抽帧失败，跳过");
        return;
    };
    if let Some(obj) = s2.as_object_mut() {
        obj.insert("last_frame_url".into(), serde_json::json!(frame));
    }
    *s2_raw = s2.to_string();
}

/// 阶段一出参 + 阶段二增强请求；`s1_raw` 空（未开 enable_log）→ None。
fn cascade_upstream_req_combined(s1_raw: &str, s2: &serde_json::Value) -> Option<String> {
    if s1_raw.is_empty() {
        return None;
    }
    let s1: serde_json::Value = serde_json::from_str(s1_raw).unwrap_or(serde_json::json!({}));
    Some(serde_json::json!({ "stage1": s1, "stage2": s2 }).to_string())
}

/// 阶段二 POST HTTP200 无有效 task_id 时的分类（文案/状态码由调用方拼，避免 cascade↔proxy 耦合）。
enum CascadeS2Post200Fail {
    /// 上游业务错误体
    Upstream(serde_json::Value),
    /// 非错误体但解析不到 task_id（调用方宜 warn 原文）
    MissingTaskId,
}

/// 阶段二 POST HTTP200：有 task_id → Ok；否则 Err 分类。
fn cascade_s2_parse_post_200(
    text: &str,
) -> Result<(String, serde_json::Value), CascadeS2Post200Fail> {
    let post: serde_json::Value = serde_json::from_str(text).unwrap_or(serde_json::json!({}));
    let id = response_formatter::find_id(&post);
    if !id.is_empty() {
        return Ok((id, post));
    }
    if response_formatter::is_upstream_error_response(&post) {
        Err(CascadeS2Post200Fail::Upstream(post))
    } else {
        Err(CascadeS2Post200Fail::MissingTaskId)
    }
}

/// JSON 指针取非空字符串；`lower=true` 时转小写（分辨率等），任务 id 等保持原样
fn cascade_json_ptr(json: &str, pointer: &str, lower: bool) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .and_then(|v| {
            v.pointer(pointer)?.as_str().map(|s| {
                let t = s.trim();
                if lower {
                    t.to_ascii_lowercase()
                } else {
                    t.to_string()
                }
            })
        })
        .filter(|s| !s.is_empty())
}

#[inline]
pub(crate) fn cascade_json_str(json: &str, pointer: &str) -> Option<String> {
    cascade_json_ptr(json, pointer, true)
}

/// 级联对外任务号：`cgt-{YYYYMMDDHHmmss}-{5位随机}`
fn cascade_new_client_task_id() -> String {
    let ts = chrono::Local::now().format("%Y%m%d%H%M%S");
    let u = ulid::Ulid::new().to_string().to_lowercase();
    format!("cgt-{}-{}", ts, &u[21..26])
}

/// 写入 `cascade.s1_task_id`（仅内部），返回对外 cgt（由调用方写入响应体 `id` / `logs.task_id`）
pub(crate) fn cascade_seal_s1_task_id(
    plugin_tag: &mut Option<String>,
    upstream_s1_id: &str,
) -> Option<String> {
    let upstream_s1_id = upstream_s1_id.trim();
    if upstream_s1_id.is_empty() {
        return None;
    }
    let cgt = cascade_new_client_task_id();
    let mut v: serde_json::Value = plugin_tag
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    let cascade = v
        .as_object_mut()?
        .entry("cascade")
        .or_insert_with(|| serde_json::json!({}));
    cascade
        .as_object_mut()?
        .insert("s1_task_id".into(), serde_json::json!(upstream_s1_id));
    *plugin_tag = Some(v.to_string());
    Some(cgt)
}

/// 阶段一轮询上游：优先 `s1_task_id`，旧单无字段则回退 logs/path 上的 id
pub(crate) fn cascade_s1_upstream_task_id(plugin_tag: &str, fallback: &str) -> String {
    cascade_json_ptr(plugin_tag, "/cascade/s1_task_id", false)
        .unwrap_or_else(|| fallback.to_string())
}

/// 级联目标分辨率：plugin_tag.cascade.resolution → 请求体 resolution → 720p。
fn cascade_resolve_target_resolution(plugin_tag: &str, request_content: &str) -> String {
    cascade_json_str(plugin_tag, "/cascade/resolution")
        .or_else(|| cascade_json_str(request_content, "/resolution"))
        .unwrap_or_else(|| "720p".into())
}

/// 从阶段二增强响应提取帧率（result.fps / 顶层 fps）
fn cascade_s2_fps(s2: &serde_json::Value) -> Option<i64> {
    s2.pointer("/result/fps")
        .or_else(|| s2.get("fps"))
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_u64().map(|u| u as i64))
                .or_else(|| v.as_f64().map(|f| f as i64))
        })
        .filter(|&f| f > 0)
}

/// 递归覆写已存在的同名 string / number 字段（不凭空插入）
fn patch_json_fields_by_key(
    value: &mut serde_json::Value,
    str_patches: &[(&str, &str)],
    num_patches: &[(&str, i64)],
) {
    match value {
        serde_json::Value::Object(map) => {
            for &(key, val) in str_patches {
                if map.get(key).is_some_and(|v| v.is_string()) {
                    map.insert(key.to_string(), serde_json::json!(val));
                }
            }
            for &(key, val) in num_patches {
                if map.get(key).is_some_and(|v| v.is_number()) {
                    map.insert(key.to_string(), serde_json::json!(val));
                }
            }
            for (_, child) in map.iter_mut() {
                patch_json_fields_by_key(child, str_patches, num_patches);
            }
        }
        serde_json::Value::Array(arr) => {
            for child in arr.iter_mut() {
                patch_json_fields_by_key(child, str_patches, num_patches);
            }
        }
        _ => {}
    }
}

/// 级联成功对外：S1 官方骨架对齐 S2 产物（视频 URL / 目标分辨率 / 帧率 / 抽帧尾图；ratio·duration·usage 保持）
/// 无增强产物 URL 时不改元数据，避免分辨率/帧率与底座视频不一致
fn cascade_s1_with_s2_url(
    s1: &serde_json::Value,
    s2: &serde_json::Value,
    plugin_tag: &str,
) -> serde_json::Value {
    let old_url = response_formatter::find_urls(s1)
        .into_iter()
        .next()
        .unwrap_or_default();
    let new_url = response_formatter::find_urls(s2)
        .into_iter()
        .next()
        .unwrap_or_default();
    let mut out = s1.clone();
    if new_url.is_empty() {
        return out;
    }

    if !old_url.is_empty() {
        replace_exact_url_in_json(&mut out, &old_url, &new_url);
    } else {
        // S1 未解析到旧链时，直接覆写已有 video_url（如 content.video_url）
        patch_json_fields_by_key(&mut out, &[("video_url", new_url.as_str())], &[]);
    }

    let target_res = cascade_resolve_target_resolution(plugin_tag, "");
    let fps = cascade_s2_fps(s2).unwrap_or(60);
    patch_json_fields_by_key(
        &mut out,
        &[("resolution", target_res.as_str())],
        &[("framespersecond", fps), ("fps", fps)],
    );

    // 对外/用户端展示：用 S2 抽帧尾图覆盖 S1 原尾帧（落库 stage1 仍为原图）
    if let (Some(old_frame), Some(new_frame)) = (
        response_formatter::find_last_frame_url(s1),
        response_formatter::find_last_frame_url(s2),
    ) {
        replace_exact_url_in_json(&mut out, old_frame, new_frame);
    }
    out
}

fn replace_exact_url_in_json(value: &mut serde_json::Value, old_url: &str, new_url: &str) {
    if old_url.is_empty() || new_url.is_empty() {
        return;
    }
    match value {
        serde_json::Value::Object(map) => {
            for (_, val) in map.iter_mut() {
                replace_exact_url_in_json(val, old_url, new_url);
            }
        }
        serde_json::Value::Array(arr) => {
            for val in arr.iter_mut() {
                replace_exact_url_in_json(val, old_url, new_url);
            }
        }
        serde_json::Value::String(s) => {
            if s == old_url {
                *s = new_url.to_string();
            }
        }
        _ => {}
    }
}

/// 列表/仪表盘/终态落库：去掉 plugin_tag.cascade 中的密钥与上游渠道细节。
/// 返回是否发生了字段删除（无变更则不改写字符串）。
pub(crate) fn cascade_scrub_plugin_tag_for_user(plugin_tag: &mut Option<String>) -> bool {
    let Some(raw) = plugin_tag.as_deref() else {
        return false;
    };
    if !raw.contains("\"cascade\"") {
        return false;
    }
    let Ok(mut v) = serde_json::from_str::<serde_json::Value>(raw) else {
        return false;
    };
    let Some(obj) = v.get_mut("cascade").and_then(|c| c.as_object_mut()) else {
        return false;
    };
    let mut changed = false;
    for key in ["api_key", "base_url", "ch_name", "ch_id", "mid"] {
        if obj.remove(key).is_some() {
            changed = true;
        }
    }
    if changed {
        *plugin_tag = Some(v.to_string());
    }
    changed
}

/// 普通用户日志级联字段脱敏（仅 response / post_response；上游出参接口层已不返回）。
/// - 未完成级联：响应改为处理中形态，硬保证无产物 URL
/// - 已完成级联：折叠 stage，合并 S2 产物 URL
/// - 非级联：仅在有 cascade 配置时修补 resolution
pub(crate) fn cascade_sanitize_for_user(
    response: &mut Option<String>,
    post_resp: &mut Option<String>,
    plugin_tag: Option<&str>,
    is_completed: bool,
    task_id: &str,
    request_content: Option<&str>,
) {
    fn parse_cascade(s: &str) -> Option<(serde_json::Value, serde_json::Value)> {
        let v: serde_json::Value = serde_json::from_str(s).ok()?;
        Some((v.get("stage1")?.clone(), v.get("stage2")?.clone()))
    }

    fn s2_resolution(s2: &serde_json::Value) -> Option<String> {
        s2.get("resolution")
            .or_else(|| s2.pointer("/result/resolution"))
            .and_then(|r| r.as_str())
            .map(|s| s.to_string())
    }

    fn apply_resolution(s: &str, res: &str) -> String {
        serde_json::from_str::<serde_json::Value>(s)
            .map(|mut v| {
                patch_json_fields_by_key(&mut v, &[("resolution", res)], &[]);
                v.to_string()
            })
            .unwrap_or_else(|_| s.to_string())
    }

    fn take_map(slot: &mut Option<String>, f: impl FnOnce(String) -> String) {
        if let Some(raw) = slot.take() {
            *slot = Some(f(raw));
        }
    }

    fn fold_post(raw: String) -> String {
        parse_cascade(&raw)
            .map(|(s1, _)| s1.to_string())
            .unwrap_or(raw)
    }

    let has_cascade = plugin_tag
        .map(|t| t.contains("\"cascade\""))
        .unwrap_or(false);
    // 仅真实级联配置才取目标分辨率，避免无 cascade 的 plugin_tag 被默认成 720p
    let target_res: Option<String> = has_cascade
        .then(|| cascade_resolve_target_resolution(plugin_tag.unwrap_or(""), ""))
        .filter(|r| !r.is_empty());

    let cascade_inflight = !is_completed
        && (has_cascade
            || post_resp.as_deref().and_then(parse_cascade).is_some()
            || response.as_deref().and_then(parse_cascade).is_some());

    if cascade_inflight {
        let s1_ack = post_resp
            .as_deref()
            .and_then(|s| {
                parse_cascade(s)
                    .map(|(s1, _)| s1)
                    .or_else(|| serde_json::from_str(s).ok())
            })
            .unwrap_or_else(|| serde_json::json!({}));
        let tid = if !task_id.is_empty() {
            task_id
        } else {
            s1_ack.get("id").and_then(|v| v.as_str()).unwrap_or("")
        };
        *response = Some(cascade_user_processing_response(
            &s1_ack,
            tid,
            request_content,
            target_res.as_deref(),
        ));
        take_map(post_resp, |raw| {
            let mut folded = fold_post(raw);
            response_formatter::force_json_task_id(&mut folded, tid);
            folded
        });
        return;
    }

    if is_completed {
        take_map(response, |raw| {
            let mut out = if let Some((s1, s2)) = parse_cascade(&raw) {
                let mut merged = cascade_s1_with_s2_url(&s1, &s2, plugin_tag.unwrap_or(""));
                if let Some(ref res) = s2_resolution(&s2).or_else(|| target_res.clone()) {
                    patch_json_fields_by_key(&mut merged, &[("resolution", res)], &[]);
                }
                merged.to_string()
            } else if let Some(ref res) = target_res {
                apply_resolution(&raw, res)
            } else {
                raw
            };
            if has_cascade {
                response_formatter::force_json_task_id(&mut out, task_id);
            }
            out
        });
    }
    take_map(post_resp, |raw| {
        let mut folded = fold_post(raw);
        if has_cascade {
            response_formatter::force_json_task_id(&mut folded, task_id);
        }
        folded
    });
}

/// 从 plugin_tag.cascade 还原阶段二轮询目标（渠道 + 转发配置 + 模型）
fn cascade_stage2_poll_target(
    channel: &Channel,
    resolved: &forward::ResolvedForward,
    plugin_tag: &str,
    stage2_task_id: &str,
) -> (Channel, forward::ResolvedForward, String) {
    let tag_json: serde_json::Value =
        serde_json::from_str(plugin_tag).unwrap_or(serde_json::json!({}));
    let cascade_info = tag_json
        .get("cascade")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    let mut ch = channel.clone();
    ch.id = cascade_info
        .get("ch_id")
        .and_then(|v| v.as_i64())
        .unwrap_or(channel.id);
    ch.name = cascade_info
        .get("ch_name")
        .and_then(|v| v.as_str())
        .unwrap_or(&channel.name)
        .to_string();
    ch.base_url = cascade_info
        .get("base_url")
        .and_then(|v| v.as_str())
        .unwrap_or(&channel.base_url)
        .to_string();
    ch.api_key = cascade_info
        .get("api_key")
        .and_then(|v| v.as_str())
        .unwrap_or(&channel.api_key)
        .to_string();
    ch.rate = cascade_info
        .get("rate")
        .and_then(|v| v.as_f64())
        .unwrap_or(channel.rate);

    let mut res = resolved.clone();
    res.mid = cascade_info
        .get("mid")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    res.auth_type = cascade_info
        .get("auth_type")
        .and_then(|v| v.as_str())
        .unwrap_or(&resolved.auth_type)
        .to_string();
    res.upstream_path = cascade_info
        .get("upstream_path")
        .and_then(|v| v.as_str())
        .unwrap_or(&resolved.upstream_path)
        .to_string();
    res.target_type = cascade_info
        .get("target_type")
        .and_then(|v| v.as_str())
        .unwrap_or(&resolved.target_type)
        .to_string();
    res.poll_path = cascade_info
        .get("poll_path")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let final_model = cascade_info
        .get("final_model")
        .and_then(|v| v.as_str())
        .or_else(|| cascade_info.get("mid").and_then(|v| v.as_str()))
        .unwrap_or("vve-sd")
        .to_string();

    tracing::info!(
        "[Cascade S2] 轮询目标: 阶段2任务ID={}, 渠道={}, 模型ID={:?}, 最终模型={}",
        stage2_task_id,
        ch.name,
        res.mid,
        final_model
    );
    (ch, res, final_model)
}

/// 用户端「处理中」响应：POST ack 骨架，补 model/resolution，硬保证无产物 URL。
fn cascade_user_processing_response(
    stage1_submit: &serde_json::Value,
    task_id: &str,
    request_content: Option<&str>,
    target_res: Option<&str>,
) -> String {
    let mut s = stage1_submit
        .as_object()
        .map(|_| stage1_submit.to_string())
        .unwrap_or_default();
    cascade_apply_processing_status(&mut s, task_id, false);
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed == "{}" {
        s = serde_json::json!({"id": task_id, "status": "running"}).to_string();
    }
    if let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&s) {
        if let Some(obj) = v.as_object_mut() {
            if let Some(model) = request_content
                .and_then(|r| serde_json::from_str::<serde_json::Value>(r).ok())
                .and_then(|r| r.get("model")?.as_str().map(|m| m.to_string()))
            {
                obj.insert("model".to_string(), serde_json::json!(model));
            }
            if let Some(res) = target_res.filter(|r| !r.is_empty()) {
                obj.insert("resolution".to_string(), serde_json::json!(res));
            }
            s = serde_json::to_string(&v).unwrap_or(s);
        }
    }
    if serde_json::from_str::<serde_json::Value>(&s)
        .map(|v| !response_formatter::find_urls(&v).is_empty())
        .unwrap_or(false)
    {
        return serde_json::json!({"id": task_id, "status": "running"}).to_string();
    }
    s
}

/// 写入对外任务号，并将终态/空 status 改为处理中；去掉 content/usage 等产物字段。
fn cascade_apply_processing_status(s: &mut String, task_id: &str, openai_compatible: bool) {
    response_formatter::force_json_task_id(s, task_id);
    let Ok(mut v) = serde_json::from_str::<serde_json::Value>(s) else {
        return;
    };
    let Some(obj) = v.as_object_mut() else {
        return;
    };
    let st = obj.get("status").and_then(|x| x.as_str()).unwrap_or("");
    if st.is_empty() || matches!(normalize_task_status(st), "succeeded" | "failed") {
        obj.insert(
            "status".to_string(),
            serde_json::json!(if openai_compatible {
                "in_progress"
            } else {
                "running"
            }),
        );
    }
    for k in ["content", "output", "usage", "results", "data"] {
        obj.remove(k);
    }
    if let Ok(out) = serde_json::to_string(&v) {
        *s = out;
    }
}

/// 级联阶段二进行中：对外返回阶段一 POST 提交态（处理中）。
/// 禁止返回 S1 成功产物（含视频 URL）或 S2 增强接口原始响应。
pub(crate) fn cascade_s2_client_processing(
    raw_path: &str,
    category: &str,
    stage1_submit: &serde_json::Value,
    task_id: &str,
) -> String {
    let mut s = response_formatter::apply_format(
        raw_path,
        category,
        &stage1_submit.to_string(),
        false,
        Some(task_id),
    );
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed == "{}" {
        return serde_json::json!({"id": task_id, "status": "running"}).to_string();
    }
    cascade_apply_processing_status(
        &mut s,
        task_id,
        response_formatter::is_openai_compatible_path(raw_path),
    );
    s
}

/// 级联落库：stage1 + stage2 原始串 → combined JSON
pub(crate) fn cascade_combine_stages(s1: &serde_json::Value, s2_raw: &str) -> String {
    let s2: serde_json::Value = serde_json::from_str(s2_raw).unwrap_or(serde_json::json!(s2_raw));
    serde_json::json!({ "stage1": s1, "stage2": s2 }).to_string()
}

/// 阶段二成功：stage1 usage × res_mul
fn apply_cascade_res_mul_to_stage1(
    s1: &mut serde_json::Value,
    res_mul: &HashMap<String, f64>,
    plugin_tag: &str,
) {
    let res = cascade_resolve_target_resolution(plugin_tag, "");
    forward::scale_usage_in_json(s1, forward::lookup_res_mul(res_mul, &res));
}

/// 级联阶段二提交结果：Submitted=已提交超分；InProgress=他处正在裁剪/提交
pub(crate) enum CascadeS2SubmitOutcome {
    Submitted(String),
    InProgress,
}

/// 0=非级联 / 1=阶段一 / 2=阶段二
pub(crate) fn cascade_stage_num(is_cascade: bool, post: &serde_json::Value) -> u8 {
    if !is_cascade {
        0
    } else if post.get("stage2").is_some() {
        2
    } else {
        1
    }
}

/// 落库 response_content 是否为级联 combined（stage1+stage2）
#[inline]
pub(crate) fn cascade_is_combined_resp(v: &serde_json::Value) -> bool {
    v.get("stage1").is_some() && v.get("stage2").is_some()
}

/// 从级联 stage2 节点提取失败文案（字符串 / 错误体 / 兜底）
pub(crate) fn cascade_stage2_err_text(stage2: &serde_json::Value, fallback: &str) -> String {
    let raw = stage2
        .as_str()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            response_formatter::extract_error_message_from_value(stage2).filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| fallback.to_string());
    proxy::sanitize_error_message(&raw)
}

/// 解析阶段二轮询目标。
/// - `Ok(None)`：非阶段二
/// - `Ok(Some((channel, s2_task_id, resolved, model)))`：可轮询
/// - `Err(err_text)`：stage2 无有效任务 ID（调用方结案退费）
pub(crate) fn cascade_resolve_s2_poll(
    cascade_stage: u8,
    post_resp: &serde_json::Value,
    channel: &Channel,
    resolved: &forward::ResolvedForward,
    plugin_tag: &str,
) -> Result<Option<(Channel, String, forward::ResolvedForward, String)>, String> {
    if cascade_stage != 2 {
        return Ok(None);
    }
    let stage2_val = &post_resp["stage2"];
    let s2_id = response_formatter::find_id(stage2_val);
    if s2_id.is_empty() {
        return Err(cascade_stage2_err_text(
            stage2_val,
            "级联阶段二提交失败，无有效任务ID",
        ));
    }
    let (ch, res, model) = cascade_stage2_poll_target(channel, resolved, plugin_tag, &s2_id);
    Ok(Some((ch, s2_id, res, model)))
}

/// 阶段二成功对外体：S1 骨架换 S2 URL → apply_format → 固定用户侧 task_id
pub(crate) fn cascade_format_s2_succeeded(
    raw_path: &str,
    category: &str,
    plugin_tag: &str,
    s1: &serde_json::Value,
    s2: &serde_json::Value,
    task_id: &str,
) -> String {
    let new_stage1 = cascade_s1_with_s2_url(s1, s2, plugin_tag);
    let mut formatted = response_formatter::apply_format(
        raw_path,
        category,
        &new_stage1.to_string(),
        true,
        Some(task_id),
    );
    response_formatter::force_json_task_id(&mut formatted, task_id);
    formatted
}

/// 进程内互斥：占位成功则持有，Drop 时 remove（仅 stage2_submit 使用）
struct CascadeS2InflightGuard<'a> {
    map: &'a dashmap::DashMap<i64, ()>,
    id: i64,
}

impl<'a> CascadeS2InflightGuard<'a> {
    fn try_acquire(map: &'a dashmap::DashMap<i64, ()>, id: i64) -> Option<Self> {
        if map.insert(id, ()).is_some() {
            return None;
        }
        Some(Self { map, id })
    }
}

impl Drop for CascadeS2InflightGuard<'_> {
    fn drop(&mut self) {
        self.map.remove(&self.id);
    }
}

/// 级联阶段二提交：阶段一底座成功后向画质增强提交超分（GET / 后台共用）。
/// `crop_480p`：仅目标 720p 且底座 480 时是否走 MediaKit 裁剪（转发规则同名字段，缺省 true）。
pub(crate) async fn cascade_stage2_submit(
    state: &Arc<AppState>,
    user_id: &str,
    token_id: Option<i64>,
    task_id: &str,
    db_log_id: i64,
    log_post_response: &str,
    log_request_content: &str,
    log_upstream_req: &str,
    pre_deduction: f64,
    pre_deduct_gift: f64,
    stage1_channel: &Channel,
    base_video_url: &str,
    log_plugin_tag: &str,
    stage1_response: &str,
    crop_480p: bool,
) -> Result<CascadeS2SubmitOutcome, String> {
    let Some(_guard) = CascadeS2InflightGuard::try_acquire(&state.cascade_s2_inflight, db_log_id)
    else {
        tracing::info!("[Cascade S2] skip log_id={db_log_id}（并发互斥）");
        return Ok(CascadeS2SubmitOutcome::InProgress);
    };

    let post_resp: serde_json::Value =
        serde_json::from_str(log_post_response).unwrap_or(serde_json::json!({}));

    let mut updated_tag_opt: Option<String> = None;
    if !log_plugin_tag.is_empty() {
        if let Ok(mut pt) = serde_json::from_str::<serde_json::Value>(log_plugin_tag) {
            if let Some(cascade) = pt.get_mut("cascade").and_then(|v| v.as_object_mut()) {
                if cascade.remove("api_key").is_some() {
                    updated_tag_opt = Some(pt.to_string());
                }
            }
        }
    }
    // S1 轮询成功体：根 id 换成用户侧 cgt 再落库（后续 combine/展示同源）
    let mut s1_body = stage1_response.to_string();
    response_formatter::force_json_task_id(&mut s1_body, task_id);
    let s1_json: serde_json::Value =
        serde_json::from_str(&s1_body).unwrap_or(serde_json::json!({}));

    let write_error = |state: &Arc<AppState>,
                       err_msg: &str,
                       post_resp_json: &serde_json::Value,
                       s1: &serde_json::Value,
                       s2_raw: &str,
                       tag: &Option<String>,
                       upstream: Option<String>| {
        let state = state.clone();
        let err = err_msg.to_string();
        let updated = serde_json::json!({"stage1": post_resp_json, "stage2": s2_raw}).to_string();
        let s2_json: serde_json::Value =
            serde_json::from_str(s2_raw).unwrap_or(serde_json::json!(s2_raw));
        let resp_content = serde_json::json!({"stage1": s1, "stage2": s2_json}).to_string();
        let tag = tag.clone();
        let db_id = db_log_id;
        async move {
            let _ = sqlx::query(&state.db.format_query(
                "UPDATE logs SET post_response = ?, response_content = ?, error_message = ?, plugin_tag = COALESCE(?, plugin_tag), upstream_req_content = COALESCE(?, upstream_req_content) WHERE id = ?"
            )).bind(&updated).bind(&resp_content).bind(&err).bind(&tag).bind(&upstream).bind(db_id).execute(&state.db.pool).await;
        }
    };

    if base_video_url.is_empty() {
        let err_msg = "底座视频生成成功但未能获取到视频直链地址";
        execute_refund_tx(
            state,
            db_log_id,
            user_id,
            token_id,
            Some(stage1_channel.id),
            pre_deduction,
            pre_deduct_gift,
            err_msg,
            500,
        )
        .await;
        write_error(
            state,
            err_msg,
            &post_resp,
            &s1_json,
            err_msg,
            &updated_tag_opt,
            None,
        )
        .await;
        return Err(err_msg.to_string());
    }

    let seed_resolved = forward::ResolvedForward {
        target_type: "volcengine_media_enhance".to_string(),
        upstream_path: "/api/v1/tools/enhance-video".to_string(),
        auth_type: "volcengine_sign".to_string(),
        ..Default::default()
    };
    let (enhance_ch, mut volc_resolved, final_model) =
        cascade_stage2_poll_target(stage1_channel, &seed_resolved, log_plugin_tag, task_id);
    let volc_model_mid = volc_resolved
        .mid
        .get_or_insert_with(|| "vve-sd".to_string())
        .clone();

    let target_resolution = cascade_resolve_target_resolution(log_plugin_tag, log_request_content);
    let base_video_url = if crop_480p {
        let req_hint: serde_json::Value =
            serde_json::from_str(log_request_content).unwrap_or(serde_json::json!({}));
        let up_hint: serde_json::Value =
            serde_json::from_str(log_upstream_req).unwrap_or(serde_json::json!({}));
        let mk = CascadeMk {
            http: &state.http_client,
            ch: &enhance_ch,
            auth_type: &volc_resolved.auth_type,
        };
        cascade_ensure_standard_480p_video(
            &mk,
            base_video_url,
            &s1_json,
            &target_resolution,
            &[&up_hint, &req_hint],
        )
        .await
    } else {
        base_video_url.to_string()
    };

    let volc_url = forward::build_upstream_url(
        &enhance_ch.base_url,
        &volc_resolved,
        &final_model,
        &enhance_ch.api_key,
    );

    let mut volc_payload = serde_json::json!({
        "video_url": base_video_url,
        "fps": 24,
        "bitrate_level": "high"
    });
    cascade_s2_apply_resolution_param(&mut volc_payload, &target_resolution, &volc_model_mid);
    if let Some(tv) = forward::volc_enhance_tool_version(&volc_model_mid) {
        volc_payload["tool_version"] = serde_json::json!(tv);
        if tv == "standard" {
            let scene = cascade_json_str(log_plugin_tag, "/cascade/scene")
                .and_then(|s| cascade_scene_pair(&s))
                .unwrap_or("common");
            volc_payload["scene"] = serde_json::json!(scene);
        }
    }

    // 临时错最多 5 次；退避 10→20→40→60s（总睡眠约 130s，原固定 120s×4≈480s）
    let max_attempts = 5u32;
    let mut attempt = 0u32;

    let (stage2_id, post_json) = loop {
        attempt += 1;
        let mut volc_body = volc_payload.clone();
        let builder = state
            .http_client
            .post(&volc_url)
            .header("Content-Type", "application/json");
        let builder =
            crate::services::http_client::with_upstream_timeout(forward::apply_request_auth(
                builder,
                &volc_resolved,
                &enhance_ch.api_key,
                &mut volc_body,
                &enhance_ch.base_url,
            ));

        let (should_retry, err_msg, err_status, raw_text) = match builder.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let text = resp.text().await.unwrap_or_default();
                if status == 200 {
                    match cascade_s2_parse_post_200(&text) {
                        Ok(ok) => break ok,
                        Err(CascadeS2Post200Fail::Upstream(post)) => {
                            let err =
                                match response_formatter::extract_error_message_from_value(&post) {
                                    Some(m) if !m.is_empty() => format!(
                                        "火山增强提交失败: {}",
                                        proxy::sanitize_error_message(&m)
                                    ),
                                    _ => "火山增强提交失败（业务错误，无任务 ID）".to_string(),
                                };
                            (false, err, proxy::infer_error_status_code(&post), text)
                        }
                        Err(CascadeS2Post200Fail::MissingTaskId) => {
                            let snippet: String = text.chars().take(240).collect();
                            tracing::warn!(
                                "[Cascade S2 POST] HTTP200 无任务ID log_id={} url={} body={}",
                                db_log_id,
                                volc_url,
                                snippet
                            );
                            (
                                false,
                                "火山增强提交成功但未能解析到超分任务 ID".to_string(),
                                500,
                                text,
                            )
                        }
                    }
                } else {
                    let err_text_raw = proxy::extract_error_message(&text);
                    let err_text = proxy::sanitize_error_message(&if err_text_raw.is_empty() {
                        format!("火山增强提交失败 HTTP {}", status)
                    } else {
                        err_text_raw
                    });
                    const RETRY_CODES: &[&str] = &[
                        "requestlimitexceeded",
                        "internalserviceerror",
                        "downloadfileerror",
                        "abilityprocessingerror",
                        "serviceinitializingerror",
                        "internalservicetimeout",
                    ];
                    let retry = proxy::is_poll_transport_retryable(status)
                        || serde_json::from_str::<serde_json::Value>(&text)
                            .ok()
                            .and_then(|v| response_formatter::extract_error_code_from_value(&v))
                            .is_some_and(|code| {
                                let c = code.to_lowercase();
                                RETRY_CODES.iter().any(|&k| c.contains(k))
                            });
                    (retry, err_text, status, text)
                }
            }
            Err(e) => (
                true,
                proxy::sanitize_error_message(&format!("火山增强接口提交连接失败: {:?}", e)),
                502,
                String::new(),
            ),
        };

        if should_retry && attempt < max_attempts {
            let delay_secs = (10u64 << (attempt - 1).min(3)).min(60);
            tracing::warn!(
                "[Cascade S2 POST] 临时错误 {}/{}，{}s 后重试: {}",
                attempt,
                max_attempts,
                delay_secs,
                err_msg
            );
            tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
        } else {
            let err_status = proxy::normalize_error_http_status(err_status);
            tracing::error!(
                "[Cascade S2 POST] 终态失败 ({}/{}): log_id={}, status={}, err={}",
                attempt,
                max_attempts,
                db_log_id,
                err_status,
                err_msg
            );
            execute_refund_tx(
                state,
                db_log_id,
                user_id,
                token_id,
                Some(stage1_channel.id),
                pre_deduction,
                pre_deduct_gift,
                &err_msg,
                err_status,
            )
            .await;
            write_error(
                state,
                &err_msg,
                &post_resp,
                &s1_json,
                &raw_text,
                &updated_tag_opt,
                cascade_upstream_req_combined(log_upstream_req, &volc_payload),
            )
            .await;
            return Err(err_msg);
        }
    };

    let updated = serde_json::json!({"stage1": post_resp, "stage2": post_json}).to_string();
    let upstream_combined = cascade_upstream_req_combined(log_upstream_req, &volc_payload);
    let _ = sqlx::query(&state.db.format_query("UPDATE logs SET post_response = ?, response_content = ?, upstream_req_content = COALESCE(?, upstream_req_content) WHERE id = ?"))
        .bind(&updated).bind(&s1_body).bind(&upstream_combined).bind(db_log_id).execute(&state.db.pool).await;

    tracing::info!(
        "[Cascade S2] 级联提交成功 日志ID={} 阶段1={} 阶段2={} MID={} 分辨率={} 渠道={}",
        db_log_id,
        task_id,
        stage2_id,
        volc_model_mid,
        target_resolution,
        enhance_ch.name
    );
    Ok(CascadeS2SubmitOutcome::Submitted(stage2_id))
}
