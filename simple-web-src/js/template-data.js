/**
 * template-data.js - Pre-compiled Template Functions and UI Helpers
 *
 * This file contains:
 * 1. Pre-compiled template render functions (window.__TMPL__)
 * 2. UI helper functions: showTip, closeTip, isHtml
 * 3. Title manager
 * 4. Token/API initialization
 * 5. Template loading helpers (requireTmpl, renderTmpl)
 * 6. Watcher reactive state class
 * 7. Book source management (setBookSource, loadMoreBookSource)
 * 8. Login/Settings/Page positioning UI
 * 9. BookApi class
 * 10. Pagination class
 * 11. Menu class
 */

// ============================================================
// Section 1: Tip Dialog System
// ============================================================

/**
 * Show a tip/modal dialog
 * @param {string} title - Dialog title
 * @param {string} closeText - Close button text
 * @param {string} bodyHtml - Dialog body HTML content
 * @param {string} extraClass - Optional CSS class to add
 */
function showTip(title, closeText, bodyHtml, extraClass) {
    .html(title);
    .html(closeText);
    .html(bodyHtml);
    .css("display", "block");

    // Remove previous extra class if any
    var lastClass = .attr("data-lastclass");
    if (lastClass) {
        .removeClass(lastClass);
    }

    // Add new extra class
    if (extraClass) {
        .addClass(extraClass);
        .attr("data-lastclass", extraClass);
    }

    .attr("class", "dropdown-backdrop");
}

/**
 * Close the tip dialog
 */
function closeTip() {
    .attr("class", "");
    .css("display", "none");
    var lastClass = .attr("data-lastclass");
    if (lastClass) {
        .removeClass(lastClass);
    }
}

/**
 * Check if a string contains HTML markup
 */
