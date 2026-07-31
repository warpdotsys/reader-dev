/**
 * indexPage.js - Bookshelf Page Logic
 *
 * This module handles the book shelf (index) page:
 * - Loading and displaying the user's bookshelf
 * - Filtering/searching books by keyword
 * - Showing book info popup
 * - Refreshing chapter lists
 * - Changing book sources
 * - Pagination of the book list
 *
 * Dependencies: common.js, template-data.js (Watcher, Pagination, BookApi, Menu, etc.)
 */

// ============================================================
// State
// ============================================================

var pagination = new Pagination();
var bookApi = new BookApi();
var showBookListWatcher = new Watcher([]);

// ============================================================
// Search / Filter
// ============================================================

/**
 * Check if a string contains a substring
 */
function contains(str, substr) {
    return str && str.indexOf(substr) >= 0;
}

/**
 * Called when the search keyword changes - recompute visible book list
 */
function onKeywordChange() {
    computeShowBookList();
}

/**
 * Filter and sort books based on keyword input
 */
function computeShowBookList() {
    var keyword = .val().replace(/^\s+/g, "").replace(/\s+$/g, "");
    var allBooks = bookListWatcher.getValue();

    // Sort by last read time (most recent first)
    allBooks = allBooks.sort(function (a, b) {
        var timeA = a.durChapterTime || 0;
        var timeB = b.durChapterTime || 0;
        return timeB - timeA;
    });

    if (keyword) {
        if (allBooks && allBooks.length > 0) {
            var filtered = [];
            for (var i = 0; i < allBooks.length; i++) {
                var book = allBooks[i];
                if (contains(book.name, keyword) ||
                    contains(book.author, keyword) ||
                    contains(book.kind, keyword)) {
                    filtered.push(book);
                }
            }
            showBookListWatcher.update(filtered);
        } else {
            showBookListWatcher.update([]);
        }
    } else {
        showBookListWatcher.update(allBooks);
    }
}

// ============================================================
// Page Initialization
// ============================================================

/**
 * Initialize the page after login is confirmed
 */
function onPageInit() {
    bookApi.getBookshelf();
    var container = ;

    showBookListWatcher.onChange(function (books) {
        if (books && books.length > 0) {
            renderTmpl("bookList", { list: books, isLoading: false, isSearch: false }, function (html) {
                container.html("");
                container.append(html);
                pagination.init(".book-info", 0);
            });
            .text("(" + books.length + ")");
        } else {
            .text("(0)");
            container.html("");
            container.append(
                "<div style='font-size: 1.6em;margin-top: 140px;text-align: center'>\n" +
                "        书架为空，请用其他浏览器搜索添加书籍\n" +
                "</div>"
            );
        }
    }, true);
}

// Re-filter when bookshelf data changes
bookListWatcher.onChange(function () {
    computeShowBookList();
});

// ============================================================
// Book Info Dialog
// ============================================================

var bookInfoWatcher = new Watcher();

/**
 * Show book info popup for book at given index
 */
function showBookInfo(index) {
    bookInfoWatcher.onChange(function (bookInfo) {
        renderTmpl("bookInfo", {
            bookInfo: bookInfo,
            isSearch: false,
            encodeBookUrl: encodeURIComponent(bookInfo.bookUrl)
        }, function (html) {
            showTip(bookInfo.name, "关闭", html);
        });
    });

    var book = showBookListWatcher.getValue()[index];
    bookInfoWatcher.update(book);
}

/**
 * Refresh chapter list for a book
 */
function refreshChapterList(bookUrl) {
    $.ajax.post("/getChapterList", { url: bookUrl }, function (resp) {
        var result = JSON.parse(resp);
        if (result.isSuccess) {
            var info = bookInfoWatcher.getValue();
            info.totalChapterNum = result.data.length;
            info.latestChapterTitle = result.data[result.data.length - 1].title;
            bookInfoWatcher.update(info);
            bookApi.updateBook(info);
        }
    }, true);
}

// ============================================================
// Book Source Switching
// ============================================================

var sourceListWatcher = new Watcher();

/**
 * Show book source change dialog
 */
function changeSource(bookUrl) {
    var bookInfo = bookApi.getBookInfoByUrl(bookUrl);

    sourceListWatcher.onChange(function (sources) {
        if (sources === "loading") {
            showTip(bookInfo.name + "(" + bookInfo.originName + ")", "关闭",
                '<div style="text-align: center;">\n    <b>书源加载中...请稍后...(时间较长)</b>\n</div>');
        } else if (sources === "error") {
            showTip(bookInfo.name + "(" + bookInfo.originName + ")", "关闭",
                '<div style="text-align: center;">\n    <b>抱歉，找不到该书籍,请稍后再试！</b>\n</div>');
        } else if (sources && sources.length !== 0) {
            renderTmpl("sourceList", { bookUrl: bookUrl, list: sources }, function (html) {
                showTip(bookInfo.name + "(" + bookInfo.originName + ")", "关闭",
                    "<div style='flex: 1'>" + html + "</div>");
            });
        } else {
            showTip(bookInfo.name + "(" + bookInfo.originName + ")", "关闭",
                '<div style="text-align: center;">\n    <b onclick='loadMoreBookSource("' +
                bookUrl + '")'> 加载更多书源</b>\n</div>');
        }
    }, true);

    sourceListWatcher.update("loading");

    $.ajax.post("/getAvailableBookSource", { url: bookUrl }, function (resp) {
        try {
            var result = JSON.parse(resp);
            if (result.isSuccess) {
                sourceListWatcher.update(result.data);
            } else {
                sourceListWatcher.update("error");
            }
        } catch (e) {
            sourceListWatcher.update("error");
        }
    }, true);
}

// ============================================================
// Boot
// ============================================================

(new Menu()).initMenu();
isLoginWatcher.onChange(onPageInit, true);
