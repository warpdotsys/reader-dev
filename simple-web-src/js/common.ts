/**
 * common.js - Core Library for Simple-Web Reader
 *
 * This file provides:
 * 1. Utility functions (loadScript, loadLink, getQueryString)
 * 2. Polyfills (Function.bind, Set, Array methods, Element.matches)
 * 3. Custom _$ (jQuery-like) DOM library
 * 4. _$.ajax - AJAX helper with baseURL and token
 * 5. _$.getUrlPra, _$.url, _$.cookie - URL and cookie utilities
 * 6. Common UI components (Watcher, Pagination, Menu, BookApi)
 * 7. Login, settings, and page positioning logic
 *
 * Note: template.js v0.7.1 is loaded separately in the same <script> tag
 * in the minified version but lives in its own file here.
 */

// ============================================================
// Section 1: Utility Functions
// ============================================================

/**
 * Determine the type of a value (ES5-compatible typeof helper)
 */
function _typeof(t: any): any {
    return (_typeof = typeof Symbol === 'function' && typeof Symbol.iterator === 'symbol'
        ? function (t: any) { return typeof t; }
        : function (t: any) {
            return t && typeof Symbol === 'function' && t.constructor === Symbol && t !== Symbol.prototype
                ? 'symbol' : typeof t;
        }
    )(t);
}

/**
 * Dynamically load a script element
 * @param {string} src - Script URL
 * @param {function} onload - Callback when loaded
 * @param {boolean} async - Whether to load asynchronously
 */
function loadScript(src: string, onload: any, async: boolean) {
    var script = document.createElement('script');
    if (async) {
        script.async = 'async';
    }
    script.src = src;
    if (onload) {
        script.onload = onload;
    }
    document.head.appendChild(script);
}

/**
 * Dynamically load a CSS link element
 * @param {string} href - CSS URL
 * @param {function} onload - Callback when loaded
 * @param {string} rel - Link rel attribute (default: 'stylesheet')
 * @param {string} type - Link type attribute (default: 'text/css')
 */
function loadLink(href: string, onload: any, rel?: string, type?: string) {
    var link = document.createElement('link');
    link.rel = rel || 'stylesheet';
    link.type = type || 'text/css';
    link.href = href;
    if (onload) {
        link.onload = onload;
    }
    document.getElementsByTagName('head')[0].appendChild(link);
}

/**
 * Get a query string parameter from the current URL
 * @param {string} name - Parameter name
 * @param {boolean} caseSensitive - Whether matching is case-sensitive
 * @returns {string} Decoded parameter value or empty string
 */
function getQueryString(name: string, caseSensitive?: boolean): string {
    return decodeURIComponent(
        window.location.search.replace(
            new RegExp(
                '^(?:.*' + (caseSensitive ? '?' : '') + '[&\\?]' +
                encodeURI(name).replace(/[\.\+\*]/g, '\\$&') +
                '(?:\\=([^&]*))?)?.*$',
                'i'
            ),
            '$1'
        )
    );
}

// ============================================================
// Section 2: Storage Fallback
// ============================================================

/**
 * Fake localStorage for environments where localStorage is unavailable
 */
var fakeStorage = {
    items: {},
    isFake: true,
    getItem: function (key: any) { return this.items[key]; },
    setItem: function (key: any, value: any) { this.items[key] = value; },
    clear: function () { this.items = {}; },
    removeItem: function (key: any) { delete this.items[key]; }
};

window.myStorage = window.localStorage || fakeStorage;

// ============================================================
// Section 3: Debug and Error Handling
// ============================================================

window.isDebug = window.getQueryString('debug');

var errorMsg = window.navigator.userAgent + '\n';

/**
 * Check if a function is native (not user-defined)
 */
function isNative(fn: any): boolean {
    return fn !== null && /native code/.test(fn.toString());
}

/**
 * Global error handler - logs errors and shows alert
 */
window.onerror = function (message: any, url: any, line: any, column: any, error: any) {
    var errorStr = [
        'Type:' + _typeof(message),
        'Message: ' + message,
        'URL: ' + url,
        'Line: ' + line,
        'Column: ' + column,
        'Error object: ' + JSON.stringify(error)
    ].join('  ');

    errorMsg += errorStr + '\n';
    console.error(errorStr);

    // Try to use showErrorMsg if available, otherwise alert
    (showErrorMsg && showErrorMsg(errorStr)) || alert(errorStr);
    return false;
};

