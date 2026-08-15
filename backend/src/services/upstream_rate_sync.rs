/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

//! NewAPI 等上游分组倍率拉取与应用到渠道预设 `rate`。

use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

pub const SYSTEM_NEWAPI: &str = "newapi";

pub const UPSTREAM_SYSTEMS: &[&str] = &["兼容", "官方", "newapi", "akeapi", "火山引擎", "阿里云"];

#[derive(Debug, Clone, PartialEq)]
pub struct UpstreamGroupRatio {
    pub name: String,
    pub ratio: f64,
    pub label: String,
}

#[derive(Debug, Deserialize)]
struct NewapiPricingBody {
    #[serde(default)]
    success: Option<bool>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    group_ratio: Option<BTreeMap<String, f64>>,
    #[serde(default)]
    usable_group: Option<BTreeMap<String, String>>,
    #[serde(default)]
    data: Option<Value>,
}

pub fn is_known_upstream_system(value: &str) -> bool {
    value.is_empty() || UPSTREAM_SYSTEMS.contains(&value)
}

pub fn normalize_newapi_origin(base_url: &str) -> Result<String, String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("请填写端点基础地址".into());
    }
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err("端点基础地址必须以 http:// 或 https:// 开头".into());
    }
    let mut origin = trimmed.to_string();
    loop {
        let lower = origin.to_ascii_lowercase();
        let stripped = ["/v1", "/api"].iter().find_map(|suffix| {
            if origin.len() > suffix.len() && lower.ends_with(suffix) {
                Some(
                    origin[..origin.len() - suffix.len()]
                        .trim_end_matches('/')
                        .to_string(),
                )
            } else {
                None
            }
        });
        match stripped {
            Some(next) => origin = next,
            None => break,
        }
    }
    Ok(origin)
}

pub fn newapi_pricing_url(base_url: &str) -> Result<String, String> {
    Ok(format!("{}/api/pricing", normalize_newapi_origin(base_url)?))
}

pub fn applied_channel_rate(group_ratio: f64, rate_add: f64) -> f64 {
    let add = if rate_add.is_finite() && rate_add > 0.0 {
        rate_add
    } else {
        0.0
    };
    let ratio = if group_ratio.is_finite() {
        group_ratio
    } else {
        0.0
    };
    (ratio + add).max(0.0)
}

pub fn is_sync_due(synced_at: Option<&str>, interval_minutes: i32, now: DateTime<Utc>) -> bool {
    if interval_minutes <= 0 {
        return false;
    }
    let Some(raw) = synced_at.map(str::trim).filter(|s| !s.is_empty()) else {
        return true;
    };
    let Ok(last) = DateTime::parse_from_rfc3339(raw) else {
        return true;
    };
    now.signed_duration_since(last.with_timezone(&Utc)) >= Duration::minutes(interval_minutes as i64)
}

pub fn parse_newapi_groups(body: &str) -> Result<Vec<UpstreamGroupRatio>, String> {
    let parsed: NewapiPricingBody =
        serde_json::from_str(body).map_err(|e| format!("上游定价响应不是 JSON: {e}"))?;
    if parsed.success == Some(false) {
        let msg = parsed
            .message
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| "上游返回失败".into());
        return Err(msg);
    }

    let mut ratios = parsed.group_ratio.unwrap_or_default();
    let mut labels = parsed.usable_group.unwrap_or_default();
    if let Some(Value::Object(data)) = parsed.data {
        if ratios.is_empty() {
            if let Some(Value::Object(map)) = data.get("group_ratio") {
                for (k, v) in map {
                    if let Some(n) = v.as_f64() {
                        ratios.insert(k.clone(), n);
                    }
                }
            }
        }
        if labels.is_empty() {
            if let Some(Value::Object(map)) = data.get("usable_group") {
                for (k, v) in map {
                    if let Some(s) = v.as_str() {
                        labels.insert(k.clone(), s.to_string());
                    }
                }
            }
        }
    }

    if ratios.is_empty() {
        return Err("上游未返回分组倍率".into());
    }

    let mut groups: Vec<UpstreamGroupRatio> = ratios
        .into_iter()
        .map(|(name, ratio)| {
            let label = labels.get(&name).cloned().unwrap_or_default();
            UpstreamGroupRatio {
                name,
                ratio,
                label,
            }
        })
        .collect();
    groups.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(groups)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn strips_v1_suffix_for_pricing_url() {
        assert_eq!(
            newapi_pricing_url("https://api.hoxkai.top/v1/").unwrap(),
            "https://api.hoxkai.top/api/pricing"
        );
        assert_eq!(
            newapi_pricing_url("https://api.hoxkai.top").unwrap(),
            "https://api.hoxkai.top/api/pricing"
        );
    }

    #[test]
    fn rejects_empty_base() {
        assert!(normalize_newapi_origin("  ").is_err());
    }

    #[test]
    fn parses_top_level_group_ratio() {
        let body = r#"{"success":true,"group_ratio":{"grok":1.6,"codex":0.12},"usable_group":{"grok":"grok"}}"#;
        let groups = parse_newapi_groups(body).unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[1].name, "grok");
        assert_eq!(groups[1].ratio, 1.6);
        assert_eq!(groups[1].label, "grok");
    }

    #[test]
    fn parses_nested_data_group_ratio() {
        let body = r#"{"data":{"group_ratio":{"anti":0.3}}}"#;
        let groups = parse_newapi_groups(body).unwrap();
        assert_eq!(groups[0].name, "anti");
        assert_eq!(groups[0].ratio, 0.3);
    }

    #[test]
    fn applied_rate_adds_positive_delta() {
        assert_eq!(applied_channel_rate(1.45, 0.1), 1.55);
        assert_eq!(applied_channel_rate(1.45, 0.0), 1.45);
        assert_eq!(applied_channel_rate(1.45, -1.0), 1.45);
    }

    #[test]
    fn sync_due_when_never_synced_or_interval_elapsed() {
        let now = Utc.with_ymd_and_hms(2026, 8, 14, 6, 0, 0).unwrap();
        assert!(is_sync_due(None, 10, now));
        assert!(!is_sync_due(Some("2026-08-14T05:55:00.000Z"), 10, now));
        assert!(is_sync_due(Some("2026-08-14T05:50:00.000Z"), 10, now));
        assert!(!is_sync_due(Some("2026-08-14T05:50:00.000Z"), 0, now));
    }
}
