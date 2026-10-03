async (bookUrl) => {
  const shelfReply = await fetch('/reader3/getBookshelf', { credentials: 'same-origin' });
  const shelf = await shelfReply.json();
  if (shelfReply.status !== 200 || !shelf.isSuccess) throw new Error('Fixture bookshelf read failed');
  const template = shelf.data.find(book => book.bookUrl === bookUrl);
  if (!template) throw new Error('Missing deterministic fixture book');
  for (let index = 0; index < 15; index++) {
    const response = await fetch('/reader3/saveBook', {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        ...template,
        bookUrl: index === 0 ? bookUrl : bookUrl + '?layout=' + index,
        name: 'Layout fixture ' + String(index).padStart(2, '0'),
        author: 'Loopback fixture',
        intro: '长简介用于布局回归，不含用户数据。'.repeat(12),
        canUpdate: false,
      }),
    });
    const reply = await response.json();
    if (response.status !== 200 || !reply.isSuccess) throw new Error('Fixture saveBook failed');
  }
}
