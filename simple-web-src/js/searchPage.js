/**
 * searchPage.js - Book Search Page Logic
 *
 * This module handles the multi-source book search page:
 * - Reactive signal-based state management (createSignal/createEffect)
 * - Multi-source search with deduplication
 * - Single source search (bookSourceName@keyword)
 * - Group source search (groupName#keyword)
 * - Book info popup with save-to-shelf
 * - Book source switching from search results
 * - Pagination of search results
 *
 * Dependencies: common.js, template-data.js
 */

// ============================================================
// Section 1: Reactive Primitives (SolidJS-like signals)
// ============================================================

var _observers = [];
var _isObserveSignal = true;

/**
 * Create a reactive effect that re-runs when its dependencies change
 */
function createEffect(fn) {
    function effect() {
        _observers.push(effect);
        try {
            fn();
        } finally {
            _observers.pop();
        }
    }
    effect();
}

/**
 * Read a value without tracking it as a dependency
 */
function untrack(fn) {
    return (function () {
        _isObserveSignal = false;
        try {
            return fn();
        } finally {
            _isObserveSignal = true;
        }
    })();
}

/**
 * Create a reactive signal (getter/setter pair)
 * @param {*} initialValue - Initial signal value
 * @param {object} options - Options with optional `equals` comparator
 * @returns {Array} [getter, setter]
 */
function createSignal(initialValue, options) {
    var defaults = { equals: false };
    options = options || defaults;
    options.equals = (options.hasOwnProperty('equals') ? options : defaults).equals;

    var state = {
        value: initialValue,
        subscribers: new Set(),
        comparator: options.equals || undefined
    };

    return [
        readSignal.bind(state),
        function (newValue) {
            if (typeof newValue === 'function') {
                newValue = newValue(state.value);
            }
            return writeSignal(state, newValue);
        }
    ];
}

function readSignal() {
    if (_isObserveSignal) {
        var observer = _observers[_observers.length - 1];
        if (observer) {
            this.subscribers.add(observer);
        }
    }
    return this.value;
}

function writeSignal(state, newValue) {
    if (state.comparator && state.comparator(state.value, newValue)) {
        return newValue;
    }
    state.value = newValue;
    state.subscribers.forEach(function (fn) { return fn(); });
    return newValue;
}

/**
 * Create a derived/computed signal
 */
function createMemo(fn) {
    var pair = createSignal();
    var getter = pair[0];
    var setter = pair[1];
    createEffect(function () { return setter(fn()); });
    return getter;
}

// ============================================================
// Section 2: Search State
// ============================================================

var _s1 = createSignal(false);
var isSearching = _s1[0], setIsSearching = _s1[1];

var _s2 = createSignal([]);
var searchBookList = _s2[0], setSearchBookList = _s2[1];

var _s3 = createSignal([]);
var bookSourceList = _s3[0], setBookSourceList = _s3[1];

var _s4 = createSignal([]);
var bookShelfList = _s4[0], setBookShelfList = _s4[1];

var _s5 = createSignal(null);
var bookInfo = _s5[0], setBookInfo = _s5[1];

var bookListContainer = $("#bookList");
var pagination = new Pagination();
var searchConfig = null;
var bookSourceGroupList = [];

window.hasSearched = false;

// ============================================================
// Section 3: Search Functions
// ============================================================

function onSearchBookListPageChange(page) {
    if (page === pagination.pageCount && searchConfig && !searchConfig.isEnd) {
        searchConfig.page += 1;
        searchBookWithConfig(searchConfig);
    }
}

function search() {
    var keyword = $("#keyword").val();
    if (keyword) {
        searchBook(keyword);
    } else {
        showNotice("\u8bf7\u8f93\u5165\u5173\u952e\u5b57");
    }
}

function searchBookWithSourceIndex(index) {
    closeTip();
    searchBook($("#keyword").val(), index);
}

function searchBook(keyword, sourceIndex) {
    if (isSearching()) return;

    searchConfig = getSearchConfig(keyword, sourceIndex);
    if (!searchConfig) return;

    window.hasSearched = true;
    setSearchBookList([]);
    setIsSearching(true);
    searchBookWithConfig(searchConfig);
}

