/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)\r\n */

/**
 * 合并进行中的相同异步任务（如 React StrictMode 双挂载、并发调用）。
 * 可选短时缓存：避免 settings 等依赖刚到又立刻重打同一接口。
 */
const inflight = new Map<string, Promise<unknown>>();
const recent = new Map<string, { at: number; value: unknown }>();

const DEFAULT_RECENT_MS = 1200;

export function coalesceAsync<T>(
  key: string,
  factory: () => Promise<T>,
  options?: { recentMs?: number },
): Promise<T> {
  const recentMs = options?.recentMs ?? DEFAULT_RECENT_MS;
  if (recentMs > 0) {
    const cached = recent.get(key);
    if (cached && Date.now() - cached.at < recentMs) {
      return Promise.resolve(cached.value as T);
    }
  }

  const existing = inflight.get(key);
  if (existing) return existing as Promise<T>;

  const promise = factory()
    .then((value) => {
      if (recentMs > 0) {
        recent.set(key, { at: Date.now(), value });
      }
      return value;
    })
    .finally(() => {
      if (inflight.get(key) === promise) {
        inflight.delete(key);
      }
    });
  inflight.set(key, promise);
  return promise;
}

/**
 * 主动使指定 key 的短时缓存失效。
 * 应在写操作（删除/新建/上传）成功后调用，确保下次读取能获得最新数据而非旧缓存。
 * @example invalidateAsync('pg2026:storage-stats');
 */
export function invalidateAsync(...keys: string[]): void {
  for (const key of keys) {
    recent.delete(key);
    inflight.delete(key);
  }
}

/** 清除指定 key 的短时缓存与进行中 Promise（后台改配置后强制刷新） */
export function invalidateCoalesce(key: string): void {
  recent.delete(key);
  inflight.delete(key);
}