function isHtml(str) {
    return /<[a-z]+\d?(\s+[\w-]=("[^"]*"|'[^']*'))*\s*\/?>|&#?\w+;/i.test(str);
}

// Tip dialog event handlers
.click(function () { closeTip(); });
.click(function (e) { closeTip(); });

// ============================================================
// Section 2: Title Manager
// ============================================================

var Title = function () {
    this.title = document.title;
    this.set = function (t) { document.title = t; };
    this.reset = function () { document.title = this.title; };
    this.update = function (suffix) { document.title = this.title + " - " + suffix; };
};

// ============================================================
// Section 3: Token and API Initialization
// ============================================================

var token = $.getUrlPra("accessToken");
if (token) {
    $.cookie.set("token", token, 30, "/");
} else {
    token = $.cookie.get("token") || window.myStorage.getItem("api_token");
}
$.ajax.token = token || "";

var api = $.getUrlPra("api");
if (api) {
    $.cookie.set("api", api, 30, "/");
} else {
    api = $.cookie.get("api");
}
$.ajax.baseURL = api || window.myStorage.getItem("api_prefix") || "/reader3";

// Response interceptor - check for login required
$.ajax.onResponse = function (responseText) {
    try {
        var data = JSON.parse(responseText);
        if (data && data.data === "NEED_LOGIN") {
            isLoginWatcher.update(false);
        }
    } catch (e) {}
    return responseText;
};

// ============================================================
// Section 4: Template Loading Helpers
// ============================================================

/**
 * Load a template by name (from cache or server)
 * @param {string} name - Template name (without .tmpl extension)
 * @param {function} callback - Called with template string or pre-compiled function
 */
function requireTmpl(name, callback) {
    window.__TMPL__ = window.__TMPL__ || {};
    if (window.__TMPL__[name]) {
        callback(window.__TMPL__[name]);
    } else {
        $.ajax.get("./assets/template/" + name + ".tmpl", function (tmplStr) {
            callback(tmplStr);
        }, true);
    }
}

/**
 * Render a template with data
 * @param {string} name - Template name
 * @param {object} data - Data to render
 * @param {function} callback - Called with rendered HTML string
 */
function renderTmpl(name, data, callback) {
    requireTmpl(name, function (tmpl) {
        var html;
        if (typeof tmpl === "string") {
            html = template(tmpl, data);
        } else {
            html = template.generate(tmpl, data);
        }
        callback(html);
    });
}

// ============================================================
// Section 5: Watcher - Reactive State Class
// ============================================================

/**
 * Watcher - Observable value with change listeners
 * @param {*} initialValue - Initial value
 */
function Watcher(initialValue) {
    var listeners = [];
    var currentValue = initialValue;
    var previousValue = null;

    /**
     * Register a change listener
     * @param {function} fn - Callback(newValue, oldValue)
     * @param {boolean} immediate - If true, call immediately with current value
     */
    this.onChange = function (fn, immediate) {
        if (fn) {
            listeners.push(fn);
            if (immediate) {
                fn(currentValue, previousValue);
            }
        }
    };

    /** Trigger all listeners with current value */
    this.trigger = function () {
        for (var i = 0; i < listeners.length; i++) {
            listeners[i](currentValue, previousValue);
        }
    };

    /** Update the value and trigger if changed */
    this.update = function (newValue) {
        previousValue = currentValue;
        currentValue = newValue;
        if (currentValue !== previousValue) {
            this.trigger();
        }
    };

    /** Get current value */
    this.getValue = function () {
        return currentValue;
    };
}

// ============================================================
// Section 6: Book Source Management
// ============================================================

var sourceListWatcher = new Watcher([]);

/**
 * Set the book source for a book
 */
function setBookSource(bookUrl, index) {
    var sourceList = sourceListWatcher.getValue();
    if (!sourceList || !sourceList[index]) return;

    var source = sourceList[index];
    $.ajax.post("/setBookSource", {
        bookUrl: bookUrl,
        newUrl: source.bookUrl,
        bookSourceUrl: source.origin
    }, function (resp) {
        var result = JSON.parse(resp);
        if (result.isSuccess) {
            try {
                location.replace("./reader.html?bookUrl=" + encodeURIComponent(source.bookUrl));
            } catch (e) {
                location.href = "./reader.html?bookUrl=" + encodeURIComponent(source.bookUrl);
            }
        } else {
            alert(result.errorMsg);
        }
    }, true);
}

var lastSourceIndex = 0;

/**
 * Load more book sources for a given book URL
 */
function loadMoreBookSource(bookUrl) {
    var currentList = sourceListWatcher.getValue();
    if (typeof currentList === "object") {
        lastSourceIndex = lastSourceIndex || currentList.length;
    }

    $.ajax.post("/searchBookSource", {
        url: bookUrl,
        lastIndex: lastSourceIndex
    }, function (resp) {
        try {
            var result = JSON.parse(resp);
            if (result.isSuccess) {
                var newItems = result.data.list || [];
                var existing = sourceListWatcher.getValue();
                existing = Array.isArray(existing) ? existing : [];

                // Deduplicate by bookUrl
                var seen = {};
                existing.map(function (item) { seen[item.bookUrl] = 1; });
                var merged = [].concat(existing, newItems.filter(function (item) {
                    return !seen[item.bookUrl];
                }));

                if (result.data.lastIndex) {
                    lastSourceIndex = result.data.lastIndex;
                }
                sourceListWatcher.update(merged);
            }
        } catch (e) {
            sourceListWatcher.update("error");
        }
    }, true);
}

// ============================================================
// Section 7: Login System
// ============================================================

function showLoginBox() {
    showTip("登录", "关闭",
        "<div class='login-form'>
" +
        "    <form action='/login' onsubmit='return doLogin(this, true);'>
" +
        "       <label>用户名：<input type='text' name='username' /></label>
" +
        "       <label>密　码：<input type='text' name='password' /></label>
" +
        "       <label>邀请码：<input type='text' name='code' /></label>
" +
        "       <button type='submit' onclick='doLogin(this.parentNode, false)'>注册</button>
" +
        "       <button type='submit' onclick='doLogin(this.parentNode, true)'>登录</button>
" +
        "   </form>
" +
        "</div>",
        "top-tip"
    );
}

/**
 * Submit a form via AJAX
 */
function submitForm(form, transform, callback) {
    var data = {};
    var inputs = form.querySelectorAll("[name]");
    for (var i = 0; i < inputs.length; i++) {
        var input = inputs[i];
        data[input.getAttribute("name")] = input.value;
    }
    data = transform ? transform(data) : data;
    if (data) {
        $.ajax.post(form.getAttribute("action"), data, function (resp) {
            try {
                var result = JSON.parse(resp);
                if (callback) callback(result);
            } catch (e) {}
        }, false);
    }
}

function doLogin(form, isLogin) {
    submitForm(form, function (data) {
        if (!data.username || !data.password) return false;
        data.isLogin = isLogin;
        return data;
    }, function (result) {
        if (result.isSuccess) {
            closeTip();
            if (result.data && result.data.accessToken) {
                token = result.data.accessToken;
                $.cookie.set("token", token, 30, "/");
                $.ajax.token = token;
                isLoginWatcher.update(true);
            }
        }
    });
    return false;
}

function doLogout() {
    $.ajax.post("/logout", "", function (resp) {
        try {
            var result = JSON.parse(resp);
            if (result && result.isSuccess) {
                token = "";
                $.cookie.set("token", token, -1, "/");
                $.ajax.token = token;
                isLoginWatcher.update(false);
            }
        } catch (e) {}
    }, false);
}

// Login watcher - shows login box when logged out
var isLoginWatcher = new Watcher(true);
isLoginWatcher.onChange(function (loggedIn) {
    if (!loggedIn) showLoginBox();
});

// ============================================================
// Section 8: Dark Theme
// ============================================================

window.darkThemeWatcher = new Watcher(false);

var isBlack = $.cookie.get("black");
if (isBlack && isBlack.toString() === "true") {
    window.darkThemeWatcher.update(true);
    .attr("class", "dark-theme");
} else {
    .attr("class", "white-theme");
}

// ============================================================
// Section 9: Settings and Page Position
// ============================================================

function showSettingBox() {
    var pageName = window.location.pathname.split("/");
    pageName = pageName[pageName.length - 1].replace(".html", "").toLowerCase();
    var saved = $.cookie.get(pageName + "bj");
    var values = [0, 0, 0, 0, window.innerHeight, 0];
    if (saved) {
        values = saved.split(",");
        if (values.length < 5) values.push(window.innerHeight);
        if (values.length < 6) values.push(0);
    }

    showTip("设置", "关闭",
        "<div class='login-form'>
" +
        "    <form action='' onsubmit='return saveSetting(this);'>
" +
        "       <label>上边界：<input type='text' name='top' value='" + values[0] + "' /></label>
" +
        "       <label>下边界：<input type='text' name='bottom' value='" + values[1] + "' /></label>
" +
        "       <label>左边界：<input type='text' name='left' value='" + values[2] + "' /></label>
" +
        "       <label>右边界：<input type='text' name='right' value='" + values[3] + "' /></label>
" +
        "       <label>最大高度：<input type='text' name='maxHeight' value='" + values[4] + "' /></label>
" +
        "       <input id='hideSBInput' type='hidden' name='hideSB' value='" + values[5] + "' />
" +
        "       <label>隐藏滚动条：<span style='border: 1px solid #ddd;display: inline-block; padding: 3px 10px;' onclick='toggleScrollBarHidden(this)'>" +
        (values[5] == 1 ? "已开启" : "已关闭") + "</span></label>
" +
        "       <button type='submit'>保存</button>
" +
        "   </form>
" +
        "</div>",
        "top-tip"
    );
}

function clearStorage() {
    if (window.myStorage && window.myStorage.clear) {
        window.myStorage.clear();
    }
}

function showErrorMsg() {
    showTip("温馨提示", "关闭", "<pre class='notice-info'>
" + errorMsg + "</pre>");
}

function showNotice(msg) {
    showTip("温馨提示", "关闭", "<pre class='notice-info'>
" + msg + "</pre>");
}

function isInteger(val) {
    return Math.floor(val) === val;
}

function saveSetting(form) {
    submitForm(form, function (data) {
        if (!isInteger(parseInt(data.top)) || !isInteger(parseInt(data.bottom)) ||
            !isInteger(parseInt(data.left)) || !isInteger(parseInt(data.right))) {
            showNotice("边界值只能是数字");
            return false;
        }
        if (!isInteger(parseInt(data.maxHeight))) {
            showNotice("最大高度只能是数字");
            return false;
        }
        if (parseInt(data.maxHeight) < window.innerHeight / 2) {
            showNotice("最大高度不能小于窗口的一半");
            return false;
        }

        var pageName = window.location.pathname.split("/");
        pageName = pageName[pageName.length - 1].replace(".html", "").toLowerCase();
        var cookieVal = parseInt(data.top) + "," + parseInt(data.bottom) + "," +
            parseInt(data.left) + "," + parseInt(data.right) + "," +
            data.maxHeight + "," + data.hideSB;
        $.cookie.set(pageName + "bj", cookieVal, 0);
        updatePagePosition();
        return false;
    }, false);
    return false;
}

function toggleScrollBarHidden(el) {
    var val = .val();
    if (parseInt(val) === 1) {
        .val("0");
        el.innerText = "已关闭";
    } else {
        .val("1");
        el.innerText = "已开启";
    }
}

function updatePagePosition() {
    var pageName = window.location.pathname.split("/");
    pageName = pageName[pageName.length - 1].replace(".html", "").toLowerCase();
    var saved = $.cookie.get(pageName + "bj");
    var box = ;

    if (saved) {
        var vals = saved.split(",");
        box.css("top", (0 | vals[0]) + "px");
        box.css("bottom", (0 | vals[1]) + "px");
        box.css("left", (0 | vals[2]) + "px");
        box.css("right", (0 | vals[3]) + "px");

        if (vals.length >= 6) {
            if (0 | vals[5]) {
                var maxH = 0 | vals[4];
                maxH = maxH > window.innerHeight / 2 ? maxH : window.innerHeight;
                .css("max-height", maxH + "px");
                .css("overflow", "hidden");
                .css("max-height", maxH + "px");
                .css("overflow", "hidden");
            } else {
                if (.css("max-height")) .css("max-height", "none");
                .css("overflow", "auto");
                if (.css("max-height")) .css("max-height", "none");
                .css("overflow", "auto");
            }

            if (window.reader && window.reader.viewDisplay) {
                window.reader.viewDisplay();
            }
        }
    }
}

// ============================================================
// Section 10: Utility Functions
// ============================================================

/**
 * Resolve a relative URL against a base URL
 */
function resolveUrl() {
    var argc = arguments.length;
    if (argc === 0) throw new Error("resolveUrl requires at least one argument; got none.");

    var base = document.createElement("base");
    base.href = arguments[0];
    if (argc === 1) return base.href;

    var head = document.getElementsByTagName("head")[0];
    head.insertBefore(base, head.firstChild);

    var anchor = document.createElement("a");
    var resolved;
    for (var i = 1; i < argc; i++) {
        anchor.href = arguments[i];
        resolved = anchor.href;
        base.href = resolved;
    }
    head.removeChild(base);
    return resolved;
}

/**
 * Debounce a function call
 */
function debounce(fn, delay) {
    var timer;
    return function () {
        var context = this;
        clearTimeout(timer);
        var args = arguments;
        timer = setTimeout(function () {
            fn.apply(context, args);
        }, delay);
    };
}

/**
 * Switch active menu tab
 */
function chooseMenu(element, sectionId) {
    .removeClass("choose");
    .addClass("choose");
    .addClass("choose");
}

/**
 * Load user info from server
 */
function loadUserInfo() {
    $.ajax.get("/getUserInfo", function (resp) {
        var result = JSON.parse(resp);
        if (result.isSuccess) {
            console.log(result);
        }
    }, true);
}

/**
 * Register a callback to run on page mount
 */
function onMounted(fn) {
    if (fn) window.__onMountedHook.push(fn);
}

// ============================================================
// Section 11: Page Initialization
// ============================================================

window.__onMountedHook = [loadUserInfo];

window.onload = function () {
    updatePagePosition();
    window.__onMountedHook.forEach(function (fn) { return fn(); });
};

// ============================================================
// Section 12: BookApi Class
// ============================================================

var bookListWatcher = new Watcher([]);

var BookApi = function () {
    /** Update a book in the bookshelf list */
    this.updateBook = function (book) {
        var list = bookListWatcher.getValue();
        for (var i = 0; i < list.length; i++) {
            if (book.bookUrl === list[i].bookUrl) {
                list[i] = book;
                break;
            }
        }
        bookListWatcher.update(list);
    };

    /** Fetch the bookshelf from server */
    this.getBookshelf = function (callback) {
        $.ajax.get("/getBookshelf", function (resp) {
            var result = JSON.parse(resp);
            if (result.isSuccess) {
                for (var i = 0; i < result.data.length; i++) {
                    result.data[i].id = i;
                    result.data[i].title = result.data[i].name;
                }
                bookListWatcher.update(result.data);
                if (callback) callback();
            }
        }, true);
    };

    /** Find a book by URL */
    this.getBookInfoByUrl = function (url) {
        var list = bookListWatcher.getValue();
        for (var i = 0; i < list.length; i++) {
            if (url === list[i].bookUrl) return list[i];
        }
        return null;
    };

    this.find = function (index) {
        return (index >= 0 && index < this.book.length) ? index : -1;
    };

    this.isFavourite = function (index) {
        return this.find(index) === -1 ? "收藏本书" : "取消收藏";
    };

    this.favourite = function (index) {
        if (this.find(index) === -1) {
            this.insert();
            return true;
        } else {
            this.delete(index);
            return false;
        }
    };

    this.getData = function (index) {
        var i = this.find(index);
        if (i === -1) {
            return {
                title: "", id: "", readCount: 0, totalCount: 1,
                index: 0, readChapter: "", totalChapter: "",
                siteName: "", author: "", page: 0
            };
        }
        return this.book[i];
    };
};

// ============================================================
// Section 13: Pagination Class
// ============================================================

var Pagination = function () {
    this.list = [];
    this.page = 1;
    this.pageCount = 1;
    this.next = ;
    this.before = ;
    this.pageIndexList = [];
    this.containerHeight = 0;
    this.onPageChange = null;

    /**
     * Initialize pagination for a list of elements
     * @param {string} itemSelector - CSS selector for list items
     * @param {number} offset - Height offset
     * @param {string} containerSelector - Container selector for height calculation
     * @param {function} onPageChange - Callback when page changes
     * @param {number} initialPage - Starting page number
     */
    this.init = function (itemSelector, offset, containerSelector, onPageChange, initialPage) {
        this.list = ;
        if (!this.list || !this.list.length) return;

        this.onPageChange = onPageChange;
        containerSelector = containerSelector || ".right_t.flexone";
        this.containerHeight = [0].offsetHeight - offset;
        this.computePage();
        this.page = initialPage || 1;
        this.display(this.page);

        if (this.pageCount <= 1) {
            this.set("before", "beforeN");
            this.set("next", "nextN");
        }
    };

    /** Compute page boundaries based on item heights */
    this.computePage = function () {
        var height = 0;
        this.pageIndexList = [0];

        for (var i = 0; i < this.list.length; i++) {
            if (height + this.list[i].offsetHeight >= this.containerHeight) {
                this.pageIndexList.push(i);
                height = this.list[i].offsetHeight;
            } else {
                height += this.list[i].offsetHeight;
            }
        }
        if (height > 0) {
            this.pageIndexList.push(i);
        }
        this.pageCount = this.pageIndexList.length - 1;
        console.log(this.pageIndexList);
    };

    /** Set pagination icon state */
    this.set = function (which, className) {
        .removeClass(which + "Y").removeClass(which + "N").addClass(className);
    };

    this.beforeClick = function () { this.display(this.page - 1); };
    this.nextClick = function () { this.display(this.page + 1); };

    /** Display a specific page */
    this.display = function (page) {
        if (page < 1 || page > this.pageCount) return;
        if (this.onPageChange) this.onPageChange(page);

        console.log("display", page);
        this.page = page;

        // Hide all items
        this.list.css("display", "none");

        // Show items for current page
        var start = this.pageIndexList[this.page - 1];
        var end = this.pageIndexList[this.page];
        for (var i = start; i < end; i++) {
            .css("display", "block");
        }

        // Update pagination icons
        if (this.page === this.pageCount) {
            this.set("next", "nextN");
        } else {
            this.set("next", "nextY");
        }
        if (this.page === 1) {
            this.set("before", "beforeN");
        } else {
            this.set("before", "beforeY");
        }
    };
};
