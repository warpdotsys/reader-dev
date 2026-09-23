// Compatibility shim for the original minified simple-web bundle.
// Keep explicit API settings intact; only fix the default for mounted deployments.
(function () {
    if (!window.$ || !$.ajax || !window.myStorage) return;
    var explicitApi = $.getUrlPra("api") || $.cookie.get("api") ||
        window.myStorage.getItem("api_prefix");
    if (explicitApi) return;
    var match = window.location.pathname.match(/^(.*)\/simple-web(?:\/|$)/);
    if (match) $.ajax.baseURL = match[1] + "/reader3";
})();
