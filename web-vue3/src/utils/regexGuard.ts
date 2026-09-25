/**
 * P1-5 前端侧：正则测试防护（ReDoS）。
 *
 * 浏览器主线程同步执行 `new RegExp(...).test/match` 无法超时中断——灾难性回溯
 * （catastrophic backtracking）可能卡死页面。策略：
 * - 测试正则长度上限 200 字符（超限拒绝并提示）；
 * - 说明：长度限制不能根除短模式 + 长文本的灾难性回溯，但能挡住最典型的
 *   超长恶意模式（`(a+)+$` 这类嵌套量词短模式仍可能卡顿，属已知残余风险）。
 */
export const MAX_TEST_REGEX_LEN = 200

/**
 * 校验测试用正则：合法返回 null；超限返回错误提示文案。
 */
export function checkTestRegex(pattern: string): string | null {
  if (pattern.length > MAX_TEST_REGEX_LEN) {
    return `正则超过 ${MAX_TEST_REGEX_LEN} 字符上限（防止灾难性回溯卡死页面）`
  }
  return null
}
