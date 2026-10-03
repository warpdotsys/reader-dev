async (bookUrl) => {
  const shelfReply = await fetch('/reader3/getBookshelf', { credentials: 'same-origin' });
  const shelf = await shelfReply.json();
  if (shelfReply.status !== 200 || !shelf.isSuccess) throw new Error('Fixture bookshelf read failed');
  const template = shelf.data.find(book => book.bookUrl === bookUrl);
  if (!template) throw new Error('Missing deterministic fixture book');
  const intro = '长简介用于布局回归，不含用户数据。'.repeat(12);
  for (let index = 0; index < 15; index++) {
    const response = await fetch('/reader3/saveBook', {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        ...template,
        bookUrl: index === 0 ? bookUrl : bookUrl + '?layout=' + index,
        // saveBookToShelf identifies existing entries by name + author, not bookUrl.
        // Preserve the original entry's identity instead of adding a renamed duplicate.
        name: index === 0 ? template.name : 'Layout fixture ' + String(index).padStart(2, '0'),
        author: index === 0 ? template.author : 'Loopback fixture',
        intro,
        // The legacy source cache can refresh intro; customIntro remains the user's override.
        customIntro: intro,
        canUpdate: false,
      }),
    });
    const reply = await response.json();
    if (response.status !== 200 || !reply.isSuccess) throw new Error('Fixture saveBook failed');
  }
  const verification = await fetch('/reader3/getBookshelf', { credentials: 'same-origin' });
  const saved = await verification.json();
  if (verification.status !== 200 || !saved.isSuccess || saved.data.length !== 15) {
    throw new Error('Expected 15 persisted fixture books; count=' + saved.data?.length);
  }
  if (!saved.data.every(book => book.customIntro === intro)) {
    throw new Error('Fixture custom introduction was not persisted');
  }
  return saved.data.length;
}
