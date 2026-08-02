/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

/** 管理后台「可见基础菜单」权限树：一级分类 + 可选二级菜单 */

export type AdminMenuPermChild = {
  label: string;
  value: string;
};

export type AdminMenuPermNode = {
  label: string;
  value: string;
  children?: AdminMenuPermChild[];
};

export const ADMIN_MENU_PERMISSIONS: AdminMenuPermNode[] = [
  { label: '仪表盘 (Dashboard)', value: 'dashboard' },
  { label: '中转网关 (Relay API)', value: 'relay_api' },
  { label: '令牌管理 (Tokens)', value: 'tokens' },
  {
    label: '日志管理 (Logs)',
    value: 'logs',
    children: [
      { label: '使用日志', value: 'logs.usage' },
      { label: '任务列表', value: 'logs.tasks' },
    ],
  },
  {
    label: '渠道管理 (Channels)',
    value: 'channels',
    children: [
      { label: '模型渠道分组', value: 'channels.groups' },
      { label: '上游渠道配置', value: 'channels.configs' },
    ],
  },
  {
    label: '模型管理 (Models)',
    value: 'models',
    children: [
      { label: '模型列表', value: 'models.list' },
      { label: '计费规则', value: 'models.billing_rules' },
      { label: '转发规则', value: 'models.forward_rules' },
    ],
  },
  {
    label: '营销管理 (Marketing)',
    value: 'marketing',
    children: [
      { label: '兑换码', value: 'marketing.redemptions' },
      { label: '注册赠送', value: 'marketing.registration_gifts' },
      { label: '提示通知', value: 'marketing.announcements' },
    ],
  },
  {
    label: '用户管理 (Users)',
    value: 'users',
    children: [
      { label: '用户列表', value: 'users.list' },
      { label: '管理员列表', value: 'users.admins' },
      { label: '用户等级', value: 'users.levels' },
      { label: '权限分组管理', value: 'admin_groups' },
    ],
  },
  {
    label: '财务管理 (Finance)',
    value: 'finance',
    children: [
      { label: '充值记录', value: 'finance.recharges' },
      { label: '赠送记录', value: 'finance.gifts' },
      { label: '订单记录', value: 'finance.orders' },
      { label: '财务数据分析', value: 'finance.analysis' },
    ],
  },
  {
    label: '系统设置 (Settings)',
    value: 'settings',
    children: [
      { label: '基础设置', value: 'settings.basic' },
      { label: '支付设置', value: 'settings.payment' },
      { label: '消息通知', value: 'settings.message_notification' },
      { label: 'OAuth 设置', value: 'settings.oauth' },
      { label: '数据库设置', value: 'settings.database' },
    ],
  },
];

/** 扁平化所有可勾选权限值（含一级与二级） */
export function flattenAdminMenuPermissions(
  nodes: AdminMenuPermNode[] = ADMIN_MENU_PERMISSIONS,
): string[] {
  return nodes.flatMap((n) => [n.value, ...(n.children?.map((c) => c.value) || [])]);
}

/** 旧数据仅有一级权限时，编辑回显时展开为「一级 + 全部二级」 */
export function expandLegacyAdminMenuPermissions(perms: string[]): string[] {
  const result = new Set(perms);
  for (const group of ADMIN_MENU_PERMISSIONS) {
    if (!group.children?.length) continue;
    const hasAnyChild = group.children.some((c) => result.has(c.value));
    if (result.has(group.value) && !hasAnyChild) {
      group.children.forEach((c) => {
        // admin_groups 历史需单独勾选，回显时不要因一级 users 自动勾上
        if (c.value === 'admin_groups') return;
        result.add(c.value);
      });
    }
  }
  return Array.from(result);
}

/**
 * 保存前规范化：若勾选了任一二级，确保写入对应一级；
 * 若一级下全部二级都勾选，保留一级与全部二级。
 */
export function normalizeAdminMenuPermissions(perms: string[]): string[] {
  const result = new Set(perms.filter((p) => !p.startsWith('plugin:')));
  for (const group of ADMIN_MENU_PERMISSIONS) {
    if (!group.children?.length) continue;
    const selectedChildren = group.children.filter((c) => result.has(c.value));
    if (selectedChildren.length > 0) {
      result.add(group.value);
    } else if (result.has(group.value)) {
      // 仅勾一级、未勾二级：视为全部二级（兼容旧逻辑；admin_groups 除外）
      group.children.forEach((c) => {
        if (c.value === 'admin_groups') return;
        result.add(c.value);
      });
    }
  }
  return Array.from(result);
}

/** 权限标签展示用短名 */
export function getAdminMenuPermissionLabel(value: string): string {
  for (const group of ADMIN_MENU_PERMISSIONS) {
    if (group.value === value) return group.label.split(' (')[0];
    const child = group.children?.find((c) => c.value === value);
    if (child) return `${group.label.split(' (')[0]} / ${child.label}`;
  }
  return value;
}

/**
 * 子菜单可见性：
 * - 超级管理员：放行
 * - 若 permissions 中存在该一级下任一二级 key：按二级精确匹配
 * - 否则回退为仅检查一级（兼容旧数据）
 */
export function hasAdminChildMenuPermission(
  permissions: string[] | undefined | null,
  parentKey: string,
  childKey: string,
  isSuperAdmin = false,
): boolean {
  if (isSuperAdmin) return true;
  if (!permissions) return false;
  const group = ADMIN_MENU_PERMISSIONS.find((g) => g.value === parentKey);
  const childKeys = group?.children?.map((c) => c.value) || [];
  const hasGranular = childKeys.some((k) => permissions.includes(k));
  if (hasGranular) {
    return permissions.includes(childKey);
  }
  // 旧数据仅有一级权限：放开该分类下二级；admin_groups 历史上一向需单独授权
  if (childKey === 'admin_groups') {
    return permissions.includes('admin_groups');
  }
  return permissions.includes(parentKey);
}
