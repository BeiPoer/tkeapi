/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

/** 从详情 plugin_tag 解析展开展示字段（列表不返回该列） */
export function parsePluginTagMeta(pluginTag?: string | null): {
  clientCt?: string;
  cascadeS1TaskId?: string;
} {
  if (!pluginTag) return {};
  try {
    const tag = JSON.parse(pluginTag);
    const s1 = tag?.cascade?.s1_task_id;
    return {
      clientCt: typeof tag?.client_ct === 'string' && tag.client_ct ? tag.client_ct : undefined,
      cascadeS1TaskId: typeof s1 === 'string' && s1 ? s1 : undefined,
    };
  } catch {
    return {};
  }
}
