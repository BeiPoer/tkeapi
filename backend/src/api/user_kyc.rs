/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

use crate::api::settings::default_registration_settings;
use crate::auth;
use crate::error::{AppError, AppResult};
use crate::models::{
    normalize_id_doc_type, normalize_kyc_status, normalize_kyc_type, normalize_validity_type,
    RegistrationSettings, UpsertUserKycRequest, UserKyc,
};
use crate::time_system::DbTs;
use crate::AppState;
use axum::{
    extract::{Extension, Multipart, Path, State},
    Json,
};
use std::sync::Arc;

async fn load_registration(state: &AppState) -> AppResult<RegistrationSettings> {
    let val: Option<String> = sqlx::query_scalar(
        &state
            .db
            .format_query("SELECT value FROM settings WHERE key = ?"),
    )
    .bind("registration_settings")
    .fetch_optional(&state.db.pool)
    .await?;
    Ok(val
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_else(default_registration_settings))
}

async fn ensure_kyc_enabled(state: &AppState) -> AppResult<()> {
    let reg = load_registration(state).await?;
    if !reg.enable_user_kyc {
        return Err(AppError::BadRequest(
            "站点未开启用户实名认证功能".to_string(),
        ));
    }
    Ok(())
}

async fn fetch_kyc(state: &AppState, user_id: &str) -> AppResult<UserKyc> {
    let row: Option<UserKyc> = sqlx::query_as(
        &state
            .db
            .format_query("SELECT * FROM user_kyc WHERE user_id = ?"),
    )
    .bind(user_id)
    .fetch_optional(&state.db.pool)
    .await?;
    Ok(row.unwrap_or_else(|| UserKyc::empty_for(user_id)))
}

