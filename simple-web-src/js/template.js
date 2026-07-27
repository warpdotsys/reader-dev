/*!
 * template.js v0.7.1 (https://github.com/yanhaijing/template.js)
 * API https://github.com/yanhaijing/template.js/blob/master/doc/api.md
 * Copyright 2015 yanhaijing. All Rights Reserved
 * Licensed under MIT (https://github.com/yanhaijing/template.js/blob/master/MIT-LICENSE.txt)
 */
;(function(root, factory) {
    var template = factory(root);
    if (typeof define === 'function' && define.amd) {
        // AMD
        define('template', function() {
            return template;
        });
    } else if (typeof exports === 'object') {
        // Node.js
        module.exports = template;
    } else {
        // Browser globals
        var _template = root.template;

        template.noConflict = function() {
            if (root.template === template) {
                root.template = _template;
            }
            return template;
        };
        root.template = template;
    }
}(this, function(root) {
    'use strict';

    // Default configuration
    var o = {
        sTag: '<%',        // Start tag
        eTag: '%>',        // End tag
        compress: false,   // Whether to compress HTML output
        escape: true,      // Whether output is HTML-escaped by default
        error: function (e) {} // Error callback
    };

    var functionMap = {}; // Registered template functions

    // Modifier map - controls output escaping
    var modifierMap = {
        '': function (param) { return nothing(param); },
        'h': function (param) { return encodeHTML(param); },
        'u': function (param) { return encodeURI(param); }
    };

    var toString = {}.toString;
    var slice = [].slice;

    /**
     * Get the type of a value as a lowercase string
     */
    function type(x) {
        if (x === null) {
            return 'null';
        }
        var t = typeof x;
        if (t !== 'object') {
            return t;
        }
        var c = toString.call(x).slice(8, -1).toLowerCase();
        if (c !== 'object') {
            return c;
        }
        if (x.constructor == Object) {
            return c;
        }
        return 'unknown';
    }

    function isObject(obj) { return type(obj) === 'object'; }
    function isFunction(fn) { return type(fn) === 'function'; }
    function isString(str) { return type(str) === 'string'; }

    /**
     * Extend target object with properties from source objects
     */
    function extend() {
        var target = arguments[0] || {};
        var arrs = slice.call(arguments, 1);
        var len = arrs.length;

        for (var i = 0; i < len; i++) {
            var arr = arrs[i];
            for (var name in arr) {
                target[name] = arr[name];
            }
        }
        return target;
    }

    /**
     * Clone objects by extending into a new empty object
     */
    function clone() {
        var args = slice.call(arguments);
        return extend.apply(null, [{}].concat(args));
    }

    function nothing(param) {
        return param;
    }

    /**
     * Encode HTML special characters for safe display
     */
    function encodeHTML(source) {
        return String(source)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/\\/g, '&#92;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#39;');
    }

    /**
     * Compress HTML by removing extra whitespace and comments
     */
    function compress(html) {
        return html.replace(/\s+/g, ' ').replace(/<!--[\w\W]*?-->/g, '');
    }

    /**
     * Adapter for console methods
     */
    function consoleAdapter(cmd, msg) {
        typeof console !== 'undefined' && console[cmd] && console[cmd](msg);
    }

    /**
     * Handle template errors - log and invoke error callback
     */
    function handelError(e) {
        var message = 'template.js error\n\n';

        for (var key in e) {
            message += '<' + key + '>\n' + e[key] + '\n\n';
        }
        message += '<message>\n' + e.message + '\n\n';
        consoleAdapter('error', message);

        o.error(e);

        function error() {
            return 'template.js error';
        }
        error.toString = function () {
            return '__code__ = "template.js error"';
        };
        return error;
    }

    /**
     * Parse template string into executable code
     */
    function parse(tpl, opt) {
        var code = '';
        var sTag = opt.sTag;
        var eTag = opt.eTag;
        var escape = opt.escape;

        // Parse HTML portions (outside template tags)
        function parsehtml(line) {
            line = line.replace(/('|")/g, '\\$1');
            var lineList = line.split('\n');
            var code = '';
            for (var i = 0; i < lineList.length; i++) {
                code += ';__code__ += ("' + lineList[i] +
                    (i === lineList.length - 1 ? '")\n' : '\\n")\n');
            }
            return code;
        }

        // Parse JS expression portions (inside template tags)
        function parsejs(line) {
            var reg = /^(?:=|(:.*?)=)(.*)$/;
            var html;
            var arr;
            var modifier;

            // Handle output expressions: =, :h=, :u=, etc.
            if (arr = reg.exec(line)) {
                html = arr[2];
                if (Boolean(arr[1])) {
                    // Custom modifier (e.g., :h= for HTML escape)
                    modifier = arr[1].slice(1);
                } else {
                    // Default: escape if configured
                    modifier = escape ? 'h' : '';
                }
                return ';__code__ += __modifierMap__["' + modifier +
                    '"](typeof (' + html + ') !== "undefined" ? (' + html + ') : "")\n';
            }

            // Raw JavaScript statement
            return ';' + line + '\n';
        }

        var tokens = tpl.split(sTag);

        for (var i = 0, len = tokens.length; i < len; i++) {
            var token = tokens[i].split(eTag);

            if (token.length === 1) {
                code += parsehtml(token[0]);
            } else {
                code += parsejs(token[0], true);
                if (token[1]) {
                    code += parsehtml(token[1]);
                }
            }
        }
        return code;
    }

    /**
     * Compile template source into a render function
     */
    function compiler(tpl, opt) {
        var mainCode = parse(tpl, opt);

        var headerCode = '\n' +
            '    var html = (function (__data__, __modifierMap__) {\n' +
            '        var __str__ = "", __code__ = "";\n' +
            '        for(var key in __data__) {\n' +
            '            __str__+=("var " + key + "=__data__[\'" + key + "\'];");\n' +
            '        }\n' +
            '        eval(__str__);\n\n';

        var footerCode = '\n' +
            '        ;return __code__;\n' +
            '    }(__data__, __modifierMap__));\n' +
            '    return html;\n';

        var code = headerCode + mainCode + footerCode;
        code = code.replace(/[\r]/g, ' '); // IE 7/8 compatibility

        try {
            var Render = new Function('__data__', '__modifierMap__', code);
            Render.toString = function () {
                return mainCode;
            };
            return Render;
        } catch(e) {
            e.temp = 'function anonymous(__data__, __modifierMap__) {' + code + '}';
            throw e;
        }
    }

    /**
     * Compile a template string, returning a render function
     */
    function compile(tpl, opt) {
        opt = clone(o, opt);

        try {
            var Render = compiler(tpl, opt);
        } catch(e) {
            e.name = 'CompileError';
            e.tpl = tpl;
            e.render = e.temp;
            delete e.temp;
            return handelError(e);
        }

        function render(data) {
            data = clone(functionMap, data);
            try {
                var html = Render(data, modifierMap);
                html = opt.compress ? compress(html) : html;
                return html;
            } catch(e) {
                e.name = 'RenderError';
                e.tpl = tpl;
                e.render = Render.toString();
                return handelError(e)();
            }
        }

        render.toString = function () {
            return Render.toString();
        };
        return render;
    }

    /**
     * Main template function - compile and optionally render
     * @param {string} tpl - Template string
     * @param {object} data - Optional data to render immediately
     * @returns {function|string} Render function or rendered HTML
     */
    function template(tpl, data) {
        if (typeof tpl !== 'string') {
            return '';
        }

        var fn = compile(tpl);
        if (!isObject(data)) {
            return fn;
        }

        return fn(data);
    }

    /**
     * Generate HTML from a pre-compiled render function
     */
    template.generate = function (render, data, opt) {
        opt = clone(o, opt);
        data = clone(functionMap, data);
        try {
            var html = render(data, modifierMap);
            html = opt.compress ? compress(html) : html;
            return html;
        } catch(e) {
            e.name = 'RenderError';
            e.render = render.toString();
            return handelError(e)();
        }
    };

    /**
     * Update template configuration
     */
    template.config = function (option) {
        if (isObject(option)) {
            o = extend(o, option);
        }
        return clone(o);
    };

    /**
     * Register a helper function available in templates
     */
    template.registerFunction = function(name, fn) {
        if (!isString(name)) {
            return clone(functionMap);
        }
        if (!isFunction(fn)) {
            return functionMap[name];
        }
        return functionMap[name] = fn;
    };

    template.unregisterFunction = function (name) {
        if (!isString(name)) {
            return false;
        }
        delete functionMap[name];
        return true;
    };

    /**
     * Register a custom output modifier
     */
    template.registerModifier = function(name, fn) {
        if (!isString(name)) {
            return clone(modifierMap);
        }
        if (!isFunction(fn)) {
            return modifierMap[name];
        }
        return modifierMap[name] = fn;
    };

    template.unregisterModifier = function (name) {
        if (!isString(name)) {
            return false;
        }
        delete modifierMap[name];
        return true;
    };

    template.__encodeHTML = encodeHTML;
    template.__compress = compress;
    template.__handelError = handelError;
    template.__compile = compile;
    template.version = '0.7.1';

    return template;
}));
