() => {
  // 只观察当前文档，不修改样式、滚动、动画、事件或 Reader 状态。
  const frame = document.querySelector('iframe.epub-frame');
  if (!frame || frame.getAttribute('aria-busy') !== 'false') return false;
  try {
    const current = frame.contentDocument;
    if (!current?.body || !current.documentElement || current.readyState !== 'complete'
      || current.fonts?.status !== 'loaded' || typeof frame.srcdoc !== 'string' || !frame.srcdoc) return false;
    // srcdoc 属性已更新时，iframe 的旧文档仍可能有足量文字。
    // 比较规范化 DOM，不能仅凭文本长度、URL 或属性已赋值认定新章提交。
    const expected = new DOMParser().parseFromString(frame.srcdoc, 'text/html');
    return !!expected.documentElement
      && current.documentElement.outerHTML === expected.documentElement.outerHTML;
  } catch { return false; }
}
