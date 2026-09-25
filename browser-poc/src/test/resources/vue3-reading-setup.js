async (base) => {
  const post = async (path, body) => {
    const response = await fetch('/reader3/' + path, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
      credentials: 'same-origin',
    });
    const result = await response.json();
    if (response.status !== 200 || !result.isSuccess) {
      throw new Error(path + ': ' + (result.errorMsg || response.status));
    }
    return result.data;
  };
  const source = {
    bookSourceUrl: base,
    bookSourceName: 'Vue 3 reading fixture',
    searchUrl: base + '/search?key={{key}}',
    ruleSearch: {
      bookList: '.book', name: '.name@text', author: '.author@text', bookUrl: 'a@href',
    },
    ruleBookInfo: { name: 'h1@text', author: '.author@text', tocUrl: '.toc@href' },
    ruleToc: { chapterList: '.chapter', chapterName: 'a@text', chapterUrl: 'a@href' },
    ruleContent: { content: '.content@html' },
  };
  await post('saveBookSource', source);
  const bookUrl = base + '/book';
  const info = await post('getBookInfo', { url: bookUrl, bookSource: source });
  await post('saveBook', info);
  return bookUrl;
}
