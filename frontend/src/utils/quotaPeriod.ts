/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia 
 * @license        MIT (https://www.tokensbyte.ai/)
 */

import dayjs from 'dayjs';
import utc from 'dayjs/plugin/utc';
import timezone from 'dayjs/plugin/timezone';
import customParseFormat from 'dayjs/plugin/customParseFormat';
import useSettingsStore from '../store/settings';

dayjs.extend(utc);
dayjs.extend(timezone);
dayjs.extend(customParseFormat);

/** 渠道共享额度周期：与后端一致，使用站点默认时区 */
function siteTimezone(): string {
  return (
    useSettingsStore.getState().settings?.site?.default_timezone?.trim() ||
    'Asia/Shanghai'
  );
}

/** InputNumber 展示：-1 → 无限额 */
export function formatQuotaLimitDisplay(val: number | string | null | undefined) {
  return val === -1 || val === '-1' ? '无限额' : `${val ?? ''}`;
}

/** InputNumber 解析：清空 / 无限文案 → -1，避免误存为 0 */
export function parseQuotaLimitInput(val: string | undefined): number {
  if (val == null || val === '' || val === '无限额' || val === '不限制') return -1;
  const n = parseFloat(val);
  return Number.isFinite(n) ? n : -1;
}

/** 有限额度（≥0）；-1 / null / undefined 视为不限制，不参与大小比较 */
export function isFiniteQuotaLimit(n: unknown): n is number {
  return typeof n === 'number' && Number.isFinite(n) && n >= 0;
}

/**
 * 校验额度层级：日 ≤ 周 ≤ 月 ≤ 总（仅双方均为有限额度时比较）。
 * @returns 错误文案；合法则返回 null
 */
export function validateQuotaHierarchy(values: {
  quota_limit?: number | null;
  daily_quota_limit?: number | null;
  weekly_quota_limit?: number | null;
  monthly_quota_limit?: number | null;
}): string | null {
  const total = values.quota_limit;
  const day = values.daily_quota_limit;
  const week = values.weekly_quota_limit;
  const month = values.monthly_quota_limit;

  for (const [label, v] of [
    ['总额度', total],
    ['日额度', day],
    ['周额度', week],
    ['月额度', month],
  ] as const) {
    if (v != null && Number.isFinite(Number(v)) && Number(v) < -1) {
      return `${label}不能小于 -1`;
    }
  }

  const check = (
    smaller: number | null | undefined,
    larger: number | null | undefined,
    msg: string,
  ) => {
    if (isFiniteQuotaLimit(smaller) && isFiniteQuotaLimit(larger) && smaller > larger) {
      return msg;
    }
    return null;
  };

  return (
    check(day, week, '日额度不能大于周额度') ||
    check(day, month, '日额度不能大于月额度') ||
    check(day, total, '日额度不能大于总额度') ||
    check(week, month, '周额度不能大于月额度') ||
    check(week, total, '周额度不能大于总额度') ||
    check(month, total, '月额度不能大于总额度') ||
    null
  );
}

/** 根据其它字段计算某项额度的动态上限（用于 InputNumber max） */
function getQuotaInputMax(

  field: 'quota_limit' | 'daily_quota_limit' | 'weekly_quota_limit' | 'monthly_quota_limit',
  values: {
    quota_limit?: number | null;
    daily_quota_limit?: number | null;
    weekly_quota_limit?: number | null;
    monthly_quota_limit?: number | null;
  },
): number | undefined {
  const caps: number[] = [];
  if (field === 'daily_quota_limit') {
    if (isFiniteQuotaLimit(values.weekly_quota_limit)) caps.push(values.weekly_quota_limit);
    if (isFiniteQuotaLimit(values.monthly_quota_limit)) caps.push(values.monthly_quota_limit);
    if (isFiniteQuotaLimit(values.quota_limit)) caps.push(values.quota_limit);
  } else if (field === 'weekly_quota_limit') {
    if (isFiniteQuotaLimit(values.monthly_quota_limit)) caps.push(values.monthly_quota_limit);
    if (isFiniteQuotaLimit(values.quota_limit)) caps.push(values.quota_limit);
  } else if (field === 'monthly_quota_limit') {
    if (isFiniteQuotaLimit(values.quota_limit)) caps.push(values.quota_limit);
  }
  if (caps.length === 0) return undefined;
  return Math.min(...caps);
}

/** 日·周·月周期键（与后端 `%Y-%U` 周键一致；周日为一周起点） */
export function getLocalPeriodKeys(tz: string) {
  const now = dayjs().tz(tz);
  const nowDay = now.format('YYYY-MM-DD');
  const nowMonth = now.format('YYYY-MM');
  const year = now.year();
  const janFirst = dayjs.tz(`${year}-01-01`, tz);
  const days = now.startOf('day').diff(janFirst.startOf('day'), 'day');
  const weekNum = Math.floor((days + janFirst.day()) / 7);
  const nowWeek = `${year}-${String(weekNum).padStart(2, '0')}`;
  return { nowDay, nowWeek, nowMonth };
}

export function getEffectiveChannelPeriodUsed(
  record: {
    last_reset_day?: string | null;
    last_reset_week?: string | null;
    last_reset_month?: string | null;
    daily_quota_used?: number | null;
    weekly_quota_used?: number | null;
    monthly_quota_used?: number | null;
  },
  tz?: string,
) {
  const { nowDay, nowWeek, nowMonth } = getLocalPeriodKeys(tz || siteTimezone());
  return {
    dailyUsed: record.last_reset_day === nowDay ? (record.daily_quota_used || 0) : 0,
    weeklyUsed: record.last_reset_week === nowWeek ? (record.weekly_quota_used || 0) : 0,
    monthlyUsed: record.last_reset_month === nowMonth ? (record.monthly_quota_used || 0) : 0,
  };
}

/**
 * 兑换码是否过期：与后端一致。
 * - 纯日期：站点时区该日结束（次日 00:00）前仍有效
 * - 无时区日期时间：按站点时区墙钟解释
 */
export function isRedemptionExpired(
  expiresAt: string | null | undefined,
  tz?: string,
): boolean {
  if (!expiresAt || !String(expiresAt).trim()) return false;
  const zone = tz || siteTimezone();
  const exp = String(expiresAt).trim();
  const now = dayjs().tz(zone);

  if (/^\d{4}-\d{2}-\d{2}$/.test(exp)) {
    const end = dayjs.tz(exp, 'YYYY-MM-DD', zone).add(1, 'day').startOf('day');
    return !now.isBefore(end);
  }

  if (exp.includes('T') && (exp.includes('+') || exp.endsWith('Z') || /[+-]\d{2}:\d{2}$/.test(exp))) {
    return now.isAfter(dayjs(exp));
  }

  const dt = dayjs.tz(exp, 'YYYY-MM-DD HH:mm:ss', zone);
  if (dt.isValid()) return now.isAfter(dt);

  const dt2 = dayjs.tz(exp, 'YYYY-MM-DDTHH:mm:ss', zone);
  if (dt2.isValid()) return now.isAfter(dt2);

  return false;
}
