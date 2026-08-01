<script>
import Axios from "../plugins/axios";
import { simplized, traditionalized } from "../plugins/chinese";

function absoluteUrl(value, base) {
  try {
    return new URL(value, base).toString();
  } catch (error) {
    return value;
  }
}

function rewriteCss(css, base) {
  const rewritten = css.replace(/url\((\s*["']?)([^)"']+)\1\s*\)/gi, (all, quote, url) => {
    if (/^(data:|blob:|https?:|\/\/|#)/i.test(url.trim())) {
      return all;
    }
    return `url(${quote}${absoluteUrl(url.trim(), base)}${quote})`;
  });
  return rewritten.replace(/(@import\s+)(["'])([^"']+)\2/gi, (all, prefix, quote, url) => {
    if (/^(data:|blob:|https?:|\/\/|#)/i.test(url.trim())) {
      return all;
    }
    return `${prefix}${quote}${absoluteUrl(url.trim(), base)}${quote}`;
  });
}

export default {
  name: "ShadowIframe",
  props: ["src", "iframeStyle", "renderType"],
  data() {
    return {
      loading: true,
      initing: false,
      nowSrc: "",
      shadowRoot: null,
      readerStyleDom: null,
      resizeHandler: null,
      loadToken: 0
    };
  },
  computed: {
    contentWindow() {
      return this.$refs.iframe && this.$refs.iframe.contentWindow;
    },
    renderIframe() {
      return this.renderType === "iframe";
    },
    completeSrc() {
      const src = this.nowSrc || this.src || "";
      return src.startsWith("//") ? window.location.protocol + src : src;
    },
    chineseFont() {
      return this.$store.getters.config.chineseFont;
    }
  },
  watch: {
    src(value) {
      this.loadShadowDom(value);
    },
    renderIframe(value) {
      if (!value) {
        this.loadShadowDom(this.nowSrc || this.src);
      }
    },
    chineseFont(value) {
      if (this.renderIframe || !this.shadowRoot || !this.shadowRoot.documentElement) {
        return;
      }
      this.convertChinese(this.shadowRoot.documentElement, value);
    }
  },
  render(h) {
    if (this.renderIframe) {
      return h("iframe", {
        class: "epub-iframe",
        ref: "iframe",
        style: this.iframeStyle,
        attrs: { src: this.src }
      });
    }
    return h("div", { ref: "shadowDom" }, [
      this.loading ? h("span", ["Loading..."]) : null
    ]);
  },
  created() {
    this.initIframe();
  },
  mounted() {
    window.shadowIframe = this;
    // The original bundle exposes these converters for embedded reader code.
    window.simplized = simplized;
    window.traditionalized = traditionalized;
    this.loadShadowDom(this.completeSrc);
  },
  beforeDestroy() {
    this.loadToken += 1;
    if (this.resizeHandler) {
      window.removeEventListener("resize", this.resizeHandler);
    }
    delete window.shadowIframe;
  },
  methods: {
    convertChinese(root, value) {
      if (!root) return;
      const converter = value === "简体" ? simplized : value === "繁体" ? traditionalized : null;
      if (typeof converter === "function") converter(root);
    },
    async loadShadowDom(src) {
      if (this.renderIframe || this.initing || !src) return;
      const token = ++this.loadToken;
      this.initing = true;
      this.nowSrc = src;
      try {
        const response = await this.fetchResource(this.completeSrc);
        if (!response.ok) throw new Error(`EPUB request failed: ${response.status}`);
        let html = await response.text();
        html = this.rewriteHtml(html, this.completeSrc);
        html = this.chineseFont === "简体" ? simplized(html) : traditionalized(html);
        const documentNode = new DOMParser().parseFromString(html, "text/html");
        await this.inlineStyles(documentNode, this.completeSrc);
        if (token !== this.loadToken) return;
        this.renderShadowDom(documentNode.documentElement.innerHTML, this.completeSrc);
      } catch (error) {
        // Keep the iframe fallback available when a browser cannot resolve Shadow DOM resources.
        // eslint-disable-next-line no-console
        console.error(error);
      } finally {
        if (token === this.loadToken) this.initing = false;
      }
    },
    async fetchResource(url) {
      const response = await Axios.get(url, { silent: true });
      return {
        ok: response.status >= 200 && response.status < 400,
        status: response.status,
        text: async () => response.data
      };
    },
    rewriteHtml(html, base) {
      // Keep the same attribute boundaries as the JAR htmlLoader. Parsing the
      // whole document first changes malformed EPUB markup before it can be
      // rendered and does not match the bundle's URL handling.
      return html
        .replace(/\sxlink:href=('|\")?([^>'\"\s]+)/gi, all => {
          const match = all.match(/href=('|\")?([^>'\"\s]+)/);
          if (!match) return all;
          return ` xlink:href="${absoluteUrl(match[2], base)}"`;
        })
        .replace(/<a[^>]*>/gi, tag => {
          const match = tag.match(/href=('|\")?([^>'\"\s]+)/);
          if (!match) return tag;
          return tag.replace(match[2], absoluteUrl(match[2], base));
        })
        .replace(/\ssrc=['\"](\S+)['\"]/gi, all => {
          const match = all.match(/src=('|\")?([^>'\"\s]+)/);
          if (!match) return all;
          return ` src="${absoluteUrl(match[2], base)}"`;
        });
    },
    async inlineStyles(documentNode, base) {
      const links = Array.from(documentNode.querySelectorAll("link[rel~='stylesheet'][href]"));
      await Promise.all(links.map(async link => {
        const href = absoluteUrl(link.getAttribute("href"), base);
        try {
          const response = await this.fetchResource(href);
          if (!response.ok) {
            link.setAttribute("href", href);
            return;
          }
          const style = documentNode.createElement("style");
          style.textContent = rewriteCss(await response.text(), href);
          link.replaceWith(style);
        } catch (error) {
          // A broken optional stylesheet must not prevent the chapter from rendering.
          link.setAttribute("href", href);
        }
      }));
    },
    renderShadowDom(html, src) {
      if (!this.$refs.shadowDom) {
        setTimeout(() => this.renderShadowDom(html, src), 10);
        return;
      }
      this.shadowRoot = this.$refs.shadowDom.shadowRoot || this.$refs.shadowDom.attachShadow({ mode: "open" });
      const documentNode = document.createElement("html");
      documentNode.innerHTML = html;
      const head = documentNode.querySelector("head") || documentNode.insertBefore(document.createElement("head"), documentNode.firstChild);
      const body = documentNode.querySelector("body") || documentNode.appendChild(document.createElement("body"));
      this.readerStyleDom = document.createElement("style");
      head.appendChild(this.readerStyleDom);
      while (this.shadowRoot.firstChild) this.shadowRoot.firstChild.remove();
      this.shadowRoot.documentElement = documentNode;
      this.shadowRoot.head = head;
      this.shadowRoot.body = body;
      this.shadowRoot.appendChild(documentNode);
      this.loading = false;
      this.initShadowDomJavascript(src);
    },
    initShadowDomJavascript(src) {
      const root = this.shadowRoot;
      const html = root && root.querySelector("html");
      const body = root && root.querySelector("body");
      if (!html || !body) return;
      const chapterUrl = new URL(src, window.location.href);
      const emit = (event, data) => this.$emit("iframeEvent", event, data);
      const updateSize = () => {
        emit("setHeight", Math.max(html.scrollHeight, body.scrollHeight));
        emit("setWidth", Math.max(html.scrollWidth, body.scrollWidth));
      };
      const findParent = (node, name) => {
        while (node && node.nodeName) {
          if (node.nodeName.toLowerCase() === name) return node;
          node = node.parentNode;
        }
        return null;
      };
      const clickHandler = event => {
        const anchor = findParent(event.target, "a");
        const image = findParent(event.target, "img");
        if (anchor) {
          const href = absoluteUrl(anchor.getAttribute("href"), src);
          const link = new URL(href, window.location.href);
          if (link.pathname === chapterUrl.pathname) {
            const target = html.querySelector(link.hash);
            if (target) {
              emit("clickHash", target.getBoundingClientRect());
              event.preventDefault();
            }
          } else {
            event.preventDefault();
            emit("clickA", href);
            this.loadShadowDom(href);
          }
        } else if (image) {
          const images = Array.from(root.querySelectorAll("img"));
          emit("previewImageList", {
            imageList: images.map(item => item.src),
            imageIndex: images.indexOf(image)
          });
        } else {
          emit("click", {
            target: event.target.nodeName,
            clientX: event.clientX,
            clientY: event.clientY
          });
        }
      };
      html.addEventListener("load", () => {
        updateSize();
        emit("load", src);
      });
      html.addEventListener("click", clickHandler);
      html.addEventListener("keydown", event => this.$emit("iframeEvent", "keydown", event));
      html.addEventListener("DOMNodeInserted", updateSize, false);
      this.resizeHandler = updateSize;
      window.addEventListener("resize", updateSize);
      this.$emit("iframeEvent", "inited");
      this.initing = false;
      this.$nextTick(() => {
        this.$emit("iframeEvent", "load");
      });
    },
    initIframe() {
      window.addEventListener("message", event => {
        if (!this.$refs.iframe || event.source !== this.$refs.iframe.contentWindow) return;
        try {
          const message = JSON.parse(event.data);
          this.$emit("iframeEvent", message.event, message.data);
        } catch (error) {
          // Ignore messages that are not reader protocol messages.
        }
      });
    },
    syncIframeHeight() {
      if (this.renderIframe) {
        this.sendToIframe("execute", {
          script: "var syncHeight=function(){if(!document.body){setTimeout(syncHeight,10);return;}reader_notify('setWidth',document.documentElement.scrollHeight||document.body.scrollHeight);};syncHeight();"
        });
      } else if (this.shadowRoot) {
        const html = this.shadowRoot.querySelector("html");
        const body = this.shadowRoot.querySelector("body");
        this.$emit("iframeEvent", "setHeight", Math.max(html.scrollHeight, body.scrollHeight));
      }
    },
    syncIframeWidth() {
      if (this.renderIframe) {
        this.sendToIframe("execute", {
          script: "var syncWidth=function(){if(!document.body){setTimeout(syncWidth,10);return;}reader_notify('setWidth',document.body.scrollWidth);};syncWidth();"
        });
      } else if (this.shadowRoot) {
        const html = this.shadowRoot.querySelector("html");
        const body = this.shadowRoot.querySelector("body");
        this.$emit("iframeEvent", "setWidth", Math.max(html.scrollWidth, body.scrollWidth));
      }
    },
    setIframeStyle(style) {
      if (this.renderIframe) {
        this.sendToIframe("setStyle", { style });
      } else if (this.readerStyleDom) {
        this.readerStyleDom.textContent = `${style}
        html {
          height: 100%;
          columns: calc(100vw - 16px - var(--horizontal-padding, 0px)) 1;
          column-gap: calc(16px + var(--horizontal-padding, 0px));
        }
        `;
      }
    },
    transformX(x) {
      if (this.renderIframe) {
        this.sendToIframe("execute", { script: `document.body.style.transform="translateX(${x}px)";` });
      } else if (this.shadowRoot) {
        this.shadowRoot.querySelector("body").style.transform = `translateX(${x}px)`;
      }
    },
    syncCSSProperty() {
      if (this.renderIframe) {
        this.sendToIframe("execute", {
          script: `document.documentElement.style.setProperty('--horizontal-padding', "${0 | this.$store.state.config.horizontalPadding}px");`
        });
      }
    },
    sendToIframe(event, data) {
      if (!this.$refs.iframe || !this.$refs.iframe.contentWindow) {
        setTimeout(() => this.sendToIframe(event, data), 10);
        return;
      }
      this.$refs.iframe.contentWindow.postMessage(JSON.stringify({ event, ...data }), "*");
    }
  }
};
</script>
