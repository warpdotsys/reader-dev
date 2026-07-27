# simple-web Source Code

This directory contains the readable, reverse-engineered source code for the `simple-web` lightweight reader interface. The compiled/minified output lives in `src/main/resources/simple-web/`.

## Directory Structure

```
simple-web-src/
  js/           - JavaScript source files
  css/          - CSS stylesheets
  html/         - HTML pages
  templates/    - Template files (*.tmpl)
  fonts/        - Font references (actual TTF files in assets/font/)
```

## JavaScript Modules

| File | Description |
|------|-------------|
| `js/common.js` | Core library: polyfills, custom `_$` jQuery-like DOM library, `_$.ajax`, URL/cookie utilities, common UI helpers (Menu, Pagination, Watcher, etc.) |
| `js/template.js` | template.js v0.7.1 - template engine library (from https://github.com/yanhaijing/template.js) |
| `js/template-data.js` | Pre-compiled template functions and template rendering helpers (showTip, renderTmpl, etc.) |
| `js/polyfill.js` | Tiny Promise polyfill detection (browserify bundle stub) |
| `js/indexPage.js` | Bookshelf (index) page logic: book list, search/filter, pagination |
| `js/readerPage.js` | Reader page logic: chapter loading, pagination, font settings, renderers (Text and HTML), ToC, book source switching |
| `js/searchPage.js` | Search page logic: multi-source book search, results display, book info |
| `js/rssPage.js` | RSS reader page logic: source list, article list, article content display, pagination |

## CSS Files

| File | Description |
|------|-------------|
| `css/layout.css` | Main layout styles (normalize.css base, flex layout, menu, pagination icons, dark theme) |
| `css/read.css` | Reader-specific styles (font faces, font sizes, line heights, article formatting) |

## HTML Pages

| File | Description |
|------|-------------|
| `html/index.html` | Bookshelf page |
| `html/reader.html` | Reading page with font/layout settings menu |
| `html/search.html` | Book search page |
| `html/rss.html` | RSS reader page |

## Templates

Template files use the `<% %>` syntax from template.js v0.7.1:
- `bookList.tmpl` - Book list rendering (used in index and search)
- `bookInfo.tmpl` - Book detail popup
- `rssList.tmpl` - RSS source list
- `articleList.tmpl` - RSS article list
- `searchSourceList.tmpl` - Book source selection prompt
- `sourceList.tmpl` - Available book sources list

## Architecture

The app is a vanilla JavaScript single-page-style application with 4 HTML entry points. Key patterns:

1. **Reactive state** - Uses a custom `Watcher` class for observable values with `onChange` callbacks
2. **Signal-based reactivity** (search/rss pages) - Uses `createSignal`/`createEffect` for fine-grained reactivity
3. **Custom DOM library** (`_$`) - Minimal jQuery-like API for DOM manipulation
4. **Template rendering** - Server-side style templates rendered client-side via template.js
5. **API communication** - All API calls go through `_$.ajax` with `baseURL` and `token`
6. **Page pagination** - Custom `PageContainer` class that splits content into pages by viewport height

## API Endpoints Used

- `/getBookshelf` - Get user's bookshelf
- `/getChapterList` - Get chapter list for a book
- `/getBookContent` - Get chapter content
- `/searchBook` - Search in single source
- `/searchBookMulti` - Search across multiple sources
- `/saveBook` - Add book to shelf
- `/deleteBook` - Remove book from shelf
- `/saveBookProgress` - Save reading progress
- `/getBookInfo` - Get book metadata
- `/getRssSources` - Get RSS source list
- `/getRssArticles` - Get articles from RSS source
- `/getRssContent` - Get RSS article content
- `/setBookSource` - Switch book source
- `/getAvailableBookSource` - Get available sources for a book
- `/searchBookSource` - Search for more book sources
- `/getBookSources` - Get all configured book sources
- `/getUserInfo` - Get current user info
- `/login` - User login
- `/logout` - User logout

## Build / Deploy

Currently there is no automated build pipeline for simple-web. The minified files in
`src/main/resources/simple-web/` are the deployed versions with content-hash filenames.

To deploy changes from this source:
1. Edit files in `simple-web-src/`
2. Minify JS/CSS (e.g., using terser/cssnano)
3. Add content hashes to filenames
4. Update HTML `<script>` and `<link>` references
5. Copy to `src/main/resources/simple-web/`
