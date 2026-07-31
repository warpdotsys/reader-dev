function _typeof(t){return(_typeof="function"==typeof Symbol&&"symbol"==typeof Symbol.iterator?function(t){return typeof t}:function(t){return t&&"function"==typeof Symbol&&t.constructor===Symbol&&t!==Symbol.prototype?"symbol":typeof t})(t)}function loadScript(t,e,n){var o=document.createElement("script");n&&(o.async="async"),o.src=t,e&&(o.onload=e),document.head.appendChild(o)}function loadLink(t,e,n,o){var r=document.createElement("link");r.rel=n||"stylesheet",r.type=o||"text/css",r.href=t,e&&(r.onload=e),document.getElementsByTagName("head")[0].appendChild(r)}function getQueryString(t,e){return decodeURIComponent(window.location.search.replace(new RegExp("^(?:.*"+(e?"?":"")+"[&\\?]"+encodeURI(t).replace(/[\.\+\*]/g,"\\$&")+"(?:\\=([^&]*))?)?.*$","i"),"$1"))}var fakeStorage={items:{},isFake:!0,getItem:function(t){return this.items[t]},setItem:function(t,e){this.items[t]=e},clear:function(){this.items={}},removeItem:function(t){delete this.items[t]}},errorMsg=(window.myStorage=window.localStorage||fakeStorage,window.isDebug=window.getQueryString("debug"),window.navigator.userAgent+"\n");function isNative(t){return null!==t&&/native code/.test(t.toString())}window.onerror=function(t,e,n,o,r){t=["Type:"+_typeof(t),"Message: "+t,"URL: "+e,"Line: "+n,"Column: "+o,"Error object: "+JSON.stringify(r)].join("  ");return errorMsg+=t+"\n",console.error(t),showErrorMsg&&showErrorMsg(t)||alert(t),!1},window.isDebug&&window.loadScript("https://cdn.bootcdn.net/ajax/libs/vConsole/3.9.1/vconsole.min.js",function(){new VConsole},!0),window.isKindle=0<=window.navigator.userAgent.toLowerCase().indexOf("kindle"),Function.prototype.bind||(Function.prototype.bind=function(t){if("function"!=typeof this)throw new TypeError("Function.prototype.bind - what is trying to be bound is not callable");function e(){return o.apply(this instanceof r?this:t,n.concat(Array.prototype.slice.call(arguments)))}var n=Array.prototype.slice.call(arguments,1),o=this,r=function(){};return r.prototype=this.prototype,e.prototype=new r,e}),"undefined"!=typeof Set&&isNative(Set)||(window.Set=function(t){if(this.set=Object.create(null),this.has=function(t){return!0===this.set[t]},this.add=function(t){this.set[t]=!0},this.clear=function(){this.set=Object.create(null)},t)for(var e=0;e<t.length;e++)this.add(t[e])}),Array.isArray||(Array.isArray=function(t){return"[object Array]"===Object.prototype.toString.call(t)}),Array.prototype.filter||(Array.prototype.filter=function(t,e){if("Function"!=typeof t&&"function"!=typeof t||!this)throw new TypeError;var n=this.length>>>0,o=new Array(n),r=this,i=0,a=-1;if(e)for(;++a!=n;)a in this&&t.call(e,r[a],a,r)&&(o[i++]=r[a]);else for(;++a!=n;)a in r&&t(r[a],a,r)&&(o[i++]=r[a]);return o.length=i,o}),Array.prototype.reduce||(Array.prototype.reduce=function(t){if(null===this)throw new TypeError("Array.prototype.reduce called on null or undefined");if("function"!=typeof t)throw new TypeError(t+" is not a function");var e,n=Object(this),o=n.length>>>0,r=0;if(2<=arguments.length)e=arguments[1];else{for(;r<o&&!(r in n);)r++;if(o<=r)throw new TypeError("Reduce of empty array with no initial value");e=n[r++]}for(;r<o;)r in n&&(e=t(e,n[r],r,n)),r++;return e}),Array.prototype.includes||Object.defineProperty(Array.prototype,"includes",{value:function(t,e){if(null==this)throw new TypeError('"this" is null or not defined');var n=Object(this),o=n.length>>>0;if(0!=o)for(var r,i,e=0|e,a=Math.max(0<=e?e:o-Math.abs(e),0);a<o;){if((r=n[a])===(i=t)||"number"==typeof r&&"number"==typeof i&&isNaN(r)&&isNaN(i))return!0;a++}return!1}}),Array.prototype.forEach||(Array.prototype.forEach=function(t,e){if("function"!=typeof t)throw new TypeError(t+" is not a function");for(var n=Object(this),o=n.length>>>0,r=1<arguments.length?e:null,i=0;i<o;i++)i in n&&t.call(r,n[i],i,n)});
var _$;Element.prototype.matches||(Element.prototype.matches=Element.prototype.matchesSelector||Element.prototype.mozMatchesSelector||Element.prototype.msMatchesSelector||Element.prototype.oMatchesSelector||Element.prototype.webkitMatchesSelector||function(t){for(var n=(this.document||this.ownerDocument).querySelectorAll(t),e=n.length;0<=--e&&n.item(e)!==this;);return-1<e}),_$=function(t,n){return new _$.fn.init(t,n)},_$.fn=_$.prototype,_$.fn.init=function(t,n){var e=[];"string"==typeof t?e=(n||document).querySelectorAll(t):t instanceof Node?e[0]=t:(t instanceof NodeList||t instanceof Array)&&(e=t),this.length=e.length;for(var i=0;i<this.length;i+=1)this[i]=e[i];return this},_$.fn.init.prototype=_$.fn,_$.fn.each=function(t,n){for(var e=[],i=0;i<this.length;i++)e[i]=t.call(this[i]);return n?1===e.length?e[0]:e:this},_$.fn.eq=function(){for(var t=[],n=0;n<arguments.length;n++)t[n]=this[arguments[n]];return _$(t)},_$.fn.first=function(){return this.eq(0)},_$.fn.last=function(){return this.eq(this.length-1)},_$.fn.find=function(t){var n=[],e=this.each(function(){return this.querySelectorAll(t)},1);if(e instanceof Array)for(var i=0;i<e.length;i++)for(var r=0;r<e[i].length;r++)n.push(e[i][r]);else n=e;return _$(n)},_$.fn.parent=function(){return _$(this.each(function(){return this.parentNode},1))},_$.fn.hide=function(){return this.each(function(){this.style.display="none"})},_$.fn.show=function(){return this.each(function(){this.style.display=""})},_$.fn.text=function(t){return void 0!==t?this.each(function(){this.innerText=t}):this.each(function(){return this.innerText},1)},_$.fn.html=function(t){return void 0!==t?this.each(function(){this.innerHTML=t}):this.each(function(){return this.innerHTML},1)},_$.fn.outHtml=function(t){return void 0!==t?this.each(function(){this.outerHTML=t}):this.each(function(){return this.outerHTML},1)},_$.fn.val=function(t){return void 0!==t?this.each(function(){this.value=t}):this.each(function(){return this.value},1)},_$.fn.css=function(t,n,e){return void 0!==n?this.each(function(){this.style.setProperty(t,n,e)}):this.each(function(){return this.style.getPropertyValue(t)},1)},_$.fn.attr=function(t,n){return void 0!==n?this.each(function(){this.setAttribute(t,n)}):this.each(function(){return this.getAttribute(t)},1)},_$.fn.removeAttr=function(t){return this.each(function(){this.removeAttribute(t)})},_$.fn.remove=function(){return this.each(function(){this.remove()})},_$.fn.append=function(t){return this.each(function(){this.insertAdjacentHTML("beforeend",t)})},_$.fn.prepend=function(t){return this.each(function(){this.insertAdjacentHTML("afterbegin",t)})},_$.fn.hasClass=function(t){return this.each(function(){return this.classList.contains(t)},1)},_$.fn.addClass=function(t){return this.each(function(){return this.classList.add(t)})},_$.fn.removeClass=function(t){return this.each(function(){return this.classList.remove(t)})},_$.fn.click=function(t){"function"==typeof t?this.each(function(){this.addEventListener("click",t)}):this.each(function(){var t=document.createEvent("HTMLEvents");t.initEvent("click",!0,!0),this.dispatchEvent(t)})},_$.fn.tag=function(t){return this[0]=document.createElement(t),this},_$.fn.dom=function(t){var n=document.createElement("p");return n.innerHTML=t,this[0]=n.childNodes[0],this},_$.fn.on=function(t,e,n){var i;n||(n=e,e=null),e&&(i=n,n=function(t){var n=t.target;n&&n.matches(e)&&i.bind(n)(t)}),this.each(function(){return this.addEventListener?this.addEventListener(t,n):this.attachEvent?this.attachEvent("on"+t,n):this["on"+t]=n,function(){this.addEventListener?this.removeEventListener(t,n):this.attachEvent?this.detachEvent("on"+t,n):this["on"+t]=null}.bind(this)},1)},_$.fn.emit=function(t){this.each(function(){var t=document.createEvent("HTMLEvents");t.initEvent(t,!0,!0),this.dispatchEvent(t)})},_$.fn.once=function(t,n){function e(){this.addEventListener?this.removeEventListener(t,e):this.attachEvent?this.detachEvent("on"+t,e):this["on"+t]=null,n.call(this,arguments)}this.each(function(){this.addEventListener?this.addEventListener(t,e):this.attachEvent?this.attachEvent("on"+t,e):this["on"+t]=e},1)},_$.ajax={baseURL:"",token:"",onResponse:function(t){return t},formatURL:function(t){return-1!==t.indexOf("?")?t+="&accessToken="+this.token:t+="?accessToken="+this.token,0===t.indexOf("/")?this.baseURL+t:t},get:function(t,n,e){void 0===e&&(e=!0);var i=this,r=new XMLHttpRequest;r.open("GET",this.formatURL(t),e),r.onreadystatechange=function(){(4===r.readyState&&200===r.status||304===r.status)&&n.call(this,i.onResponse(r.responseText))},r.send()},post:function(t,n,e,i){void 0===i&&(i=!0);var r=this,o=new XMLHttpRequest;o.open("POST",this.formatURL(t),i),o.setRequestHeader("Content-Type","application/json"),o.onreadystatechange=function(){4!==o.readyState||200!==o.status&&304!==o.status||e.call(this,r.onResponse(o.responseText))},"string"!=typeof n&&(n=JSON.stringify(n)),o.send(n)}},_$.getUrlPra=function(t,n){n=void 0===n?document.location.toString():n,n=n.split("?");if(1<n.length)for(var e,i=n[1].split("&"),r=0;r<i.length;r++)if(null!==(e=i[r].split("="))&&e[0]===t)return e[1];return""},_$.url={add:function(t,n){var e=window.location.href.split("#")[0];""!==_$.getUrlPra(t,e)?this.update(t,n):(/\?/g.test(e)?/name=[-\w]{4,25}/g.test(e)?e=e.replace(/name=[-\w]{4,25}/g,t+"="+n):e+="&"+t+"="+n:e+="?"+t+"="+n,history.pushState(null,null,e))},update:function update(paramName,replaceWith){var oUrl=window.location.href.toString(),re=eval("/("+paramName+"=)([^&]*)/gi"),nUrl=oUrl.replace(re,paramName+"="+replaceWith);this.location=nUrl,history.pushState(null,null,nUrl)}},_$.cookie={set:function(t,n,e,i){void 0===i&&(i="/");var r,o="";e&&((r=new Date).setTime(r.getTime()+24*e*3600*1e3),o="; expires="+r.toUTCString()),document.cookie=t+"="+encodeURIComponent(n)+o+";path="+i},get:function(t){for(var n=document.cookie.replace(/[ ]/g,"").split(";"),e=null,i=0;i<n.length;i++){var r=n[i].split("=");if(t===r[0]){e=decodeURIComponent(r[1]);break}}return e}},window.$=_$;
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
    var o = {
        sTag: '<%',//开始标签
        eTag: '%>',//结束标签
        compress: false,//是否压缩html
        escape: true, //默认输出是否进行HTML转义
        error: function (e) {}//错误回调
    };
    var functionMap = {}; //内部函数对象
    //修饰器前缀
    var modifierMap = {
        '': function (param) {return nothing(param)},
        'h': function (param) {return encodeHTML(param)},
        'u': function (param) {return encodeURI(param)}
    };

    var toString = {}.toString;
    var slice = [].slice;
    function type(x) {
        if(x === null){
            return 'null';
        }

        var t= typeof x;

        if(t !== 'object'){
            return t;
        }

        var c = toString.call(x).slice(8, -1).toLowerCase();
        if(c !== 'object'){
            return c;
        }

        if(x.constructor==Object){
            return c;
        }

        return 'unknown';
    }

    function isObject(obj) {
        return type(obj) === 'object';
    }
    function isFunction(fn) {
        return type(fn) === 'function';
    }
    function isString(str) {
        return type(str) === 'string';
    }
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
    function clone() {
        var args = slice.call(arguments);
        return extend.apply(null, [{}].concat(args));
    }
    function nothing(param) {
        return param;
    }
    function encodeHTML(source) {
        return String(source)
            .replace(/&/g,'&amp;')
            .replace(/</g,'&lt;')
            .replace(/>/g,'&gt;')
            .replace(/\\/g,'&#92;')
            .replace(/"/g,'&quot;')
            .replace(/'/g,'&#39;');
    }
    function compress(html) {
        return html.replace(/\s+/g, ' ').replace(/<!--[\w\W]*?-->/g, '');
    }
    function consoleAdapter(cmd, msg) {
        typeof console !== 'undefined' && console[cmd] && console[cmd](msg);
    }
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
        }
        return error;
    }
    function parse(tpl, opt) {
        var code = '';
        var sTag = opt.sTag;
        var eTag = opt.eTag;
        var escape = opt.escape;
        function parsehtml(line) {
            // 单双引号转义，换行符替换为空格
            line = line.replace(/('|")/g, '\\$1');
            var lineList = line.split('\n');
            var code = '';
            for (var i = 0; i < lineList.length; i++) {
                code += ';__code__ += ("' + lineList[i] + (i === lineList.length - 1 ? '")\n' : '\\n")\n');
            }
            return code;
        }
        function parsejs(line) {
            //var reg = /^(:?)(.*?)=(.*)$/;
            var reg = /^(?:=|(:.*?)=)(.*)$/
            var html;
            var arr;
            var modifier;

            // = := :*=
            // :h=123 [':h=123', 'h', '123']
            if (arr = reg.exec(line)) {
                html = arr[2]; // 输出
                if (Boolean(arr[1])) {
                    // :开头
                    modifier = arr[1].slice(1);
                } else {
                    // = 开头
                    modifier = escape ? 'h' : '';
                }

                return ';__code__ += __modifierMap__["' + modifier + '"](typeof (' + html + ') !== "undefined" ? (' + html + ') : "")\n';
            }

            //原生js
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
        code = code.replace(/[\r]/g, ' '); // ie 7 8 会报错，不知道为什么
        try {
            var Render = new Function('__data__', '__modifierMap__', code);
            Render.toString = function () {
                return mainCode;
            }
            return Render;
        } catch(e) {
            e.temp = 'function anonymous(__data__, __modifierMap__) {' + code + '}';
            throw e;
        }
    }
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
    }

    template.config = function (option) {
        if (isObject(option)) {
            o = extend(o, option);
        }
        return clone(o);
    };

    template.registerFunction = function(name, fn) {
        if (!isString(name)) {
            return clone(functionMap);
        }
        if (!isFunction(fn)) {
            return functionMap[name];
        }

        return functionMap[name] = fn;
    }
    template.unregisterFunction = function (name) {
        if (!isString(name)) {
            return false;
        }
        delete functionMap[name];
        return true;
    }

    template.registerModifier = function(name, fn) {
        if (!isString(name)) {
            return clone(modifierMap);
        }
        if (!isFunction(fn)) {
            return modifierMap[name];
        }

        return modifierMap[name] = fn;
    }
    template.unregisterModifier = function (name) {
        if (!isString(name)) {
            return false;
        }
        delete modifierMap[name];
        return true;
    }

    template.__encodeHTML = encodeHTML;
    template.__compress = compress;
    template.__handelError = handelError;
    template.__compile = compile;
    template.version = '0.7.1';
    return template;
}));
function _typeof(t){return(_typeof="function"==typeof Symbol&&"symbol"==typeof Symbol.iterator?function(t){return typeof t}:function(t){return t&&"function"==typeof Symbol&&t.constructor===Symbol&&t!==Symbol.prototype?"symbol":typeof t})(t)}function showTip(t,e,o,n){$("#tipName").html(t),$("#tipClose").html(e),$("#tipBody").html(o),$("#tipBox").css("display","block");t=$("#tipBox").attr("data-lastclass");t&&$("#tipBox").removeClass(t),n&&($("#tipBox").addClass(n),$("#tipBox").attr("data-lastclass",n)),$("#dropdown-backdrop").attr("class","dropdown-backdrop")}function closeTip(){$("#dropdown-backdrop").attr("class",""),$("#tipBox").css("display","none");var t=$("#tipBox").attr("data-lastclass");t&&$("#tipBox").removeClass(t)}function isHtml(t){return/<[a-z]+\d?(\s+[\w-]+=("[^"]*"|'[^']*'))*\s*\/?>|&#?\w+;/i.test(t)}$("#tipClose").click(function(){closeTip()}),$("#dropdown-backdrop").click(function(t){closeTip()});var Title=function(){this.title=document.title,this.set=function(t){document.title=t},this.reset=function(){document.title=this.title},this.update=function(t){document.title=this.title+" - "+t}},token=$.getUrlPra("accessToken"),api=(token?$.cookie.set("token",token,30,"/"):token=$.cookie.get("token")||window.myStorage.getItem("api_token"),$.ajax.token=token||"",$.getUrlPra("api"));function requireTmpl(t,e){window.__TMPL__=window.__TMPL__||{},window.__TMPL__[t]?e(window.__TMPL__[t]):$.ajax.get("./assets/template/"+t+".tmpl",function(t){e(t)},!0)}function renderTmpl(t,e,o){requireTmpl(t,function(t){t="string"==typeof t?template(t,e):template.generate(t,e);o(t)})}function Watcher(t){var o=[],n=t,i=null;this.onChange=function(t,e){t&&(o.push(t),e)&&t(n,i)},this.trigger=function(){for(var t=0;t<o.length;t++)o[t](n,i)},this.update=function(t){i=n,(n=t)!==i&&this.trigger()},this.getValue=function(){return n}}function setBookSource(t,e){var o,n=sourceListWatcher.getValue();n&&n[e]&&(o=n[e],$.ajax.post("/setBookSource",{bookUrl:t,newUrl:o.bookUrl,bookSourceUrl:o.origin},function(t){if((t=JSON.parse(t)).isSuccess)try{location.replace("./reader.html?bookUrl="+encodeURIComponent(o.bookUrl))}catch(t){location.href="./reader.html?bookUrl="+encodeURIComponent(o.bookUrl)}else alert(t.errorMsg)},!0))}api?$.cookie.set("api",api,30,"/"):api=$.cookie.get("api"),$.ajax.baseURL=api||window.myStorage.getItem("api_prefix")||"/reader3",$.ajax.onResponse=function(t){try{var e=JSON.parse(t);e&&"NEED_LOGIN"===e.data&&isLoginWatcher.update(!1)}catch(t){}return t};var lastSourceIndex=0;function loadMoreBookSource(t){var e=sourceListWatcher.getValue();"object"===_typeof(e)&&(lastSourceIndex=lastSourceIndex||e.length),$.ajax.post("/searchBookSource",{url:t,lastIndex:lastSourceIndex},function(t){try{var e,o,n,i;(t=JSON.parse(t)).isSuccess&&(e=t.data.list||[],o=sourceListWatcher.getValue(),o=Array.isArray(o)?o:[],n={},o.map(function(t){n[t.bookUrl]=1}),i=[].concat(o,e.filter(function(t){return!n[t.bookUrl]})),t.data.lastIndex&&(lastSourceIndex=t.data.lastIndex),sourceListWatcher.update(i))}catch(t){sourceListWatcher.update("error")}},!0)}function showLoginBox(){showTip("登录","关闭","<div class='login-form'>\n    <form action='/login' onsubmit='return doLogin(this, true);'>\n       <label>用户名：<input type='text' name='username' /></label>\n       <label>密　码：<input type='text' name='password' /></label>\n       <label>邀请码：<input type='text' name='code' /></label>\n       <button type='submit' onclick='doLogin(this.parentNode, false)'>注册</button>\n       <button type='submit' onclick='doLogin(this.parentNode, true)'>登录</button>\n   </form>\n</div>","top-tip")}function showSettingBox(){var t=(t=window.location.pathname.split("/"))[t.length-1].replace(".html","").toLowerCase(),t=$.cookie.get(t+"bj"),e=[0,0,0,0,window.innerHeight,0];t&&((e=t.split(",")).length<5&&e.push(window.innerHeight),e.length<6)&&e.push(0),showTip("设置","关闭","<div class='login-form'>\n    <form action='' onsubmit='return saveSetting(this);'>\n       <label>上边界：<input type='text' name='top' value='"+e[0]+"' /></label>\n       <label>下边界：<input type='text' name='bottom' value='"+e[1]+"' /></label>\n       <label>左边界：<input type='text' name='left' value='"+e[2]+"' /></label>\n       <label>右边界：<input type='text' name='right' value='"+e[3]+"' /></label>\n       <label>最大高度：<input type='text' name='maxHeight' value='"+e[4]+"' /></label>\n       <input id='hideSBInput' type='hidden' name='hideSB' value='"+e[5]+"' />\n       <label>隐藏滚动条：<span style='border: 1px solid #ddd;display: inline-block; padding: 3px 10px;' onclick='toggleScrollBarHidden(this)'>"+(1==e[5]?"已开启":"已关闭")+"</span></label>\n       <button type='submit'>保存</button>\n   </form>\n</div>","top-tip")}function clearStorage(){window.myStorage&&window.myStorage.clear&&window.myStorage.clear()}function showErrorMsg(){showTip("温馨提示","关闭","<pre class='notice-info'>\n"+errorMsg+"</pre>")}function showNotice(t){showTip("温馨提示","关闭","<pre class='notice-info'>\n"+t+"</pre>")}function submitForm(t,e,o){for(var n={},i=t.querySelectorAll("[name]"),a=0;a<i.length;a++){var r=i[a];n[r.getAttribute("name")]=r.value}(n=e?e(n):n)&&$.ajax.post(t.getAttribute("action"),n,function(t){try{t=JSON.parse(t),o&&o(t)}catch(t){}},!1)}function isInteger(t){return Math.floor(t)===t}function saveSetting(t){return submitForm(t,function(t){var e;return isInteger(parseInt(t.top))&&isInteger(parseInt(t.bottom))&&isInteger(parseInt(t.left))&&isInteger(parseInt(t.right))?isInteger(parseInt(t.maxHeight))?parseInt(t.maxHeight)<window.innerHeight/2?showNotice("最大高度不能小于窗口的一半"):(e=(e=window.location.pathname.split("/"))[e.length-1].replace(".html","").toLowerCase(),t=parseInt(t.top)+","+parseInt(t.bottom)+","+parseInt(t.left)+","+parseInt(t.right)+","+t.maxHeight+","+t.hideSB,$.cookie.set(e+"bj",t,0),updatePagePosition()):showNotice("最大高度只能是数字"):showNotice("边界值只能是数字"),!1},!1),!1}function toggleScrollBarHidden(t){var e=$("#hideSBInput").val();1===parseInt(e)?($("#hideSBInput").val("0"),t.innerText="已关闭"):($("#hideSBInput").val("1"),t.innerText="已开启")}function updatePagePosition(){var t=(t=window.location.pathname.split("/"))[t.length-1].replace(".html","").toLowerCase(),t=$.cookie.get(t+"bj"),e=$("#pageBox");t&&(t=t.split(","),e.css("top",(0|t[0])+"px"),e.css("bottom",(0|t[1])+"px"),e.css("left",(0|t[2])+"px"),e.css("right",(0|t[3])+"px"),6<=t.length&&(0|t[5]?(e=(e=0|t[4])>window.innerHeight/2?e:window.innerHeight,$("body").css("max-height",e+"px"),$("body").css("overflow","hidden"),$("html").css("max-height",e+"px"),$("html").css("overflow","hidden")):($("body").css("max-height")&&$("body").css("max-height","none"),$("body").css("overflow","auto"),$("html").css("max-height")&&$("html").css("max-height","none"),$("html").css("overflow","auto"))),window.reader)&&window.reader.viewDisplay&&window.reader.viewDisplay()}function doLogin(t,e){return submitForm(t,function(t){return!(!t.username||!t.password)&&(t.isLogin=e,t)},function(t){t.isSuccess&&(closeTip(),t.data)&&t.data.accessToken&&(token=t.data.accessToken,$.cookie.set("token",token,30,"/"),$.ajax.token=token,isLoginWatcher.update(!0))}),!1}function doLogout(){$.ajax.post("/logout","",function(t){try{(t=JSON.parse(t))&&t.isSuccess&&(token="",$.cookie.set("token",token,-1,"/"),$.ajax.token=token,isLoginWatcher.update(!1))}catch(t){}},!1)}var isLoginWatcher=new Watcher(!0),isBlack=(isLoginWatcher.onChange(function(t){t||showLoginBox()}),window.darkThemeWatcher=new Watcher(!1),$.cookie.get("black"));function resolveUrl(){var t=arguments.length;if(0===t)throw new Error("resolveUrl requires at least one argument; got none.");var e=document.createElement("base");if(e.href=arguments[0],1===t)return e.href;for(var o,n=document.getElementsByTagName("head")[0],i=(n.insertBefore(e,n.firstChild),document.createElement("a")),a=1;a<t;a++)i.href=arguments[a],o=i.href,e.href=o;return n.removeChild(e),o}function debounce(o,n){var i;return function(){var t=this,e=(clearTimeout(i),arguments);i=setTimeout(function(){o.apply(t,e)},n)}}function chooseMenu(t,e){$(".choose").removeClass("choose"),$(t).addClass("choose"),$("#"+e).addClass("choose")}function loadUserInfo(){$.ajax.get("/getUserInfo",function(t){(t=JSON.parse(t)).isSuccess&&console.log(t)},!0)}function onMounted(t){t&&window.__onMountedHook.push(t)}isBlack&&"true"===isBlack.toString()?(window.darkThemeWatcher.update(!0),$("html").attr("class","dark-theme")):$("html").attr("class","white-theme"),window.__onMountedHook=[loadUserInfo],window.onload=function(){updatePagePosition(),window.__onMountedHook.forEach(function(t){return t()})};
var bookListWatcher=new Watcher([]),BookApi=function(){this.updateBook=function(t){for(var e=bookListWatcher.getValue(),o=0;o<e.length;o++)if(t.bookUrl===e[o].bookUrl){e[o]=t;break}bookListWatcher.update(e)},this.getBookshelf=function(o){$.ajax.get("/getBookshelf",function(t){if((t=JSON.parse(t)).isSuccess){for(var e=0;e<t.data.length;e++)t.data[e].id=e,t.data[e].title=t.data[e].name;bookListWatcher.update(t.data),o&&o()}},!0)},this.getBookInfoByUrl=function(t){for(var e=bookListWatcher.getValue(),o=0;o<e.length;o++)if(t===e[o].bookUrl)return e[o];return null},this.find=function(t){return 0<=t&&t<this.book.length?t:-1},this.isFavourite=function(t){return-1===this.find(t)?"收藏本书":"取消收藏"},this.favourite=function(t){return-1===this.find(t)?(this.insert(),!0):(this.delete(t),!1)},this.getData=function(t){t=this.find(t);return-1===t?{title:"",id:"",readCount:0,totalCount:1,index:0,readChapter:"",totalChapter:"",siteName:"",author:"",page:0}:this.book[t]}};
var Pagination=function(){this.list=[],this.page=1,this.pageCount=1,this.next=$("#next"),this.before=$("#before"),this.pageIndexList=[],this.containerHeight=0,this.onPageChange=null,this.init=function(t,i,s,e,h){this.list=$(t),this.list&&this.list.length&&(this.onPageChange=e,s=s||".right_t.flexone",this.containerHeight=$(s)[0].offsetHeight-i,this.computePage(),this.page=h||1,this.display(this.page),this.pageCount<=1)&&(this.set("before","beforeN"),this.set("next","nextN"))},this.computePage=function(){var t=0;this.pageIndexList=[0];for(var i=0;i<this.list.length;i++)t+this.list[i].offsetHeight>=this.containerHeight?(this.pageIndexList.push(i),t=this.list[i].offsetHeight):t+=this.list[i].offsetHeight;0<t&&this.pageIndexList.push(i),this.pageCount=this.pageIndexList.length-1,console.log(this.pageIndexList)},this.set=function(t,i){$("#"+t+"_img").removeClass(t+"Y").removeClass(t+"N").addClass(i)},this.beforeClick=function(){this.display(this.page-1)},this.nextClick=function(){this.display(this.page+1)},this.display=function(t){if(!(t<1||t>this.pageCount)){this.onPageChange&&this.onPageChange(t),console.log("display",t),this.page=t,this.list.css("display","none");for(var t=this.pageIndexList[this.page-1],i=this.pageIndexList[this.page],s=t;s<i;s++)$(this.list[s]).css("display","block");this.page===this.pageCount?this.set("next","nextN"):this.set("next","nextY"),1===this.page?this.set("before","beforeN"):this.set("before","beforeY")}}};
var Menu=function(){function i(){var e=$.cookie.get("black");return e=!(!e||"true"!==e.toString())}this.generateMenu=function(e){console.log("isLogin",e);for(var t=(t=window.location.pathname.split("/"))[t.length-1].replace(".html","").toLowerCase(),a=[{key:"小说书架",url:"index"},{key:"RSS阅读",url:"rss"},{key:"小说搜索",url:"search"},{key:"页面设置",url:"",action:"showSettingBox"},{key:"清除缓存",url:"",action:"clearStorage"},{key:e?"退出登录":"登录",url:"",action:e?"doLogout":"showLoginBox"}],n=i()?'<img alt="" src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAAAXNSR0IArs4c6QAAA39JREFUeF7tWu111DAQ3KkASggVEDoIHYQKIBUkVABUAKmAUAFQAVABSQfQQVLB8uY9mef4JGtXJ1v2+fQnPyJLmtHspw6y8YGN45cjAUcFbJyBowlsQQCq+k5EvgD4M8R70ApQ1VMR+Swi/EsC3myGAFW9FJFPA8DPhio4OAWo6tNw6+cR895RwUERECT/VUROEr7tAQAJ+j8OhgBVpX3T3lPjQUTOANweHAGqSuA7Dq4HNAqe/1+1AoK9U/Jn3pvv5q+WgEGIc8l+9SYQwP8QkUcOLcLCBYCbsWRvdQowOLsObxb86nyAA/xbAMMkKCqE1SjA4Ok7gNGUN2UGqyDAAf47gFgGmHQDiyfAAf4uJDr3ngp30QQ4wDPROQHgAr9YJ2hMcLqLTmZ5FiUsUgGOmyfGVwC+WcDG5iyOACf4DwDel4JfnAk4wbvC3eLDoBN8kcdfrAk4wdPpncYanCWm0NwHOMET40sAP0vALk4BqnolIh8dYPZ2esO9sgoIMZmtpOJQEwPoKGy6z91proXYUQICeNbd7KubykvLpqrKfJ2dHOv4G+zenenlNkgSMADfrXMD4CK36Nj/Hc2M/jIvhs3Mfc7Q/zZKQAJ89x1NgWpw34aqsl3929DJ6Z/RXNuXkLJDQAZ8twdby0xBd97aUocwrjv8fBK7H1WAqvKGaPO5QQUwJD3qs48QQJv31OqT2X2OgNwDQ/97kkCJjjYeVZXtKb7VecZkdm/xAR4SuN4VgOsYuoJwx2Wqx/sU82NRgLf62nFlOxGi0OP/AjD20OE4Un5qLg/wkkB/QL9wH5we/UnqoTJ2uqp5fh6+4WlMVUtIYK7AFNd7k3s1NyyAh3OsqTCLj+clGzi+mTzkxc6SJYAfBTlPSUJxU9NBcHSqiYAZSKha4npIMRMQSKBDo6N74tkkM/caAMviJsNFQCCBWSLNoQYJs2R7Y8y6CahMQjPpd6QUERBIYIhjr6B0VOnqlm6+NwGBBG/K3O3bzOu784Acw4W5/uwJTwpHsQn0F3Q2N5skPJMSEMzBkjLPnuvnFFxFAd0mhrphtjI3B7yKE4xtMkLCHQBLp8l69irzqiogmAJ/uharG5rH/Bhj1QlIkLCImD8bAQMSWD9Ue8ysovveIpMooOcUCf7c+pu92uAs601KgOUAreccCWh9A633Pyqg9Q203n/zCvgHXclLUIktpWUAAAAASUVORK5CYII=" style="width: 16px;height:16px;">':'<img alt="" src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAB9ElEQVR4nO2a4ZGCMBBGXwmUQAmUQAmWQAfawVmCHZgOvA7uOtAOzg60A+4Hw8jcAWYlycaQN7N/M98LYROikMlkMplM6nwApXYIDSrgDLSA0Y0Sni2d+LBKzUChKIAT/+VXsQoq4Idx+Ra460XzT8O0eC9faYXzzZGVyhfAFyuVH25xq5S/MS/f0vWF5Gh4Lr56+Z1SPq886/RJH3Zs5T+1AvrEVv5Cty0mha38ncTkbQ44Se/1tk++BTZKGb0hkd/rRPSHRN7oRPSHRD65ji+Rv5PY1ZZEvgVqlZSe2CGT32uELPCz1TTI5FWOuQWPS4fG4bgbZPJXFJreUL6vo4NxbS8zhhX8pDcm39eJ159GiVw++Lf9nHxfZ+Rbkc24Ubz3tiFvyJbm1K81Ub33IOvON+ya40Ewptp7P6QZCTRXW4djtUTykWOQhR7bIV7p+N9+dF7DIAt/5vHeFsz/UDlWUZ7zDfJJqLC/0RlWlJcbBd3np1RGWlHf6PqehLe41PQ5CXU4jWWUdE/LpfwhpIALKtxNwpU3WPpjuJqEOnBup9QskzehA/ugIeGub0uDfAKiPPAsQXK5GfWBZwkGu6Vf6sQLg2F+AvZawUJiGJe/KGYKytSRuVbMFJy/k2BU0yjRT0LyjW+OkkT/s5fJZDKZFPgF2+acfXQipdoAAAAASUVORK5CYII=" style="width: 16px;height:16px;">',e=$("#Real_Menu"),o=(e.html(""),""),A=0;A<a.length;A++)o+='<li data-url="'+a[A].url+'" data-action="'+(a[A].action||"")+'"><div class="display_inline">'+(t===a[A].url.toLowerCase()?n:"")+"</div>"+a[A].key+"</li>";e.append(o)},this.initMenu=function(){var t=this,a=(isLoginWatcher.onChange(function(e){t.generateMenu(e)},!0),window.darkThemeWatcher.onChange(function(e){t.generateMenu(e)}),$("#Menu"));$("#Menu_button").click(function(e){var t=$("#dropup2");"false"===a.attr("data-menu")?(t.addClass("open"),a.attr("data-menu","true")):(t.removeClass("open"),a.attr("data-menu","false")),$("#dropdown-backdrop").attr("class","dropdown-backdrop")}),$("#dropdown-backdrop").click(function(e){$("#dropup2").removeClass("open"),$("#tipBox").css("display","none"),a.attr("data-menu","false"),$("#dropdown-backdrop").attr("class","")}),$("#Real_Menu").on("click","li",function(e){var t;$(this).attr("data-url")?window.open("./"+$(this).attr("data-url")+".html","_self"):(t=$(this).attr("data-action"),console.log(t),t&&($("#dropup2").removeClass("open"),$("#tipBox").css("display","none"),a.attr("data-menu","false"),$("#dropdown-backdrop").attr("class",""),window[t]()))}),$("#white").click(function(){i()?($.cookie.set("black","false"),$("html").attr("class","white-theme"),window.darkThemeWatcher.update(!1)):($.cookie.set("black","true"),$("html").attr("class","dark-theme"),window.darkThemeWatcher.update(!0))}),$("#before").click(function(){void 0!==window.pagination&&pagination.beforeClick()}),$("#next").click(function(){void 0!==window.pagination&&pagination.nextClick()})}};