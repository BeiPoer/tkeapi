/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

/** 官方服务商 / API 服务商 / 类型 → 列表与 stats 共用的查询参数 */
export function buildClassificationParams(
  providerId: number | null | undefined,
  apiProviderId: number | null | undefined,
  typeId: number | null | undefined,
): Record<string, number> {
  const params: Record<string, number> = {};
  if (providerId != null) params.provider_id = providerId;
  if (apiProviderId != null) params.api_provider_id = apiProviderId;
  if (typeId != null) params.type_id = typeId;
  return params;
}
