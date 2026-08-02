/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

/** 仅在用户手动切换语言时写入；未设置则跟随站点 default_language */
const USER_LANGUAGE_KEY = 'tokensbyte_user_language';

/** 读取用户手动选择的语言；无记录返回 null */
export function getUserLanguagePreference(): string | null {
  const preferred = localStorage.getItem(USER_LANGUAGE_KEY);
  return preferred && preferred.trim() ? preferred : null;
}

/** 持久化用户手动语言偏好，并清理旧 detector 缓存键 */
export function persistUserLanguagePreference(lng: string): void {
  localStorage.setItem(USER_LANGUAGE_KEY, lng);
  // 旧版 LanguageDetector 会把浏览器语言写入此键，干扰「站点默认语言」逻辑
  localStorage.removeItem('i18nextLng');
}
