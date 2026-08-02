<template>
  <div v-if="!renderIframe" ref="shadowDom">
    <span v-if="loading">Loading...</span>
  </div>
  <iframe
    v-else
    class="epub-iframe"
    ref="iframe"
    :style="iframeStyle"
    :src="src"
  ></iframe>
</template>

<script>
import { Converter, HTMLConverter } from "pinyin-pro";
import Axios from "../plugins/axios";

/**
 * EPUB 渲染组件（与 reader-pro 3.2.14 JAR 产物功能对齐）
 * - renderType=iframe：传统 iframe 渲染
 * - renderType=shadowDom：shadow DOM 渲染（支持简繁转换、链接/图片重写、事件）
 */
const simplized = text => {
  const t = Converter({ from: "cn", to: "tw" });
  if (typeof text === "string") {
    return t(text);
  }
  let html = HTMLConverter(t, text, "", "zh-TW");
  html.convert();
  html = HTMLConverter(t, text, "zh-CN", "zh-TW");
  html.convert();
};

const traditionalized = text => {
  const t = Converter({ from: "tw", to: "cn" });
  if (typeof text === "string") {
    return t(text);
  }
  let html = HTMLConverter(t, text, "", "zh-CN");
  html.convert();
  html = HTMLConverter(t, text, "zh-TW", "zh-CN");
  html.convert();
};

