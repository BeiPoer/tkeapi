/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

//! Relay 上游响应头透传：过滤 hop-by-hop / body 绑定头后挂到客户端 Response。
//! 网关会重建 body，故不可原样转发 content-length / content-encoding。

use axum::http::header::{self, HeaderName, HeaderValue};
use axum::http::HeaderMap;
use axum::response::Response;

/// 是否应将上游响应头转发给客户端。
#[inline]
fn should_forward_response_header(name: &HeaderName) -> bool {
    !matches!(
        name.as_str(),
        "connection"
            | "keep-alive"
            | "proxy-connection"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailers"
            | "transfer-encoding"
            | "upgrade"
            | "content-length"
            | "content-encoding"
            | "content-type"
            | "set-cookie"
    )
}

/// 读取上游响应头字符串值。
#[inline]
pub fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

/// 上游 Content-Type 是否为 SSE。
#[inline]
pub fn is_sse(headers: &HeaderMap) -> bool {
    header_str(headers, "content-type").is_some_and(|s| s.contains("text/event-stream"))
}

/// 上游 Content-Type 是否为可流式消费（SSE / NDJSON）。
#[inline]
pub fn is_stream_content_type(headers: &HeaderMap) -> bool {
    header_str(headers, "content-type")
        .is_some_and(|s| s.contains("text/event-stream") || s.contains("application/x-ndjson"))
}

/// 将上游可转发头挂到 Response builder。
fn apply_upstream_response_headers(
    mut builder: axum::http::response::Builder,
    upstream: &HeaderMap,
) -> axum::http::response::Builder {
    for (name, value) in upstream
        .iter()
        .filter(|(n, _)| should_forward_response_header(n))
    {
        builder = builder.header(name, value);
    }
    builder
}

/// 将上游可转发头写入已构建 Response 的 HeaderMap（错误路径等）。
pub fn merge_upstream_response_headers(dest: &mut HeaderMap, upstream: &HeaderMap) {
    for (name, value) in upstream
        .iter()
        .filter(|(n, _)| should_forward_response_header(n))
    {
        dest.append(name.clone(), value.clone());
    }
}

/// 透传上游诊断头并设置指定 Content-Type。
pub fn with_content_type(
    upstream: &HeaderMap,
    content_type: impl AsRef<str>,
    body: impl Into<axum::body::Body>,
) -> Response {
    apply_upstream_response_headers(Response::builder(), upstream)
        .header(header::CONTENT_TYPE, content_type.as_ref())
        .body(body.into())
        .unwrap()
}

/// 成功 JSON 响应：透传上游诊断头 + `application/json`。
#[inline]
pub fn json_with_upstream_headers(
    upstream: &HeaderMap,
    body: impl Into<axum::body::Body>,
) -> Response {
    with_content_type(upstream, "application/json", body)
}

/// SSE 响应：透传上游诊断头 + 网关流式固定头（insert 覆盖，避免与上游 Cache-Control 重复）。
pub fn sse_with_upstream_headers(
    upstream: &HeaderMap,
    body: impl Into<axum::body::Body>,
) -> Response {
    let mut resp = with_content_type(upstream, "text/event-stream", body);
    let h = resp.headers_mut();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    h.insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));
    resp
}
