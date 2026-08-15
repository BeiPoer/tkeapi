/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

import i18n from '../i18n';

/** 历史脏值 → 规范 type */
const ALIAS: Record<string, string> = {
  'allinpay wechat': 'allinpay_wechat',
  'allinpay alipay': 'allinpay_alipay',
};

const BRAND: Record<string, string> = {
  stripe: 'Stripe',
  hyperbc: 'HyperBC',
  bonuspay: 'BonusPay',
};

const COLOR: Record<string, string> = {
  manual: 'blue',
  gift: 'cyan',
  registration: 'magenta',
  commission: 'gold',
  alipay: 'blue',
  wechat: 'green',
  allinpay_wechat: 'green',
  allinpay_alipay: 'blue',
  stripe: 'purple',
  bonuspay: 'geekblue',
  hyperbc: 'purple',
  redemption: 'orange',
  ark_video_consume: 'volcano',
  ark_video_refund: 'green',
};

const WALLET_RECHARGE_FILTERS = [
  'manual',
  'gift',
  'registration',
  'commission',
  'alipay',
  'wechat',
  'redemption',
  'ark_video_consume',
  'ark_video_refund',
] as const;

function norm(type: string): string {
  return ALIAS[type] || type;
}

/** 充值类型文案（固定读 translation/finance.*，不依赖调用方 useTranslation 的 ns） */
export function rechargeTypeLabel(type: string, walletType?: string | null): string {
  if (!type) return '-';
  if (type === 'manual' && walletType === 'credit') {
    return i18n.t('finance.recharge_type_manual_credit');
  }
  const key = norm(type);
  if (BRAND[key]) return BRAND[key];
  const i18nKey = `finance.recharge_type_${key}`;
  const label = i18n.t(i18nKey);
  return !label || label === i18nKey ? type : label;
}

export function rechargeTypeColor(type: string): string {
  return COLOR[norm(type)] || 'default';
}

export function rechargeTypeFilters() {
  return WALLET_RECHARGE_FILTERS.map((value) => ({
    text: rechargeTypeLabel(value),
    value,
  }));
}