// Load vConsole in debug mode
if (window.isDebug) {
    window.loadScript(
        'https://cdn.bootcdn.net/ajax/libs/vConsole/3.9.1/vconsole.min.js',
        function () { new VConsole(); },
        true
    );
}

// Kindle device detection
window.isKindle = window.navigator.userAgent.toLowerCase().indexOf('kindle') >= 0;

// ============================================================
// Section 4: Polyfills
// ============================================================

// Function.prototype.bind polyfill
if (!Function.prototype.bind) {
    Function.prototype.bind = function (context: any) {
        if (typeof this !== 'function') {
            throw new TypeError('Function.prototype.bind - what is trying to be bound is not callable');
        }
        var args = Array.prototype.slice.call(arguments, 1);
        var self = this;
        var NOP = function () {};
        var bound = function () {
            return self.apply(
                this instanceof NOP ? this : context,
                args.concat(Array.prototype.slice.call(arguments))
            );
        };
        NOP.prototype = this.prototype;
        bound.prototype = new NOP();
        return bound;
    };
}

// Set polyfill
if (typeof Set === 'undefined' || !isNative(Set)) {
    window.Set = function (items: any) {
        this.set = Object.create(null);
        this.has = function (value: any) { return this.set[value] === true; };
        this.add = function (value: any) { this.set[value] = true; };
        this.clear = function () { this.set = Object.create(null); };
        if (items) {
            for (var i = 0; i < items.length; i++) {
                this.add(items[i]);
            }
        }
    } as any;
}

// Array.isArray polyfill
if (!Array.isArray) {
    Array.isArray = function (arg: any): boolean {
        return Object.prototype.toString.call(arg) === '[object Array]';
    } as any;
}

// Array.prototype.filter polyfill
if (!Array.prototype.filter) {
    Array.prototype.filter = function (callback: any, thisArg: any) {
        if (typeof callback !== 'Function' && typeof callback !== 'function' || !this) {
            throw new TypeError();
        }
        var len = this.length >>> 0;
        var result = new Array(len);
        var arr = this;
        var count = 0;
        var index = -1;

        if (thisArg) {
            while (++index !== len) {
                if (index in this && callback.call(thisArg, arr[index], index, arr)) {
                    result[count++] = arr[index];
                }
            }
        } else {
            while (++index !== len) {
                if (index in arr && callback(arr[index], index, arr)) {
                    result[count++] = arr[index];
                }
            }
        }
        result.length = count;
        return result;
    } as any;
}

// Array.prototype.reduce polyfill
if (!Array.prototype.reduce) {
    Array.prototype.reduce = function (callback: any) {
        if (this === null) {
            throw new TypeError('Array.prototype.reduce called on null or undefined');
        }
        if (typeof callback !== 'function') {
            throw new TypeError(callback + ' is not a function');
        }
        var obj = Object(this);
        var len = obj.length >>> 0;
        var index = 0;
        var value: any;

        if (arguments.length >= 2) {
            value = arguments[1];
        } else {
            while (index < len && !(index in obj)) { index++; }
            if (index >= len) {
                throw new TypeError('Reduce of empty array with no initial value');
            }
            value = obj[index++];
        }
        for (; index < len;) {
            if (index in obj) {
                value = callback(value, obj[index], index, obj);
            }
            index++;
        }
        return value;
    } as any;
}

// Array.prototype.includes polyfill
if (!Array.prototype.includes) {
    Object.defineProperty(Array.prototype, 'includes', {
        value: function (searchElement: any, fromIndex: any) {
            if (this == null) {
                throw new TypeError('"this" is null or not defined');
            }
            var obj = Object(this);
            var len = obj.length >>> 0;
            if (len === 0) return false;

            var start = fromIndex | 0;
            var k = Math.max(start >= 0 ? start : len - Math.abs(start), 0);

            while (k < len) {
                var a = obj[k];
                var b = searchElement;
                if (a === b || (typeof a === 'number' && typeof b === 'number' && isNaN(a) && isNaN(b))) {
                    return true;
                }
                k++;
            }
            return false;
        }
    });
}

// Array.prototype.forEach polyfill
if (!Array.prototype.forEach) {
    Array.prototype.forEach = function (callback: any, thisArg: any) {
        if (typeof callback !== 'function') {
            throw new TypeError(callback + ' is not a function');
        }
        var obj = Object(this);
        var len = obj.length >>> 0;
        var context = arguments.length > 1 ? thisArg : null;
        for (var i = 0; i < len; i++) {
            if (i in obj) {
                callback.call(context, obj[i], i, obj);
            }
        }
    } as any;
}