export default {
  name: "ShadowIframe",
  data() {
    return {
      loading: true,
      initing: false,
      nowSrc: ""
    };
  },
  computed: {
    contentWindow() {
      return this.$refs.iframe && this.$refs.iframe.contentWindow;
    },
    renderIframe() {
      return "iframe" === this.renderType;
    },
    completeSrc() {
      if (this.nowSrc) {
        return this.nowSrc.startsWith("//")
          ? window.location.protocol + this.nowSrc
          : this.nowSrc;
      }
      return this.src.startsWith("//")
        ? window.location.protocol + this.src
        : this.src;
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
      if (!this.renderIframe) {
        if (value === "简体") {
          traditionalized(this.shadowRoot.documentElement);
        } else if (value === "繁体") {
          simplized(this.shadowRoot.documentElement);
        }
      }
    }
  },
  props: ["src", "iframeStyle", "renderType"],
  created() {
    this.initIframe();
  },
  mounted() {
    window.shadowIframe = this;
    window.simplized = traditionalized;
    window.traditionalized = simplized;
    this.loadShadowDom(this.completeSrc);
  },
  methods: {
    /**
     * 重写 HTML 中的链接/图片为绝对 URL，并按阅读配置做简繁转换
     */
    htmlLoader(html, baseUrl) {
      let result = html
        .replace(/\sxlink:href=('|")?([^>'"\s]+)/gi, match => {
          const href = match.match(/href=('|")?([^>'"\s]+)/);
          if (href) {
            const abs = new URL(href[2], baseUrl);
            return ` xlink:href="${abs}"`;
          }
          return match;
        })
        .replace(/<a[^>]*>/gi, tag => {
          const href = tag.match(/href=('|")?([^>'"\s]+)/);
          if (href) {
            const abs = new URL(href[2], baseUrl);
            return tag.replace(href[2], abs);
          }
          return tag;
        })
        .replace(/\ssrc=['"](\S+)['"]/gi, match => {
          const src = match.match(/src=('|")?([^>'"\s]+)/);
          if (src) {
            const abs = new URL(src[2], baseUrl);
            return ` src="${abs}"`;
          }
          return match;
        });
      if (this.$store.getters.config.chineseFont === "简体") {
        result = traditionalized(result);
      } else {
        result = simplized(result);
      }
      return result;
    },
    async loadShadowDom(url) {
      if (this.initing || !url) {
        return;
      }
      this.initing = true;
      this.nowSrc = url;
      try {
        const res = await Axios.get(url);
        const html = this.htmlLoader(res.data, url);
        const doc = new DOMParser().parseFromString(html, "text/html");
        const template = doc.documentElement.outerHTML;
        this.renderShadowDom(template, url);
      } catch (e) {
        // eslint-disable-next-line no-console
        console.error(e);
        this.initing = false;
      }
    },
    renderShadowDom(html, url) {
      if (!this.$refs.shadowDom) {
        setTimeout(() => {
          this.renderShadowDom(html, url);
        }, 10);
        return;
      }
      if (!this.$refs.shadowDom.shadowRoot) {
        this.shadowRoot = this.$refs.shadowDom.attachShadow({ mode: "open" });
      }
      if (this.shadowRoot) {
        const htmlEl = document.createElement("html");
        htmlEl.innerHTML = html;
        this.loading = false;
        this.reader_style_dom = document.createElement("style");
        const head = htmlEl.getElementsByTagName("head")[0];
        head.appendChild(this.reader_style_dom);
        if (this.shadowRoot.firstChild) {
          this.shadowRoot.firstChild.remove();
        }
        this.shadowRoot.documentElement = htmlEl;
        this.shadowRoot.head = head;
        this.shadowRoot.body = htmlEl.querySelector("body");
        this.shadowRoot.appendChild(htmlEl);
        this.initShadowDomJavascript(url);
      }
    },
    initShadowDomJavascript(url) {
      const self = this;
      const htmlEl = this.shadowRoot.querySelector("html");
      const body = this.shadowRoot.querySelector("body");
      const baseUrl = new URL(url);

      const emitEvent = (event, data) => {
        this.$emit("iframeEvent", event, data);
      };
      const syncSize = () => {
        emitEvent(
          "setHeight",
          Math.max(htmlEl.scrollHeight, body.scrollHeight)
        );
        emitEvent("setWidth", Math.max(htmlEl.scrollWidth, body.scrollWidth));
      };
      const findLink = el => {
        if (!el || !el.nodeName) {
          return null;
        }
        return el.nodeName.toLowerCase() === "a"
          ? el
          : findLink(el.parentNode);
      };
      const findImg = el => {
        if (!el || !el.nodeName) {
          return null;
        }
        return el.nodeName.toLowerCase() === "img" ? el : undefined;
      };

      htmlEl.addEventListener("load", () => {
        syncSize();
        emitEvent("load", url);
      });
      window.addEventListener("resize", syncSize);
      htmlEl.addEventListener("DOMNodeInserted", syncSize, false);
      htmlEl.addEventListener("click", e => {
        const link = findLink(e.target);
        const img = findImg(e.target);
        if (link) {
          if (link.pathname === baseUrl.pathname) {
            const target = htmlEl.querySelector(link.hash);
            if (target) {
              emitEvent("clickHash", target.getBoundingClientRect());
              e.preventDefault();
            }
          } else {
            e.preventDefault();
            emitEvent("clickA", e.target.href);
            self.loadShadowDom(e.target.href);
          }
        } else if (img) {
          const images = htmlEl.querySelectorAll("img");
          if (images.length) {
            const imageList = [];
            let imageIndex = 0;
            for (let i = 0; i < images.length; i++) {
              imageList.push(images[i].src);
              if (images[i] === img) {
                imageIndex = i;
              }
            }
            emitEvent("previewImageList", {
              imageList,
              imageIndex
            });
          }
        } else {
          emitEvent("click", {
            target: e.target.nodeName,
            clientX: e.clientX,
            clientY: e.clientY
          });
        }
      });
      this.$emit("iframeEvent", "inited");
      this.initing = false;
      this.$nextTick(() => {
        this.$emit("iframeEvent", "load");
      });
    },
    initIframe() {
      window.addEventListener("message", event => {
        if (
          this.$refs.iframe &&
          event.source === this.$refs.iframe.contentWindow
        ) {
          let data;
          try {
            data = JSON.parse(event.data);
          } catch (e) {
            return;
          }
          this.$emit("iframeEvent", data.event, data.data);
        }
      });
    },
    syncIframeHeight() {
      if (this.renderIframe) {
        this.sendToIframe("execute", {
          script: `
          var syncHeight = function(){
            if(!document.body){
              setTimeout(syncHeight, 10);
              return;
            }
            reader_notify('setHeight', document.documentElement.scrollHeight || document.body.scrollHeight);
          };
          syncHeight();
          `
        });
      } else {
        this.ensure(
          () => {
            const htmlEl = this.shadowRoot.querySelector("html");
            const body = this.shadowRoot.querySelector("body");
            this.$emit(
              "iframeEvent",
              "setHeight",
              Math.max(htmlEl.scrollHeight, body.scrollHeight)
            );
          },
          () => this.shadowRoot
        );
      }
    },
    syncIframeWidth() {
      if (this.renderIframe) {
        this.sendToIframe("execute", {
          script: `var syncWidth = function(){
            if(!document.body){
              setTimeout(syncWidth, 10);
              return;
            }
            reader_notify('setWidth', document.body.scrollWidth);
          };
          syncWidth();
          `
        });
      } else {
        this.ensure(
          () => {
            const htmlEl = this.shadowRoot.querySelector("html");
            const body = this.shadowRoot.querySelector("body");
            this.$emit(
              "iframeEvent",
              "setWidth",
              Math.max(htmlEl.scrollWidth, body.scrollWidth)
            );
          },
          () => this.shadowRoot
        );
      }
    },
    setIframeStyle(style) {
      if (this.renderIframe) {
        this.sendToIframe("setStyle", { style });
      } else {
        style += `
        html {
          height: 100%;
          columns: calc(100vw - 16px - var(--horizontal-padding, 0px)) 1;
          column-gap: calc(16px + var(--horizontal-padding, 0px));
        }
        `;
        this.ensure(
          () => {
            this.reader_style_dom.textContent = style;
          },
          () => this.reader_style_dom
        );
      }
    },
    transformX(value) {
      if (this.renderIframe) {
        this.sendToIframe("execute", {
          script: `document.body.style.transform="translateX(${value}px)";`
        });
      } else {
        this.ensure(
          () => {
            const body = this.shadowRoot.querySelector("body");
            body.style.transform = `translateX(${value}px)`;
          },
          () => this.shadowRoot
        );
      }
    },
    syncCSSProperty() {
      if (this.renderIframe) {
        this.sendToIframe("execute", {
          script: `document.documentElement.style.setProperty('--horizontal-padding', "${0 |
            this.$store.state.config.horizontalPadding}px");`
        });
      }
    },
    sendToIframe(event, data) {
      if (
        this.$refs.iframe &&
        this.$refs.iframe.contentWindow
      ) {
        this.$refs.iframe.contentWindow.postMessage(
          JSON.stringify({ event, ...data }),
          "*"
        );
      } else {
        setTimeout(() => {
          this.sendToIframe(event, data);
        }, 10);
      }
    },
    ensure(callback, condition) {
      if (condition && condition()) {
        if (callback) {
          callback();
        }
      } else {
        setTimeout(() => {
          this.ensure(callback, condition);
        }, 10);
      }
    }
  }
};
</script>
