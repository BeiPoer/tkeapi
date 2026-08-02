/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

use crate::auth;
use crate::models::{
    CreateRedemptionRequest, RedeemRequest, Redemption, RedemptionGroupResponse,
    RedemptionListResponse,
};
use crate::AppState;
use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use dashmap::DashMap;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use crate::error::{AppError, AppResult};

use rand::{distributions::Alphanumeric, Rng};

/// Admin: List all redemption codes
pub async fn list_redemptions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<auth::Claims>,
    Query(query): Query<crate::models::RedemptionQuery>,
) -> AppResult<Json<RedemptionListResponse>> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden("Admin only".to_string()));
    }

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).max(1).min(100);
    let offset = (page - 1) * page_size;

    let mut qb = sqlx::QueryBuilder::new("SELECT * FROM redemptions WHERE 1=1 ");
    if let Some(name) = &query.name {
        qb.push(" AND name = ");
        qb.push_bind(name);
    }
    qb.push(" ORDER BY id DESC LIMIT ");
    qb.push_bind(page_size);
    qb.push(" OFFSET ");
    qb.push_bind(offset);

    let redemptions: Vec<Redemption> = qb.build_query_as().fetch_all(&state.db.pool).await?;

    let mut count_qb = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM redemptions WHERE 1=1 ");
    if let Some(name) = &query.name {
        count_qb.push(" AND name = ");
        count_qb.push_bind(name);
    }
    let total: i64 = count_qb
        .build_query_scalar()
        .fetch_one(&state.db.pool)
        .await?;

    Ok(Json(RedemptionListResponse {
        data: redemptions,
        total,
    }))
}

pub async fn list_redemption_groups(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<auth::Claims>,
    Query(query): Query<crate::models::RedemptionQuery>,
) -> AppResult<Json<RedemptionGroupResponse>> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden("Admin only".to_string()));
    }

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).max(1).min(100);
    let offset = (page - 1) * page_size;

    let groups: Vec<crate::models::RedemptionGroup> = sqlx::query_as(&state.db.format_query(
        "SELECT \
             name, \
             COUNT(id) as total_count, \
             SUM(quota) as total_quota, \
             MAX(created_at) as created_at, \
             MAX(expires_at) as expires_at, \
             SUM(used_count) as total_used_count, \
             MAX(max_uses) as max_uses, \
             MAX(per_user_limit) as per_user_limit \
             FROM redemptions \
             GROUP BY name \
             ORDER BY MAX(created_at) DESC \
             LIMIT ? OFFSET ?",
    ))
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.db.pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        &state
            .db
            .format_query("SELECT COUNT(DISTINCT name) FROM redemptions"),
    )
    .fetch_one(&state.db.pool)
    .await?;

    Ok(Json(RedemptionGroupResponse {
        data: groups,
        total,
    }))
}