// ============================================================
// Section 5: Custom _$ DOM Library (jQuery-like)
// ============================================================

// Element.matches polyfill
if (!Element.prototype.matches) {
    Element.prototype.matches =
        Element.prototype.matchesSelector ||
        Element.prototype.mozMatchesSelector ||
        Element.prototype.msMatchesSelector ||
        Element.prototype.oMatchesSelector ||
        Element.prototype.webkitMatchesSelector ||
        function (selector: string) {
            var matches = (this.document || this.ownerDocument).querySelectorAll(selector);
            var i = matches.length;
            while (--i >= 0 && matches.item(i) !== this) {}
            return i > -1;
        } as any;
}

var _$: any;

/**
 * _$ - Lightweight jQuery-like DOM manipulation library
 * @param {string|Node|NodeList|Array} selector - CSS selector, DOM node, or collection
 * @param {Node} context - Context element for querySelector (default: document)
 * @returns {_$.fn.init} Wrapped element collection
 */
_$ = function (selector: any, context: any) {
    return new _$.fn.init(selector, context);
};

_$.fn = _$.prototype;

/**
 * Initialize the _$ collection from a selector or element(s)
 */
_$.fn.init = function (selector: any, context: any) {
    var elements: any = [];

    if (typeof selector === 'string') {
        elements = (context || document).querySelectorAll(selector);
    } else if (selector instanceof Node) {
        elements[0] = selector;
    } else if (selector instanceof NodeList || selector instanceof Array) {
        elements = selector;
    }

    this.length = elements.length;
    for (var i = 0; i < this.length; i += 1) {
        this[i] = elements[i];
    }
    return this;
};

_$.fn.init.prototype = _$.fn;

// --- Collection iteration ---

/**
 * Iterate over each element, optionally returning values
 * @param {function} fn - Callback called with `this` as element
 * @param {boolean} returnValues - If true, return array of results
 */
_$.fn.each = function (fn: any, returnValues?: boolean) {
    var results = [];
    for (var i = 0; i < this.length; i++) {
        results[i] = fn.call(this[i]);
    }
    return returnValues ? (results.length === 1 ? results[0] : results) : this;
};

/**
 * Select elements at specific indices
 */
_$.fn.eq = function () {
    var selected = [];
    for (var i = 0; i < arguments.length; i++) {
        selected[i] = this[arguments[i]];
    }
    return _$(selected);
};

_$.fn.first = function () { return this.eq(0); };
_$.fn.last = function () { return this.eq(this.length - 1); };

// --- Traversal ---

_$.fn.find = function (selector: string) {
    var all = [];
    var result = this.each(function () { return this.querySelectorAll(selector); }, true);

    if (result instanceof Array) {
        for (var i = 0; i < result.length; i++) {
            for (var j = 0; j < result[i].length; j++) {
                all.push(result[i][j]);
            }
        }
    } else {
        all = result;
    }
    return _$(all);
};

_$.fn.parent = function () {
    return _$(this.each(function () { return this.parentNode; }, true));
};

// --- Visibility ---

_$.fn.hide = function () {
    return this.each(function () { this.style.display = 'none'; });
};

_$.fn.show = function () {
    return this.each(function () { this.style.display = ''; });
};

// --- Content ---

_$.fn.text = function (value: any) {
    if (value !== undefined) {
        return this.each(function () { this.innerText = value; });
    }
    return this.each(function () { return this.innerText; }, true);
};

_$.fn.html = function (value: any) {
    if (value !== undefined) {
        return this.each(function () { this.innerHTML = value; });
    }
    return this.each(function () { return this.innerHTML; }, true);
};

_$.fn.outHtml = function (value: any) {
    if (value !== undefined) {
        return this.each(function () { this.outerHTML = value; });
    }
    return this.each(function () { return this.outerHTML; }, true);
};

_$.fn.val = function (value: any) {
    if (value !== undefined) {
        return this.each(function () { this.value = value; });
    }
    return this.each(function () { return this.value; }, true);
};

// --- Styles and Attributes ---

