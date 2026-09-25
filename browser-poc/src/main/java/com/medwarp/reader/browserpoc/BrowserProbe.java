package com.medwarp.reader.browserpoc;

import com.microsoft.playwright.Browser;
import com.microsoft.playwright.BrowserContext;
import com.microsoft.playwright.BrowserType;
import com.microsoft.playwright.Page;
import com.microsoft.playwright.Playwright;
import com.microsoft.playwright.Response;
import com.microsoft.playwright.Route;
import com.microsoft.playwright.options.Cookie;
import com.microsoft.playwright.options.Proxy;
import com.microsoft.playwright.options.WaitUntilState;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * Isolated capability probe, deliberately not wired into the Reader application.
 * Each request gets a new browser context; cookies live only in this probe's memory.
 */
public final class BrowserProbe {
    private final Browser browser;
    private final Map<String, List<Cookie>> cookiesByNamespace = new HashMap<>();

    public BrowserProbe(Browser browser) {
        this.browser = browser;
    }

    /** Playwright Java browser objects are not shared across concurrent callers. */
    public synchronized Result render(Request request) {
        if ((request.url == null) == (request.html == null)) {
            throw new IllegalArgumentException("Specify exactly one of url or html");
        }
        if (request.postData != null && request.url == null) {
            throw new IllegalArgumentException("POST requires url");
        }
        if (request.timeoutMillis <= 0) {
            throw new IllegalArgumentException("timeoutMillis must be positive");
        }

        Browser.NewContextOptions options = new Browser.NewContextOptions().setAcceptDownloads(false);
        if (request.proxy != null && !request.proxy.isEmpty()) {
            options.setProxy(new Proxy(request.proxy));
        }
        BrowserContext context = browser.newContext(options);
        try {
            List<Cookie> remembered = cookiesByNamespace.get(request.userNamespace);
            if (remembered != null && !remembered.isEmpty()) {
                context.addCookies(remembered);
            }
            context.setDefaultTimeout(request.timeoutMillis);
            context.setDefaultNavigationTimeout(request.timeoutMillis);
            if (!request.headers.isEmpty()) {
                context.setExtraHTTPHeaders(request.headers);
            }

            Page page = context.newPage();
            int status = 200;
            if (request.url != null) {
                if (request.postData != null) {
                    AtomicBoolean sent = new AtomicBoolean(false);
                    page.route("**/*", route -> {
                        if (!sent.get() && route.request().isNavigationRequest()
                                && request.url.equals(route.request().url())
                                && sent.compareAndSet(false, true)) {
                            route.resume(new Route.ResumeOptions()
                                    .setMethod("POST")
                                    .setPostData(request.postData));
                        } else {
                            route.resume();
                        }
                    });
                }
                Response response = page.navigate(request.url, new Page.NavigateOptions()
                        .setWaitUntil(WaitUntilState.DOMCONTENTLOADED)
                        .setTimeout(request.timeoutMillis));
                if (response != null) {
                    status = response.status();
                }
            } else {
                page.setContent(request.html, new Page.SetContentOptions()
                        .setWaitUntil(WaitUntilState.DOMCONTENTLOADED)
                        .setTimeout(request.timeoutMillis));
            }

            Object scriptResult = request.script == null ? null : page.evaluate(request.script);
            String renderedHtml = page.content();
            cookiesByNamespace.put(request.userNamespace, new ArrayList<>(context.cookies()));
            return new Result(status, page.url(), renderedHtml, scriptResult);
        } finally {
            context.close();
        }
    }

    public static final class Request {
        public final String url;
        public final String html;
        public final String userNamespace;
        public final String postData;
        public final Map<String, String> headers;
        public final String script;
        public final String proxy;
        public final double timeoutMillis;

        public Request(String url, String html, String userNamespace, String postData,
                       Map<String, String> headers, String script, String proxy, double timeoutMillis) {
            this.url = url;
            this.html = html;
            this.userNamespace = userNamespace == null ? "" : userNamespace;
            this.postData = postData;
            this.headers = headers == null ? Collections.emptyMap() : new HashMap<>(headers);
            this.script = script;
            this.proxy = proxy;
            this.timeoutMillis = timeoutMillis;
        }
    }

    public static final class Result {
        public final int status;
        public final String finalUrl;
        public final String html;
        public final Object scriptResult;

        private Result(int status, String finalUrl, String html, Object scriptResult) {
            this.status = status;
            this.finalUrl = finalUrl;
            this.html = html;
            this.scriptResult = scriptResult;
        }
    }

    public static void main(String[] args) {
        if (args.length != 2) {
            System.err.println("Usage: BrowserProbe <browser-executable|bundled> <url>");
            System.exit(2);
        }
        boolean bundled = "bundled".equals(args[0]);
        Path executable = bundled ? null : Path.of(args[0]);
        if (!bundled && !Files.isRegularFile(executable)) {
            throw new IllegalArgumentException("Browser executable does not exist: " + executable);
        }
        try (Playwright playwright = Playwright.create(new Playwright.CreateOptions()
                .setEnv(Collections.singletonMap("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1")))) {
            BrowserType.LaunchOptions launchOptions = new BrowserType.LaunchOptions().setHeadless(true);
            if (executable != null) {
                launchOptions.setExecutablePath(executable);
            }
            Browser browser = playwright.chromium().launch(launchOptions);
            try {
                Result result = new BrowserProbe(browser).render(new Request(
                        args[1], null, "probe", null, null, null, null, 15000));
                System.out.println("HTTP " + result.status + " " + result.finalUrl);
                System.out.println(result.html);
            } finally {
                browser.close();
            }
        }
    }
}