/// Admin: Bulk generate redemption codes
pub async fn generate_redemptions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<auth::Claims>,
    Json(request): Json<CreateRedemptionRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden("Admin only".to_string()));
    }

    if request.count <= 0 || request.count > 1000 {
        return Err(AppError::BadRequest(
            "Count must be between 1 and 1000".to_string(),
        ));
    }
    if request.quota <= 0.0 {
        return Err(AppError::BadRequest("额度必须大于 0".to_string()));
    }

    let (tz_name, _) = crate::relay::get_cached_config(&state).await;
    let expires_at = if request.permanent {
        None
    } else {
        let Some(ref exp) = request.expires_at else {
            return Err(AppError::BadRequest(
                "请设置有效期，或选择长期有效".to_string(),
            ));
        };
        Some(normalize_expiry_date(exp, &tz_name)?)
    };

    let (max_uses, per_user_limit) = if request.allow_multiple {
        if request.max_uses < -1 || request.per_user_limit < -1 {
            return Err(AppError::BadRequest(
                "兑换次数无效（-1 表示不限制）".to_string(),
            ));
        }
        // 约定 -1 = 不限；兼容前端/历史传入的 0，统一落库为 -1
        (
            if request.max_uses == 0 {
                -1
            } else {
                request.max_uses
            },
            if request.per_user_limit == 0 {
                -1
            } else {
                request.per_user_limit
            },
        )
    } else {
        (1, 1)
    };

    let mut codes = Vec::new();
    let mut unique_codes = std::collections::HashSet::new();
    let mut tx = state.db.pool.begin().await?;

    for _ in 0..request.count {
        // 8 位大写字母+数字，便于复制使用；冲突时重试
        for attempt in 0..16 {
            let code: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(8)
                .map(|c| (c as char).to_ascii_uppercase())
                .collect();

            if !unique_codes.contains(&code) {
                let exists: Option<i64> = sqlx::query_scalar(
                    &state
                        .db
                        .format_query("SELECT id FROM redemptions WHERE code = ? LIMIT 1"),
                )
                .bind(&code)
                .fetch_optional(&mut *tx)
                .await?;
                if exists.is_none() {
                    unique_codes.insert(code.clone());
                    codes.push(code);
                    break;
                }
            }

            if attempt == 15 {
                return Err(AppError::BadRequest("生成兑换码失败，请重试".to_string()));
            }
        }
    }

    if !codes.is_empty() {
        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO redemptions (name, code, quota, expires_at, max_uses, used_count, per_user_limit, is_used) "
        );
        let name = request.name.clone();
        let q = request.quota;
        query_builder.push_values(codes.clone(), |mut b, code| {
            b.push_bind(name.clone())
                .push_bind(code)
                .push_bind(q)
                .push_bind(expires_at.clone())
                .push_bind(max_uses)
                .push_bind(0)
                .push_bind(per_user_limit)
                .push_bind(0);
        });

        let query = query_builder.build();
        query.execute(&mut *tx).await?;
    }

    tx.commit().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "count": request.count,
        "codes": codes
    })))
}

/// Admin: Delete a redemption code
pub async fn delete_redemption(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<auth::Claims>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden("Admin only".to_string()));
    }

    sqlx::query(
        &state
            .db
            .format_query("DELETE FROM redemptions WHERE id = ?"),
    )
    .bind(id)
    .execute(&state.db.pool)
    .await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Admin: Update redemption status (void/disable)
pub async fn update_redemption_status(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<auth::Claims>,
    Path(id): Path<i64>,
    Json(request): Json<crate::models::UpdateRedemptionStatusRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden("Admin only".to_string()));
    }

    if request.status != 1 && request.status != 0 && request.status != -1 {
        return Err(AppError::BadRequest("Invalid status".to_string()));
    }

    let rows = sqlx::query(
        &state
            .db
            .format_query("UPDATE redemptions SET status = ? WHERE id = ?"),
    )
    .bind(request.status)
    .bind(id)
    .execute(&state.db.pool)
    .await?;

    if rows.rows_affected() == 0 {
        return Err(AppError::NotFound("Redemption code not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

#[derive(serde::Deserialize)]
pub struct DeleteGroupQuery {
    pub name: String,
}

pub async fn delete_redemption_group(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<auth::Claims>,
    Query(query): Query<DeleteGroupQuery>,
) -> AppResult<Json<serde_json::Value>> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden("Admin only".to_string()));
    }

    if query.name.trim().is_empty() {
        return Err(AppError::BadRequest("Activity name required".to_string()));
    }

    sqlx::query(
        &state
            .db
            .format_query("DELETE FROM redemptions WHERE name = ?"),
    )
    .bind(&query.name)
    .execute(&state.db.pool)
    .await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// 将有效期规范为落库的 DbTs（按站点时区计算过期瞬间的 UTC 时间）
fn normalize_expiry_date(raw: &str, tz_name: &str) -> AppResult<crate::time_system::DbTs> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(
            "请设置有效期，或选择长期有效".to_string(),
        ));
    }
    let tz: chrono_tz::Tz = tz_name.parse().unwrap_or(chrono_tz::Asia::Shanghai);

    let mut dt_opt = None;

    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        dt_opt = Some(dt.with_timezone(&chrono::Utc));
    } else if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S") {
        if let Some(local) = ndt.and_local_timezone(tz).latest() {
            dt_opt = Some(local.with_timezone(&chrono::Utc));
        }
    } else if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
        if let Some(local) = ndt.and_local_timezone(tz).latest() {
            dt_opt = Some(local.with_timezone(&chrono::Utc));
        }
    } else if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        if let Some(next) = d.succ_opt() {
            if let Some(end) = next
                .and_hms_opt(0, 0, 0)
                .and_then(|ndt| ndt.and_local_timezone(tz).latest())
            {
                dt_opt = Some(end.with_timezone(&chrono::Utc));
            }
        }
    }

    if let Some(dt) = dt_opt {
        Ok(crate::time_system::DbTs::from_utc(dt))
    } else {
        Err(AppError::BadRequest("有效期格式不正确".to_string()))
    }
}