fn empty_to_none(s: String) -> Option<String> {
    let t = s.trim().to_string();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

fn parse_expire_at(raw: Option<&String>, validity_type: &str) -> AppResult<Option<DbTs>> {
    if validity_type != "expire_date" {
        return Ok(None);
    }
    let Some(s) = raw.map(|v| v.trim()).filter(|v| !v.is_empty()) else {
        return Err(AppError::BadRequest(
            "证件有效期模式下请填写到期日期".to_string(),
        ));
    };
    // 允许日期或完整时间戳
    let normalized = if s.len() == 10 {
        format!("{s}T23:59:59.000Z")
    } else {
        s.to_string()
    };
    let ts = DbTs::new(normalized);
    if ts.to_utc().is_none() {
        return Err(AppError::BadRequest("无效的证件到期日期".to_string()));
    }
    Ok(Some(ts))
}

fn validate_payload(
    kyc_type: &str,
    id_doc_type: Option<&str>,
    real_name: Option<&str>,
    company_name: Option<&str>,
    id_doc_front_url: Option<&str>,
    id_doc_back_url: Option<&str>,
    business_license_url: Option<&str>,
    tax_registration_url: Option<&str>,
    legal_notarization_url: Option<&str>,
    for_submit: bool,
) -> AppResult<()> {
    if !for_submit {
        return Ok(());
    }
    match kyc_type {
        "personal" => {
            if real_name.map(|s| s.trim().is_empty()).unwrap_or(true) {
                return Err(AppError::BadRequest("请填写真实姓名".to_string()));
            }
            let doc = id_doc_type.ok_or_else(|| {
                AppError::BadRequest("请选择证件类型（身份证/护照/驾照）".to_string())
            })?;
            if id_doc_front_url
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
            {
                return Err(AppError::BadRequest("请上传证件正面/主页照片".to_string()));
            }
            if doc == "id_card" && id_doc_back_url.map(|s| s.trim().is_empty()).unwrap_or(true) {
                return Err(AppError::BadRequest("身份证请上传正反面照片".to_string()));
            }
        }
        "enterprise" => {
            if company_name.map(|s| s.trim().is_empty()).unwrap_or(true) {
                return Err(AppError::BadRequest("请填写企业名称".to_string()));
            }
            if business_license_url
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
            {
                return Err(AppError::BadRequest("请上传营业执照".to_string()));
            }
            if tax_registration_url
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
            {
                return Err(AppError::BadRequest("请上传税务登记证".to_string()));
            }
            if legal_notarization_url
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
            {
                return Err(AppError::BadRequest("请上传企业法务公证材料".to_string()));
            }
        }
        _ => return Err(AppError::BadRequest("无效的实名类型".to_string())),
    }
    Ok(())
}

async fn upsert_kyc(
    state: &AppState,
    user_id: &str,
    req: UpsertUserKycRequest,
    is_admin: bool,
    operator: Option<&str>,
) -> AppResult<UserKyc> {
    let existing = fetch_kyc(state, user_id).await?;

    let kyc_type = match req.kyc_type.as_deref() {
        Some(v) => normalize_kyc_type(v)
            .ok_or_else(|| AppError::BadRequest("实名类型仅支持个人或企业".to_string()))?
            .to_string(),
        None => existing.kyc_type.clone(),
    };

    let validity_type = match req.validity_type.as_deref() {
        Some(v) => normalize_validity_type(v)
            .ok_or_else(|| AppError::BadRequest("有效期类型无效".to_string()))?
            .to_string(),
        None => existing.validity_type.clone(),
    };

    let real_name = match req.real_name {
        Some(s) => empty_to_none(s),
        None => existing.real_name.clone(),
    };
    let id_doc_front_url = match req.id_doc_front_url {
        Some(s) => empty_to_none(s),
        None => existing.id_doc_front_url.clone(),
    };
    let id_doc_back_url = match req.id_doc_back_url {
        Some(s) => empty_to_none(s),
        None => existing.id_doc_back_url.clone(),
    };
    let company_name = match req.company_name {
        Some(s) => empty_to_none(s),
        None => existing.company_name.clone(),
    };
    let business_license_url = match req.business_license_url {
        Some(s) => empty_to_none(s),
        None => existing.business_license_url.clone(),
    };
    let tax_registration_url = match req.tax_registration_url {
        Some(s) => empty_to_none(s),
        None => existing.tax_registration_url.clone(),
    };
    let legal_notarization_url = match req.legal_notarization_url {
        Some(s) => empty_to_none(s),
        None => existing.legal_notarization_url.clone(),
    };

    let id_doc_type = match req.id_doc_type.as_deref() {
        Some("") => None,
        Some(v) => Some(
            normalize_id_doc_type(v)
                .ok_or_else(|| AppError::BadRequest("证件类型无效".to_string()))?
                .to_string(),
        ),
        None => existing.id_doc_type.clone(),
    };

    let expire_at = if req.validity_type.is_some() || req.expire_at.is_some() {
        parse_expire_at(req.expire_at.as_ref(), &validity_type)?
    } else if validity_type == "long_term" {
        None
    } else {
        existing.expire_at.clone()
    };

    let (status, reject_reason, admin_remark, reviewed_by, reviewed_at, submitted_at) = if is_admin
    {
        let status = match req.status.as_deref() {
            Some(v) => normalize_kyc_status(v)
                .ok_or_else(|| AppError::BadRequest("实名状态无效".to_string()))?
                .to_string(),
            None => {
                if existing.status == "none" {
                    "approved".to_string()
                } else {
                    existing.status.clone()
                }
            }
        };
        let reject_reason = match req.reject_reason {
            Some(s) => empty_to_none(s),
            None => existing.reject_reason.clone(),
        };
        let admin_remark = match req.admin_remark {
            Some(s) => empty_to_none(s),
            None => existing.admin_remark.clone(),
        };
        let (reviewed_by, reviewed_at) = if matches!(status.as_str(), "approved" | "rejected") {
            (
                operator
                    .map(|s| s.to_string())
                    .or(existing.reviewed_by.clone()),
                Some(DbTs::now()),
            )
        } else {
            (existing.reviewed_by.clone(), existing.reviewed_at.clone())
        };
        let submitted_at = if status != "none" {
            existing.submitted_at.clone().or_else(|| Some(DbTs::now()))
        } else {
            existing.submitted_at.clone()
        };
        (
            status,
            reject_reason,
            admin_remark,
            reviewed_by,
            reviewed_at,
            submitted_at,
        )
    } else {
        // 用户提交：强制 pending，清空驳回原因
        validate_payload(
            &kyc_type,
            id_doc_type.as_deref(),
            real_name.as_deref(),
            company_name.as_deref(),
            id_doc_front_url.as_deref(),
            id_doc_back_url.as_deref(),
            business_license_url.as_deref(),
            tax_registration_url.as_deref(),
            legal_notarization_url.as_deref(),
            true,
        )?;
        (
            "pending".to_string(),
            None,
            existing.admin_remark.clone(),
            None,
            None,
            Some(DbTs::now()),
        )
    };

    // 管理员保存时也做轻量校验：若状态为 approved/pending 则要求材料齐全
    if is_admin && matches!(status.as_str(), "approved" | "pending") {
        validate_payload(
            &kyc_type,
            id_doc_type.as_deref(),
            real_name.as_deref(),
            company_name.as_deref(),
            id_doc_front_url.as_deref(),
            id_doc_back_url.as_deref(),
            business_license_url.as_deref(),
            tax_registration_url.as_deref(),
            legal_notarization_url.as_deref(),
            true,
        )?;
    }

    sqlx::query(
        &state.db.format_query(
            r#"INSERT INTO user_kyc (
                user_id, kyc_type, status, real_name, id_doc_type, id_doc_front_url, id_doc_back_url,
                company_name, business_license_url, tax_registration_url, legal_notarization_url,
                validity_type, expire_at, reject_reason, admin_remark, reviewed_by, reviewed_at,
                submitted_at, created_at, updated_at
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?, ?,
                ?, ?, ?, ?, ?, ?,
                ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            )
            ON CONFLICT (user_id) DO UPDATE SET
                kyc_type = EXCLUDED.kyc_type,
                status = EXCLUDED.status,
                real_name = EXCLUDED.real_name,
                id_doc_type = EXCLUDED.id_doc_type,
                id_doc_front_url = EXCLUDED.id_doc_front_url,
                id_doc_back_url = EXCLUDED.id_doc_back_url,
                company_name = EXCLUDED.company_name,
                business_license_url = EXCLUDED.business_license_url,
                tax_registration_url = EXCLUDED.tax_registration_url,
                legal_notarization_url = EXCLUDED.legal_notarization_url,
                validity_type = EXCLUDED.validity_type,
                expire_at = EXCLUDED.expire_at,
                reject_reason = EXCLUDED.reject_reason,
                admin_remark = EXCLUDED.admin_remark,
                reviewed_by = EXCLUDED.reviewed_by,
                reviewed_at = EXCLUDED.reviewed_at,
                submitted_at = EXCLUDED.submitted_at,
                updated_at = CURRENT_TIMESTAMP
            "#,
        ),
    )
    .bind(user_id)
    .bind(&kyc_type)
    .bind(&status)
    .bind(&real_name)
    .bind(&id_doc_type)
    .bind(&id_doc_front_url)
    .bind(&id_doc_back_url)
    .bind(&company_name)
    .bind(&business_license_url)
    .bind(&tax_registration_url)
    .bind(&legal_notarization_url)
    .bind(&validity_type)
    .bind(&expire_at)
    .bind(&reject_reason)
    .bind(&admin_remark)
    .bind(&reviewed_by)
    .bind(&reviewed_at)
    .bind(&submitted_at)
    .execute(&state.db.pool)
    .await?;

    fetch_kyc(state, user_id).await
}