_$.fn.css = function (property: string, value: any, priority: any) {
    if (value !== undefined) {
        return this.each(function () { this.style.setProperty(property, value, priority); });
    }
    return this.each(function () { return this.style.getPropertyValue(property); }, true);
};

_$.fn.attr = function (name: string, value: any) {
    if (value !== undefined) {
        return this.each(function () { this.setAttribute(name, value); });
    }
    return this.each(function () { return this.getAttribute(name); }, true);
};

_$.fn.removeAttr = function (name: string) {
    return this.each(function () { this.removeAttribute(name); });
};

// --- DOM Manipulation ---

_$.fn.remove = function () {
    return this.each(function () { this.remove(); });
};

_$.fn.append = function (html: string) {
    return this.each(function () { this.insertAdjacentHTML('beforeend', html); });
};

_$.fn.prepend = function (html: string) {
    return this.each(function () { this.insertAdjacentHTML('afterbegin', html); });
};

// --- Classes ---

_$.fn.hasClass = function (className: string) {
    return this.each(function () { return this.classList.contains(className); }, true);
};

_$.fn.addClass = function (className: string) {
    return this.each(function () { return this.classList.add(className); });
};

_$.fn.removeClass = function (className: string) {
    return this.each(function () { return this.classList.remove(className); });
};

// --- Events ---

_$.fn.click = function (handler: any) {
    if (typeof handler === 'function') {
        this.each(function () { this.addEventListener('click', handler); });
    } else {
        this.each(function () {
            var event = document.createEvent('HTMLEvents');
            event.initEvent('click', true, true);
            this.dispatchEvent(event);
        });
    }
};

// --- Element Creation ---

_$.fn.tag = function (tagName: string) {
    this[0] = document.createElement(tagName);
    return this;
};

_$.fn.dom = function (htmlString: string) {
    var wrapper = document.createElement('p');
    wrapper.innerHTML = htmlString;
    this[0] = wrapper.childNodes[0];
    return this;
};

// --- Event Binding ---

/**
 * Bind an event listener, optionally with event delegation
 * @param {string} eventType - Event name (e.g., 'click')
 * @param {string} delegateSelector - Optional CSS selector for delegation
 * @param {function} handler - Event handler
 * @returns {function} Unbind function
 */
_$.fn.on = function (eventType: string, delegateSelector: any, handler: any) {
    if (!handler) {
        handler = delegateSelector;
        delegateSelector = null;
    }

    var originalHandler: any;
    if (delegateSelector) {
        originalHandler = handler;
        handler = function (event: any) {
            var target = event.target;
            if (target && target.matches(delegateSelector)) {
                originalHandler.bind(target)(event);
            }
        };
    }

    this.each(function () {
        if (this.addEventListener) {
            this.addEventListener(eventType, handler);
        } else if (this.attachEvent) {
            this.attachEvent('on' + eventType, handler);
        } else {
            this['on' + eventType] = handler;
        }

        // Return unbind function
        return function () {
            if (this.addEventListener) {
                this.removeEventListener(eventType, handler);
            } else if (this.attachEvent) {
                this.detachEvent('on' + eventType, handler);
            } else {
                this['on' + eventType] = null;
            }
        }.bind(this);
    }, true);
};

_$.fn.emit = function (eventType: string) {
    this.each(function () {
        var event = document.createEvent('HTMLEvents');
        event.initEvent(eventType, true, true);
        this.dispatchEvent(event);
    });
};

_$.fn.once = function (eventType: string, handler: any) {
    function onceHandler() {
        if (this.addEventListener) {
            this.removeEventListener(eventType, onceHandler);
        } else if (this.attachEvent) {
            this.detachEvent('on' + eventType, onceHandler);
        } else {
            this['on' + eventType] = null;
        }
        handler.call(this, arguments);
    }

    this.each(function () {
        if (this.addEventListener) {
            this.addEventListener(eventType, onceHandler);
        } else if (this.attachEvent) {
            this.attachEvent('on' + eventType, onceHandler);
        } else {
            this['on' + eventType] = onceHandler;
        }
    }, true);
};

// ============================================================
// Section 6: _$.ajax - AJAX Helper
// ============================================================