function searchBookWithConfig(config) {
    // Reset pagination for first page
    if ((config.isMulti && config.lastIndex === 0) || (!config.isMulti && config.page === 1)) {
        pagination.page = 1;
    }

    $.ajax.post(config.isMulti ? "/searchBookMulti" : "/searchBook", {
        key: config.key,
        bookSourceUrl: config.bookSourceUrl,
        bookSourceGroup: config.bookSourceGroup,
        concurrentCount: config.concurrentCount,
        lastIndex: config.lastIndex,
        page: config.page
    }, function (resp) {
        var result = JSON.parse(resp);
        if (!result.isSuccess) return;

        var newItems = [];
        if (config.isMulti) {
            searchConfig.lastIndex = result.data.lastIndex;
            newItems = result.data.list;
        } else {
            newItems = result.data;
        }

        // Merge with existing results, deduplicating by name+author
        var existing = [];
        if (config.page > 1 || config.lastIndex > 0) {
            existing = [].concat(searchBookList());
        }

        var byNameAuthor = {};
        var byUrl = {};
        var prevLength = existing.length;

        existing.reduce(function (acc, book) {
            acc[book.name + "_" + book.author] = book;
            byUrl[book.bookUrl] = book;
            return acc;
        }, byNameAuthor);

        newItems.forEach(function (book) {
            if (byUrl[book.bookUrl]) return; // Skip exact duplicates

            if (byNameAuthor[book.name + "_" + book.author]) {
                // Same book from different source - increment source count
                var existingBook = byNameAuthor[book.name + "_" + book.author];
                existingBook.sourceCount = existingBook.sourceCount || 1;
                existingBook.sourceCount += 1;
            } else {
                // New unique book
                book.sourceCount = 1;
                byNameAuthor[book.name + "_" + book.author] = book;
                existing.push(book);
            }
        });

        setSearchBookList(existing);

        // Mark as ended if no new results
        if (existing.length === prevLength) {
            searchConfig.isEnd = true;
        }

        setIsSearching(false);
    });
}

/**
 * Parse search keyword into a search configuration
 * Supports: plain keyword, groupName#keyword, sourceName@keyword
 */
function getSearchConfig(keyword, sourceIndex) {
    if (!keyword) return null;

    var config = {
        isMulti: true,
        bookSourceGroup: "",
        key: keyword,
        concurrentCount: 4,
        page: 1,
        lastIndex: 0,
        isEnd: false
    };

    if (keyword.indexOf("#") > 0) {
        // Group search: groupName#bookName
        config.bookSourceGroup = keyword.split("#")[0];
        config.key = keyword.split("#")[1];

        if (!config.bookSourceGroup) return config;

        var matchedGroups = bookSourceGroupList.filter(function (g) {
            return g.name.indexOf(config.bookSourceGroup) >= 0 ||
                   g.value.indexOf(config.bookSourceGroup) >= 0;
        });

        if (!matchedGroups || matchedGroups.length < 1) {
            showNotice("\u672a\u5339\u914d\u5230\u4e66\u6e90\u5206\u7ec4: " + config.bookSourceGroup);
            return;
        }

        if (matchedGroups.length > 1) {
            if (sourceIndex != null && sourceIndex >= 0 && sourceIndex < matchedGroups.length) {
                config.bookSourceGroup = matchedGroups[sourceIndex].value;
                return config;
            }
            renderTmpl("searchSourceList", {
                list: matchedGroups, keyword: config.key, isSourceGroup: true
            }, function (html) {
                showTip("\u8bf7\u9009\u62e9\u4e66\u6e90\u5206\u7ec4", "\u5173\u95ed",
                    "<div style='flex: 1'>" + html + "</div>");
            });
            return;
        }

        config.bookSourceGroup = matchedGroups[0].value;

    } else if (keyword.indexOf("@") > 0) {
        // Single source search: sourceName@bookName
        config.isMulti = false;
        config.key = keyword.split("@")[1];
        var sourceName = keyword.split("@")[0];

        var matchedSources = bookSourceList().filter(function (s) {
            return s.bookSourceName.indexOf(sourceName) >= 0 ||
                   s.bookSourceUrl.indexOf(sourceName) >= 0;
        });

        if (!matchedSources || matchedSources.length < 1) {
            showNotice("\u672a\u5339\u914d\u5230\u4e66\u6e90: " + sourceName);
            return;
        }

        if (matchedSources.length > 1) {
            if (sourceIndex != null && sourceIndex >= 0 && sourceIndex < matchedSources.length) {
                config.bookSourceUrl = matchedSources[sourceIndex].bookSourceUrl;
                return config;
            }
            renderTmpl("searchSourceList", {
                list: matchedSources, keyword: config.key, isSourceGroup: false
            }, function (html) {
                showTip("\u8bf7\u9009\u62e9\u4e66\u6e90", "\u5173\u95ed",
                    "<div style='flex: 1'>" + html + "</div>");
            });
            return;
        }

        config.bookSourceUrl = matchedSources[0].bookSourceUrl;
    }

    return config;
}

// ============================================================
// Section 4: Book Info and Save
// ============================================================

function showBookInfo(index) {
    var book = searchBookList()[index];
    if (!book) {
        book = searchBookList().find(function (b) { return b.bookUrl === index; });
    }
    if (!book) return;

    var shelfBook = bookShelfList().find(function (b) { return b.bookUrl === book.bookUrl; });
    book.isInShelf = !!shelfBook;
    setBookInfo(book);
}

function saveBook() {
    var info = bookInfo();
    if (!info || !info.bookUrl || !info.name || !info.author) {
        showNotice("\u4e66\u7c4d\u4fe1\u606f\u9519\u8bef");
        return;
    }

    $.ajax.post("/saveBook", info, function (resp) {
        var result = JSON.parse(resp);
        if (result.isSuccess) {
            showNotice("\u52a0\u5165\u6210\u529f");
            loadBookShelfList(function () {
                showBookInfo(bookInfo().bookUrl);
            });
        }
    }, true);
}