/// 用户获取自己的实名信息
pub async fn get_my_kyc(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<auth::Claims>,
) -> AppResult<Json<UserKyc>> {
    ensure_kyc_enabled(&state).await?;
    Ok(Json(fetch_kyc(&state, &claims.sub).await?))
}

/// 用户提交实名认证
pub async fn submit_my_kyc(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<auth::Claims>,
    Json(req): Json<UpsertUserKycRequest>,
) -> AppResult<Json<UserKyc>> {
    ensure_kyc_enabled(&state).await?;
    let existing = fetch_kyc(&state, &claims.sub).await?;
    if existing.status == "approved" {
        return Err(AppError::BadRequest(
            "实名认证已通过，如需变更请联系管理员".to_string(),
        ));
    }
    Ok(Json(
        upsert_kyc(&state, &claims.sub, req, false, None).await?,
    ))
}

/// 管理员获取指定用户实名信息（不受站点开关限制，便于后台录入）
pub async fn admin_get_user_kyc(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> AppResult<Json<UserKyc>> {
    let exists: bool = sqlx::query_scalar(
        &state
            .db
            .format_query("SELECT EXISTS(SELECT 1 FROM users WHERE id = ?)"),
    )
    .bind(&user_id)
    .fetch_one(&state.db.pool)
    .await?;
    if !exists {
        return Err(AppError::NotFound("用户不存在".to_string()));
    }
    Ok(Json(fetch_kyc(&state, &user_id).await?))
}

/// 管理员保存/审核用户实名
pub async fn admin_upsert_user_kyc(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<auth::Claims>,
    Path(user_id): Path<String>,
    Json(req): Json<UpsertUserKycRequest>,
) -> AppResult<Json<UserKyc>> {
    let exists: bool = sqlx::query_scalar(
        &state
            .db
            .format_query("SELECT EXISTS(SELECT 1 FROM users WHERE id = ?)"),
    )
    .bind(&user_id)
    .fetch_one(&state.db.pool)
    .await?;
    if !exists {
        return Err(AppError::NotFound("用户不存在".to_string()));
    }
    Ok(Json(
        upsert_kyc(&state, &user_id, req, true, Some(&claims.username)).await?,
    ))
}

/// 上传 KYC 证件图片（用户与管理员共用；管理员可传 target_user_id）
pub async fn upload_kyc_document(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<auth::Claims>,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    let is_admin = claims.role == "admin";
    if !is_admin {
        ensure_kyc_enabled(&state).await?;
    }

    let tos_config = crate::relay::tos_persist::load_system_tos_config(&state)
        .await
        .ok_or_else(|| {
            AppError::BadRequest(
                "证件上传需要先配置对象存储，请管理员在「站点设置 → 数据库/存储」中完成 TOS 配置"
                    .to_string(),
            )
        })?;

    let mut file_data: Option<axum::body::Bytes> = None;
    let mut original_name = String::from("document");
    let mut mime_type = String::from("application/octet-stream");
    let mut target_user_id = claims.sub.clone();
    let mut doc_field = String::from("doc");

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            original_name = field
                .file_name()
                .unwrap_or("document")
                .chars()
                .take(120)
                .collect();
            mime_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            if !(mime_type.starts_with("image/") || mime_type == "application/pdf") {
                return Err(AppError::BadRequest(
                    "仅支持上传图片或 PDF 证件文件".to_string(),
                ));
            }
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|_| AppError::BadRequest("读取文件失败".to_string()))?,
            );
        } else if name == "target_user_id" {
            let tid = field.text().await.unwrap_or_default();
            if is_admin && !tid.trim().is_empty() {
                target_user_id = tid.trim().to_string();
            }
        } else if name == "doc_field" {
            doc_field = field.text().await.unwrap_or_else(|_| "doc".to_string());
        }
    }

    let data = file_data.ok_or_else(|| AppError::BadRequest("请选择要上传的文件".to_string()))?;
    if data.len() > 12 * 1024 * 1024 {
        return Err(AppError::BadRequest("证件文件不能超过 12MB".to_string()));
    }

    let ext = original_name
        .rsplit('.')
        .next()
        .filter(|e| e.len() <= 8)
        .unwrap_or(if mime_type == "application/pdf" {
            "pdf"
        } else {
            "jpg"
        });
    let safe_field = doc_field
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(40)
        .collect::<String>();
    let object_key = format!(
        "kyc/{}/{}_{}.{}",
        target_user_id,
        if safe_field.is_empty() {
            "doc"
        } else {
            &safe_field
        },
        chrono::Utc::now().timestamp_millis(),
        ext
    );

    let file_url = crate::services::tos::upload_file(
        &tos_config,
        &object_key,
        data.to_vec(),
        &mime_type,
        None,
    )
    .await
    .map_err(|e| {
        tracing::error!("KYC TOS upload failed: {}", e);
        AppError::Internal("证件上传失败，请稍后重试".to_string())
    })?;

    Ok(Json(serde_json::json!({
        "file_url": file_url,
        "object_key": object_key,
    })))
}
