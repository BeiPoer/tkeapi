/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

/** 官方服务商 / API 服务商 / 类型 / 模型来源 → 列表与 stats 共用的查询参数 */
export type ModelSourceFilter = 'all' | 'system' | 'custom';

export function buildClassificationParams(
  providerId: number | null | undefined,
  apiProviderId: number | null | undefined,
  typeId: number | null | undefined,
  source?: ModelSourceFilter | null,
): Record<string, string | number> {
  const params: Record<string, string | number> = {};
  if (providerId != null) params.provider_id = providerId;
  if (apiProviderId != null) params.api_provider_id = apiProviderId;
  if (typeId != null) params.type_id = typeId;
  if (source === 'system' || source === 'custom') params.source = source;
  return params;
}