_$.ajax = {
    /** Base URL for API requests (e.g., '/reader3') */
    baseURL: '',

    /** Access token for authentication */
    token: '',

    /** Response interceptor */
    onResponse: function (responseText: string) { return responseText; },

    /**
     * Format URL with token and baseURL
     */
    formatURL: function (url: string) {
        if (url.indexOf('?') !== -1) {
            url += '&accessToken=' + this.token;
        } else {
            url += '?accessToken=' + this.token;
        }
        if (url.indexOf('/') === 0) {
            return this.baseURL + url;
        }
        return url;
    },

    /**
     * Make a GET request
     */
    get: function (url: string, callback: any, async?: boolean) {
        if (async === undefined) async = true;
        var self = this;
        var xhr = new XMLHttpRequest();
        xhr.open('GET', this.formatURL(url), async);
        xhr.onreadystatechange = function () {
            if ((xhr.readyState === 4 && xhr.status === 200) || xhr.status === 304) {
                callback.call(this, self.onResponse(xhr.responseText));
            }
        };
        xhr.send();
    },

    /**
     * Make a POST request with JSON body
     */
    post: function (url: string, data: any, callback: any, async?: boolean) {
        if (async === undefined) async = true;
        var self = this;
        var xhr = new XMLHttpRequest();
        xhr.open('POST', this.formatURL(url), async);
        xhr.setRequestHeader('Content-Type', 'application/json');
        xhr.onreadystatechange = function () {
            if (xhr.readyState === 4 && (xhr.status === 200 || xhr.status === 304)) {
                callback.call(this, self.onResponse(xhr.responseText));
            }
        };
        if (typeof data !== 'string') {
            data = JSON.stringify(data);
        }
        xhr.send(data);
    }
};

// ============================================================
// Section 7: URL and Cookie Utilities
// ============================================================

/**
 * Get a URL parameter value from a URL string
 * @param {string} name - Parameter name
 * @param {string} url - URL to parse (defaults to current location)
 */
_$.getUrlPra = function (name: string, url?: string) {
    url = url === undefined ? document.location.toString() : url;
    var parts = url.split('?');
    if (parts.length > 1) {
        var params = parts[1].split('&');
        for (var i = 0; i < params.length; i++) {
            var pair = params[i].split('=');
            if (pair !== null && pair[0] === name) {
                return pair[1];
            }
        }
    }
    return '';
};

/**
 * URL manipulation utilities
 */
_$.url = {
    /**
     * Add a parameter to the current URL (or update if exists)
     */
    add: function (paramName: string, paramValue: string) {
        var url = window.location.href.split('#')[0];
        if (_$.getUrlPra(paramName, url) !== '') {
            this.update(paramName, paramValue);
        } else {
            if (/\?/g.test(url)) {
                if (/name=[-\w]{4,25}/g.test(url)) {
                    url = url.replace(/name=[-\w]{4,25}/g, paramName + '=' + paramValue);
                } else {
                    url += '&' + paramName + '=' + paramValue;
                }
            } else {
                url += '?' + paramName + '=' + paramValue;
            }
            history.pushState(null, null, url);
        }
    },

    /**
     * Update an existing URL parameter value
     */
    update: function (paramName: string, replaceWith: string) {
        var oUrl = window.location.href.toString();
        var re = eval('/(' + paramName + '=)([^&]*)/gi');
        var nUrl = oUrl.replace(re, paramName + '=' + replaceWith);
        this.location = nUrl;
        history.pushState(null, null, nUrl);
    }
};

/**
 * Cookie utilities
 */
_$.cookie = {
    /**
     * Set a cookie
     * @param {string} name - Cookie name
     * @param {string} value - Cookie value
     * @param {number} days - Expiration in days
     * @param {string} path - Cookie path (default: '/')
     */
    set: function (name: string, value: string, days?: number, path?: string) {
        if (path === undefined) path = '/';
        var expires = '';
        if (days) {
            var date = new Date();
            date.setTime(date.getTime() + days * 24 * 3600 * 1000);
            expires = '; expires=' + date.toUTCString();
        }
        document.cookie = name + '=' + encodeURIComponent(value) + expires + ';path=' + path;
    },

    /**
     * Get a cookie value by name
     * @param {string} name - Cookie name
     * @returns {string|null} Cookie value or null
     */
    get: function (name: string) {
        var cookies = document.cookie.replace(/[ ]/g, '').split(';');
        var result: any = null;
        for (var i = 0; i < cookies.length; i++) {
            var pair = cookies[i].split('=');
            if (name === pair[0]) {
                result = decodeURIComponent(pair[1]);
                break;
            }
        }
        return result;
    }
};

// Expose _$ as global $
window.$ = _$;