/// 过期判定：无时区字符串按站点默认时区墙钟解释；纯日期则该日结束（次日 00:00）前仍有效。
fn is_expired(expires_at: Option<&str>, tz_name: &str) -> bool {
    let Some(exp) = expires_at.map(str::trim).filter(|s| !s.is_empty()) else {
        return false; // 长期有效
    };
    let tz: chrono_tz::Tz = tz_name.parse().unwrap_or(chrono_tz::Asia::Shanghai);
    let now = chrono::Utc::now().with_timezone(&tz);

    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(exp) {
        return dt.with_timezone(&tz) < now;
    }
    if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(exp, "%Y-%m-%d %H:%M:%S") {
        if let Some(local) = ndt.and_local_timezone(tz).latest() {
            return local < now;
        }
    }
    if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(exp, "%Y-%m-%dT%H:%M:%S") {
        if let Some(local) = ndt.and_local_timezone(tz).latest() {
            return local < now;
        }
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(exp, "%Y-%m-%d") {
        // 日期当天结束仍有效：站点时区下「过期日 + 1 天 00:00」起算过期
        if let Some(next) = d.succ_opt() {
            if let Some(end) = next
                .and_hms_opt(0, 0, 0)
                .and_then(|ndt| ndt.and_local_timezone(tz).latest())
            {
                return now >= end;
            }
        }
    }
    false
}

/// User: Redeem a code to balance
pub async fn redeem_code(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<auth::Claims>,
    Json(request): Json<RedeemRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let settings = crate::api::settings::load_all_settings(&state).await?;
    if !settings.marketing.enable_redemption {
        return Err(AppError::Forbidden("兑换功能未开启".to_string()));
    }

    let code = request.code.trim().to_ascii_uppercase();
    if code.is_empty() {
        return Err(AppError::BadRequest("请输入兑换码".to_string()));
    }

    let user_id = claims.sub;

    // Rate Limiting (5 requests / min per user_id)
    static REDEEM_RL: OnceLock<DashMap<String, (u32, Instant)>> = OnceLock::new();
    let rl = REDEEM_RL.get_or_init(|| DashMap::new());
    let now = Instant::now();
    let mut allowed = true;
    if let Some(mut entry) = rl.get_mut(&user_id) {
        let (count, last_reset) = *entry;
        if now.duration_since(last_reset) > Duration::from_secs(60) {
            *entry = (1, now);
        } else if count >= 5 {
            allowed = false;
        } else {
            *entry = (count + 1, last_reset);
        }
    } else {
        rl.insert(user_id.clone(), (1, now));
    }

    if !allowed {
        return Err(AppError::BadRequest("请求过于频繁，请稍后再试".to_string()));
    }

    // Start transaction to ensure atomicity
    let mut tx = state.db.pool.begin().await?;

    let existing: Option<Redemption> = sqlx::query_as(
        &state
            .db
            .format_query("SELECT * FROM redemptions WHERE code = ? LIMIT 1"),
    )
    .bind(&code)
    .fetch_optional(&mut *tx)
    .await?;

    let redemption = match existing {
        None => {
            return Err(AppError::BadRequest("兑换码无效，请检查后重试".to_string()));
        }
        Some(r) => r,
    };

    if redemption.status == 0 {
        return Err(AppError::BadRequest("该兑换码已被禁用".to_string()));
    }
    if redemption.status == -1 {
        return Err(AppError::BadRequest("该兑换码已作废".to_string()));
    }

    let tz_name = {
        let t = settings.site.default_timezone.trim();
        if t.is_empty() {
            "Asia/Shanghai".to_string()
        } else {
            t.to_string()
        }
    };
    if is_expired(redemption.expires_at.as_deref(), &tz_name) {
        return Err(AppError::BadRequest("兑换码已过期".to_string()));
    }

    // 单兑换码次数：<=0（-1 约定 / 历史 0）表示不限；兼容旧单次码 is_used
    let max_uses = redemption.max_uses;
    if max_uses > 0 && redemption.used_count >= max_uses {
        return Err(AppError::BadRequest("该兑换码兑换次数已用完".to_string()));
    }
    if max_uses == 1 && redemption.is_used != 0 && redemption.used_count == 0 {
        // 旧数据兜底
        return Err(AppError::BadRequest("该兑换码已被使用".to_string()));
    }

    // 单兑换码单用户限制：<=0（-1 约定 / 历史 0）表示不限
    let per_user_limit = redemption.per_user_limit;
    if per_user_limit > 0 {
        let user_used: i64 = sqlx::query_scalar(&state.db.format_query(
            "SELECT COUNT(*) FROM redemption_logs WHERE redemption_id = ? AND user_id = ?",
        ))
        .bind(redemption.id)
        .bind(&user_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap_or(0);

        // 兼容旧单次码：无 logs 时用 used_by 判断
        let legacy_used = redemption.used_by.as_deref() == Some(user_id.as_str())
            && user_used == 0
            && redemption.is_used != 0
            && max_uses == 1;

        if legacy_used || user_used >= per_user_limit as i64 {
            return Err(AppError::BadRequest(
                "您已达到该兑换码的兑换次数上限".to_string(),
            ));
        }
    }

    // 乐观锁：原子更新主表，防止超发
    let rows_affected = sqlx::query(&state.db.format_query(
        "UPDATE redemptions SET \
         used_count = used_count + 1, \
         is_used = CASE WHEN ? > 0 AND used_count + 1 >= ? THEN 1 ELSE is_used END, \
         used_at = CURRENT_TIMESTAMP, \
         used_by = CASE WHEN ? = 1 THEN ? ELSE used_by END, \
         updated_at = CURRENT_TIMESTAMP \
         WHERE id = ? AND (? <= 0 OR used_count < ?)",
    ))
    .bind(max_uses)
    .bind(max_uses)
    .bind(max_uses)
    .bind(&user_id)
    .bind(redemption.id)
    .bind(max_uses)
    .bind(max_uses)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::BadRequest(
            "该兑换码兑换次数已用完，或已被他人抢先兑换".to_string(),
        ));
    }

    // 入账
    sqlx::query(&state.db.format_query(
        "UPDATE users SET balance = balance + ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
    ))
    .bind(redemption.quota)
    .bind(&user_id)
    .execute(&mut *tx)
    .await?;

    // 写兑换日志
    sqlx::query(&state.db.format_query(
        "INSERT INTO redemption_logs (redemption_id, user_id, amount) VALUES (?, ?, ?)",
    ))
    .bind(redemption.id)
    .bind(&user_id)
    .bind(redemption.quota)
    .execute(&mut *tx)
    .await?;

    // 充值记录
    let recharge_id: i64 = sqlx::query_scalar::<_, i64>(
        &state.db.format_query("INSERT INTO recharge_records (user_id, amount, recharge_type, remark) VALUES (?, ?, 'redemption', ?) RETURNING id")
    )
    .bind(&user_id)
    .bind(redemption.quota)
    .bind(format!("兑换码: {}", redemption.name))
    .fetch_one(&mut *tx)
    .await?;

    if let Err(e) = crate::services::affiliate::award_commission(
        &state.db,
        &mut tx,
        &user_id,
        recharge_id,
        redemption.quota,
    )
    .await
    {
        tracing::error!(
            "Failed to award commission for redemption {}: {}",
            recharge_id,
            e
        );
    }

    tx.commit().await?;

    crate::services::notification::spawn_low_balance_check(Arc::clone(&state), user_id.clone());

    Ok(Json(serde_json::json!({
        "success": true,
        "quota_added": redemption.quota
    })))
}
