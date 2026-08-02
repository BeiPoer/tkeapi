/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

//! 站点内部账本金额精度约定：一律保留小数点后 6 位（四舍五入）。
//!
//! 适用范围：余额、赠送金、日志 cost、预扣/结算扣费、充值调账、额度用量等。
//! 不适用：支付通道对外金额（微信/支付宝等仍按对方要求保留分，即 2 位）。

/// 金额小数位数
pub const MONEY_DECIMAL_PLACES: u32 = 6;

/// 缩放因子：10^6（与 MONEY_DECIMAL_PLACES 保持一致）
pub const MONEY_SCALE: f64 = 1_000_000.0;

/// 四舍五入到约定小数位
#[inline]
pub fn round_money(v: f64) -> f64 {
    if !v.is_finite() {
        return 0.0;
    }
    (v * MONEY_SCALE).round() / MONEY_SCALE
}

/// 格式化为固定小数位字符串（日志/通知用）
#[inline]
pub fn format_money(v: f64) -> String {
    format!("{:.*}", MONEY_DECIMAL_PLACES as usize, round_money(v))
}

/// 预扣/扣费：赠送余额优先，返回 `(gift_deducted, balance_deducted)`（均已 round）。
#[inline]
pub fn split_gift_first(amount: f64, gift_balance: f64) -> (f64, f64) {
    let amount = round_money(amount);
    if amount <= 0.0 {
        return (0.0, 0.0);
    }
    let gift = round_money(amount.min(gift_balance.max(0.0)));
    (gift, round_money(amount - gift))
}

/// 结算差额 `(settled_cost, apply_balance)`：应付原样；apply 正补扣、负退多扣。
#[inline]
pub fn settlement_delta(cost: f64, pre_deducted: f64) -> (f64, f64) {
    let settled = round_money(cost);
    let apply = round_money(settled - round_money(pre_deducted));
    (settled, apply)
}
