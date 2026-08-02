/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

/** 配合 skipErrorHandler，只弹一次业务错误 */
export function apiErrMsg(e: unknown, fallback: string): string {
  const err = e as {
    response?: { data?: { error?: { message?: string }; message?: string } };
  };
  return err?.response?.data?.error?.message || err?.response?.data?.message || fallback;
}

export const SKIP_ERR = { skipErrorHandler: true } as const;
