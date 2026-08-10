/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

//! 渠道 / 上游预设额度累加与退款（事务内调用）

use crate::db::Database;
use crate::models::channel_quota::{consume_quota_sql, refund_quota_sql};
use sqlx::{Postgres, Transaction};

fn period_keys(tz_name: &str) -> (String, String, String) {
    let (day, week, month) = crate::models::quota_period_keys(tz_name);
    (day, week, month)
}

async fn consume_with_keys(
    db: &Database,
    tx: &mut Transaction<'_, Postgres>,
    table: &str,
    id: i64,
    amount: f64,
    now_day: &str,
    now_week: &str,
    now_month: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(&db.format_query(&consume_quota_sql(table)))
        .bind(amount)
        .bind(now_day)
        .bind(amount)
        .bind(amount)
        .bind(now_week)
        .bind(amount)
        .bind(amount)
        .bind(now_month)
        .bind(amount)
        .bind(amount)
        .bind(now_day)
        .bind(now_week)
        .bind(now_month)
        .bind(id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn refund_with_keys(
    db: &Database,
    tx: &mut Transaction<'_, Postgres>,
    table: &str,
    id: i64,
    amount: f64,
    now_day: &str,
    now_week: &str,
    now_month: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(&db.format_query(&refund_quota_sql(table)))
        .bind(amount)
        .bind(now_day)
        .bind(amount)
        .bind(now_week)
        .bind(amount)
        .bind(now_month)
        .bind(amount)
        .bind(id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// 消费：累加总/日/周/月已用量。
/// **共享资源**：`tz_name` 必须传入站点默认时区，禁止使用请求用户 timedisplay。
pub async fn consume(
    db: &Database,
    tx: &mut Transaction<'_, Postgres>,
    table: &str,
    id: i64,
    amount: f64,
    tz_name: &str,
) -> Result<(), sqlx::Error> {
    if amount <= 0.0 || id <= 0 {
        return Ok(());
    }
    let (now_day, now_week, now_month) = period_keys(tz_name);
    consume_with_keys(db, tx, table, id, amount, &now_day, &now_week, &now_month).await
}

/// 退款：扣减总/日/周/月已用量
pub async fn refund(
    db: &Database,
    tx: &mut Transaction<'_, Postgres>,
    table: &str,
    id: i64,
    amount: f64,
    tz_name: &str,
) -> Result<(), sqlx::Error> {
    if amount <= 0.0 || id <= 0 {
        return Ok(());
    }
    let (now_day, now_week, now_month) = period_keys(tz_name);
    refund_with_keys(db, tx, table, id, amount, &now_day, &now_week, &now_month).await
}

pub async fn consume_channel(
    db: &Database,
    tx: &mut Transaction<'_, Postgres>,
    channel_id: i64,
    amount: f64,
    tz_name: &str,
) -> Result<(), sqlx::Error> {
    consume(db, tx, "channels", channel_id, amount, tz_name).await
}

pub async fn refund_channel(
    db: &Database,
    tx: &mut Transaction<'_, Postgres>,
    channel_id: i64,
    amount: f64,
    tz_name: &str,
) -> Result<(), sqlx::Error> {
    refund(db, tx, "channels", channel_id, amount, tz_name).await
}

#[derive(sqlx::FromRow)]
struct ConfigDailyReset {
    daily_reset_hour: i32,
    daily_reset_minute: i32,
    daily_reset_cooldown_minutes: i32,
}

async fn config_period_keys(
    db: &Database,
    tx: &mut Transaction<'_, Postgres>,
    config_id: i64,
    tz_name: &str,
) -> Result<(String, String, String), sqlx::Error> {
    let (_, now_week, now_month) = period_keys(tz_name);
    let row: Option<ConfigDailyReset> = sqlx::query_as(&db.format_query(
        "SELECT daily_reset_hour, daily_reset_minute, daily_reset_cooldown_minutes FROM channel_configs WHERE id = ?",
    ))
    .bind(config_id)
    .fetch_optional(&mut **tx)
    .await?;
    let (h, m, c) = row
        .map(|r| {
            (
                r.daily_reset_hour,
                r.daily_reset_minute,
                r.daily_reset_cooldown_minutes,
            )
        })
        .unwrap_or((0, 0, 0));
    let now_day = crate::time_system::quota_day_key_with_cutover(tz_name, h, m, c);
    Ok((now_day, now_week, now_month))
}

pub async fn consume_config(
    db: &Database,
    tx: &mut Transaction<'_, Postgres>,
    config_id: i64,
    amount: f64,
    tz_name: &str,
) -> Result<(), sqlx::Error> {
    if amount <= 0.0 || config_id <= 0 {
        return Ok(());
    }
    let (now_day, now_week, now_month) = config_period_keys(db, tx, config_id, tz_name).await?;
    consume_with_keys(
        db,
        tx,
        "channel_configs",
        config_id,
        amount,
        &now_day,
        &now_week,
        &now_month,
    )
    .await
}

pub async fn refund_config(
    db: &Database,
    tx: &mut Transaction<'_, Postgres>,
    config_id: i64,
    amount: f64,
    tz_name: &str,
) -> Result<(), sqlx::Error> {
    if amount <= 0.0 || config_id <= 0 {
        return Ok(());
    }
    let (now_day, now_week, now_month) = config_period_keys(db, tx, config_id, tz_name).await?;
    refund_with_keys(
        db,
        tx,
        "channel_configs",
        config_id,
        amount,
        &now_day,
        &now_week,
        &now_month,
    )
    .await
}