function refreshChapterList(bookUrl) {
    var book = searchBookList().find(function (b) { return b.bookUrl === bookUrl; });
    if (!book) return;

    $.ajax.post("/getChapterList", { url: book.bookUrl, bookSourceUrl: book.origin }, function (resp) {
        var result = JSON.parse(resp);
        if (result.isSuccess) {
            setBookInfo(function (info) {
                info.totalChapterNum = result.data.length;
                info.latestChapterTitle = result.data[result.data.length - 1].title;
                return info;
            });
        }
    }, true);
}

// ============================================================
// Section 5: Reactive Effects (UI updates)
// ============================================================

// Render search results
createEffect(function () {
    renderTmpl("bookList", {
        list: searchBookList(),
        isLoading: isSearching(),
        isSearch: true
    }, function (html) {
        bookListContainer.html("");
        bookListContainer.append(html);
        var currentPage = pagination.page;
        pagination.init(".book-info", 0, "#bookList", onSearchBookListPageChange, currentPage);
    });
});

// Update book count display
createEffect(function () {
    var list = searchBookList();
    if (typeof list !== "string") {
        $("#bookNum").html("(" + list.length + ")");
    } else {
        $("#bookNum").html("(0)");
    }
});

// Show book info dialog when bookInfo changes
createEffect(function () {
    var info = bookInfo();
    if (info) {
        renderTmpl("bookInfo", {
            bookInfo: info,
            isSearch: true,
            encodeBookUrl: encodeURIComponent(info.bookUrl)
        }, function (html) {
            showTip(info.name, "\u5173\u95ed", html);
        });
    }
});

// ============================================================
// Section 6: Book Source Change (from search)
// ============================================================

var sourceListWatcher = new Watcher();

function changeSource(bookUrl) {
    var book = searchBookList().find(function (b) { return b.bookUrl === bookUrl; });
    if (!book) return;

    sourceListWatcher = new Watcher();
    sourceListWatcher.onChange(function (sources) {
        if (sources === "loading") {
            showTip(book.name + "(" + book.originName + ")", "\u5173\u95ed",
                '<div style="text-align: center;">\n    <b>\u4e66\u6e90\u52a0\u8f7d\u4e2d...\u8bf7\u7a0d\u540e...(\u65f6\u95f4\u8f83\u957f)</b>\n</div>');
        } else if (sources === "error") {
            showTip(book.name + "(" + book.originName + ")", "\u5173\u95ed",
                '<div style="text-align: center;">\n    <b>\u62b1\u6b49\uff0c\u627e\u4e0d\u5230\u8be5\u4e66\u7c4d,\u8bf7\u7a0d\u540e\u518d\u8bd5\uff01</b>\n</div>');
        } else if (sources && sources.length !== 0) {
            renderTmpl("sourceList", { bookUrl: bookUrl, list: sources }, function (html) {
                showTip(book.name + "(" + book.originName + ")", "\u5173\u95ed",
                    "<div style='flex: 1'>" + html + "</div>");
            });
        } else {
            showTip(book.name + "(" + book.originName + ")", "\u5173\u95ed",
                '<div style="text-align: center;">\n    <b onclick=\'loadMoreBookSource("' +
                bookUrl + '")\'> \u52a0\u8f7d\u66f4\u591a\u4e66\u6e90</b>\n</div>');
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
// Section 7: Load Book Sources
// ============================================================

function loadBookSourceList() {
    $.ajax.get("/getBookSources?simple=1", function (resp) {
        var result = JSON.parse(resp);
        if (!result.isSuccess) return;

        setBookSourceList(result.data);

        // Build group list
        var groupCounts = {};
        result.data.forEach(function (source) {
            if (source.bookSourceGroup) {
                source.bookSourceGroup.split(",").forEach(function (group) {
                    groupCounts[group] = 1 + (groupCounts[group] | 0);
                });
            }
        });

        var groups = [{ name: "\u5168\u90e8\u5206\u7ec4", value: "", count: result.data.length }];
        for (var name in groupCounts) {
            if (Object.hasOwnProperty.call(groupCounts, name)) {
                groups.push({ name: name, value: name, count: groupCounts[name] });
            }
        }
        bookSourceGroupList = groups;
    });
}

function loadBookShelfList(callback) {
    $.ajax.get("/getBookshelf", function (resp) {
        var result = JSON.parse(resp);
        if (result.isSuccess) {
            setBookShelfList(result.data);
            if (callback) callback();
        }
    });
}

// ============================================================
// Section 8: Page Mount
// ============================================================

onMounted(function () {
    (new Menu()).initMenu();
    isLoginWatcher.onChange(loadBookSourceList, true);
    $("#keyword").on("keyup", function (e) {
        if (e.key === "Enter") search();
    });
});
