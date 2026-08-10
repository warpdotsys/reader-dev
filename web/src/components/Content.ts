import DPlayer from "dplayer";
import ShadowIframe from "./ShadowIframe.vue";
import { loadFont } from "../plugins/helper";

export const Content: any = {
  name: "Content",
  data() {
    return {
      currentTime: 0,
      audioDuration: 0,
      playing: false,
      currentSpeed: 1,
      audioVolume: 100,
      startTime: 0,
      iframeStyle: {},
      iframeSize: { scrollHeight: 0, scrollWidth: 0 }
    };
  },
  props: [
    "content",
    "error",
    "title",
    "showContent",
    "isScrollRead",
    "isSlideRead",
    "showChapterList",
    "currentShowChapter",
    "isEditContent"
  ],
  render() {
    if (this.isVideo) {
      // 视频
      return this.renderVideo();
    }
    if (this.showContent) {
      if (
        this.$store.getters.currentChapter &&
        this.$store.getters.currentChapter.isVolume
      ) {
        return (
          <div
            class="content-body chapter-content reading-chapter volume-chapter"
            style={this.containerStyle}
          >
            <div class="volume-content">
              <h3 data-pos={0}>{this.title}</h3>
              <p class="volume-tag">{this.content}</p>
            </div>
          </div>
        );
      }
      if (this.isAudio) {
        // 音频
        return this.renderAudio();
      } else if (this.isEpub) {
        // epub
        return this.renderEpub();
      }
      if (this.isScrollRead) {
        return this.renderScrollChapterList();
      }
      let wordCount = this.title.length + 2; // 2为两个换行符
      return (
        <div
          class="content-body chapter-content reading-chapter"
          style={this.containerStyle}
          attrs={{ contenteditable: this.isEditContent }}
          v-lazy-container={{
            selector: "img"
          }}
        >
          {this.isCbz || this.isPdf ? null : <h3 data-pos={0}>{this.title}</h3>}
          {this.content.split(/\n+/).map(a => {
            a = a.replace(/^\s+/g, "");
            if (!a) {
              return null;
            }
            const pos = wordCount;
            wordCount += a.length + 2; // 2为两个换行符
            if (a.indexOf("<img") >= 0) {
              // 漫画
              // 将 src 替换为 data-src 懒加载
              a = a
                .replace(/src=/g, "data-src=")
                .replace(/__API_ROOT__/g, this.$store.getters.apiRoot);
              return (
                <div
                  style={this.containerStyle}
                  domPropsInnerHTML={a}
                  data-pos={pos}
                ></div>
              );
            }
            // 文本内容
            return (
              <p style={this.pStyle} domPropsInnerHTML={a} data-pos={pos} />
            );
          })}
        </div>
      );
    } else {
      return <div />;
    }
  },
  mounted() {
    if (this.isAudio) {
      this.play(true);
    } else if (this.isEpub) {
      this.initIframe();
    }
    window.contentCom = this;
    this.loadCustomFontFamil();
  },
  unmounted() {
    delete window.contentCom;
  },
  computed: {
    readingBook() {
      return this.$store.getters.readingBook;
    },
    chapter() {
      return (
        this.$store.getters.readingBook.catalog[
          this.$store.getters.readingBook.index
        ] || {}
      );
    },
    show() {
      return this.$store.state.showContent;
    },
    fontSize() {
      return this.$store.getters.config.fontSize + "px";
    },
    autoPlay: {
      get() {
        return this.$store.state.autoPlay;
      },
      set(val) {
        this.$store.commit("setAutoPlay", val);
      }
    },
    isCarToon() {
      return (
        !this.error && !this.isEpub && (this.content || "").indexOf("<img") >= 0
      );
    },
    isAudio() {
      return !this.error && this.readingBook.type === 1;
    },
    isVideo() {
      return !this.error && this.readingBook.type === 4;
    },
    isEpub() {
      return (
        !this.error && this.readingBook.bookUrl.toLowerCase().endsWith(".epub")
      );
    },
    isEpubIframe() {
      return this.$store.getters.config.epubMode === "iframe";
    },
    isCbz() {
      return (
        !this.error && this.readingBook.bookUrl.toLowerCase().endsWith(".cbz")
      );
    },
    isPdf() {
      return (
        !this.error && this.readingBook.bookUrl.toLowerCase().endsWith(".pdf")
      );
    },
    containerStyle() {
      return {
        fontSize: this.$store.getters.config.fontSize + "px",
        fontWeight: this.$store.getters.config.fontWeight || undefined,
        color:
          this.$store.getters.config.fontColor ||
          (this.$store.getters.isNight ? "#666" : "#262626"),
        ...this.$store.getters.currentFontFamily,
        ...(this.$store.getters.config.contentCSS || {})
      };
    },
    pStyle() {
      return {
        lineHeight: this.$store.getters.config.lineHeight,
        marginTop:
          typeof this.$store.getters.config.paragraphSpace !== "undefined"
            ? this.$store.getters.config.paragraphSpace + "em"
            : null,
        marginBottom:
          typeof this.$store.getters.config.paragraphSpace !== "undefined"
            ? this.$store.getters.config.paragraphSpace + "em"
            : null
      };
    },
    windowSize() {
      return this.$store.state.windowSize;
    },
    currentCustomFontFamily() {
      return this.$store.getters.currentCustomFontFamily;
    }
  },
  watch: {
    containerStyle() {
      if (this.isEpub && this.isEpubIframe) {
        this.setIframeStyle();
      }
    },
    pStyle() {
      if (this.isEpub && this.isEpubIframe) {
        this.setIframeStyle();
      }
    },
    windowSize() {
      if (this.isEpub) {
        //
      }
    },
    currentCustomFontFamily() {
      this.loadCustomFontFamil();
    },
    isSlideRead() {
      if (this.isEpub && this.isEpubIframe) {
        this.iframeStyle = {};
        this.setIframeStyle();
        if (this.isSlideRead) {
          this.syncIframeWidth();
        } else {
          this.syncIframeHeight();
        }
        this.sendToIframe("execute", {
          script: 'document.body.style.transform="translateX(0px)";'
        });
      }
    },
    showContent: {
      handler(val) {
        if (val && this.isVideo) {
          this.initDplayer();
        }
      },
      immediate: true
    }
  },
  methods: {
    renderScrollChapterList() {
      return (
        <div
          class="content-body"
          style={this.containerStyle}
          v-lazy-container={{
            selector: "img"
          }}
        >
          {this.showChapterList.map(chapter => {
            if (chapter.isVolume) {
              return (
                <div key={chapter.index} class="content-body chapter-content reading-chapter volume-chapter">
                  <div class="volume-content">
                    <h3 data-pos={0}>{chapter.title}</h3>
                    <p class="volume-tag">{chapter.content}</p>
                  </div>
                </div>
              );
            }
            let wordCount = chapter.title.length + 2; // 2为两个换行符
            return (
              <div
                key={chapter.index}
                class={[
                  "chapter-content",
                  this.readingBook.index === chapter.index
                    ? "reading-chapter"
                    : ""
                ]}
                data-index={chapter.index}
              >
                {this.isCbz || this.isPdf ? null : <h3 data-pos={0}>{chapter.title}</h3>}
                {chapter.content.split(/\n+/).map(a => {
                  a = a.replace(/^\s+/g, "");
                  if (!a) {
                    return null;
                  }
                  const pos = wordCount;
                  wordCount += a.length + 2; // 2为两个换行符
                  if (a.indexOf("<img") >= 0) {
                    // 漫画
                    // 将 src 替换为 data-src 懒加载
                    a = a
                      .replace(/src=/g, "data-src=")
                      .replace("__API_ROOT__", this.$store.getters.apiRoot);
                    return (
                      <div
                        style={this.containerStyle}
                        domPropsInnerHTML={a}
                        data-pos={pos}
                      ></div>
                    );
                  }
                  // 文本内容
                  return (
                    <p
                      style={this.pStyle}
                      domPropsInnerHTML={a}
                      data-pos={pos}
                    />
                  );
                })}
              </div>
            );
          })}
        </div>
      );
    },
    renderAudio() {
      return (
        <div class="content-audio">
          <audio
            ref="audio"
            preload="preload"
            src={this.content}
            vOn:loadMetaData={this.audioEvent}
            vOn:progress={this.onProgress}
            vOn:playing={this.onProgress}
            vOn:timeupdate={this.onTimeupdate}
            vOn:play={this.onPlay}
            vOn:pause={this.onPause}
            vOn:ended={this.onEnd}
            vOn:error={this.onError}
            vOn:seeked={this.onSeeked}
            vOn:seeking={this.onSeeking}
            vOn:stalled={this.audioEvent}
            vOn:suspend={this.onsuspend}
            vOn:loadeddata={this.audioEvent}
            vOn:loadedmetadata={this.computeDuration}
            vOn:durationchange={this.computeDuration}
            vOn:canplay={this.onCanPlay}
            vOn:canplaythrough={this.audioEvent}
            vOn:waiting={this.onWaiting}
          ></audio>
          <div class="book-cover">
            <img v-lazy={this.getCover(this.readingBook.coverUrl)} />
          </div>
          <div class="book-progress">
            <div class="progress-tip">{this.formatTime(this.currentTime)}</div>
            <div class="progress-container">
              <el-slider
                vModel={this.currentTime}
                min={0}
                max={this.audioDuration}
                show-tooltip={false}
                vOn:change={val => {
                  this.seekTime(val);
                }}
              ></el-slider>
            </div>
            <div class="progress-tip total-time">
              {this.formatTime(this.audioDuration)}
            </div>
          </div>
          <div class="book-operation">
            <i
              class="reader-iconfont reader-icon-jian15s"
              vOn:click_stop_prevent={() => {
                this.seekTime(this.$refs.audio.currentTime - 15);
              }}
            ></i>
            <i
              class="reader-iconfont reader-icon-player-backward-step"
              vOn:click_stop_prevent={this.prevChapter}
            ></i>
            <i
              class={[
                "reader-iconfont",
                this.playing
                  ? "reader-icon-player-play"
                  : "reader-icon-player-pause"
              ]}
              vOn:click_stop_prevent={this.toggle}
            ></i>
            <i
              class="reader-iconfont reader-icon-player-forward-step"
              vOn:click_stop_prevent={this.nextChapter}
            ></i>
            <i
              class="reader-iconfont reader-icon-15s"
              vOn:click_stop_prevent={() => {
                this.seekTime(this.$refs.audio.currentTime + 15);
              }}
            ></i>
          </div>
          <div class="book-operation">
            <span
              style={{
                display: "flex",
                flexDirection: "row",
                alignItems: "center"
              }}
            >
              <i
                class={[
                  "reader-iconfont",
                  this.audioVolume > 0
                    ? "reader-icon-volume"
                    : "reader-icon-volume-off"
                ]}
                vOn:click_stop_prevent={() => {
                  this.setAudioVolume(this.audioVolume > 0 ? 0 : 100);
                }}
                style={{ marginRight: this.audioVolume > 0 ? "15px" : "25px" }}
              ></i>
              <el-slider
                vModel={this.audioVolume}
                min={0}
                max={100}
                style={{ width: "180px" }}
                show-tooltip={false}
                vOn:change={val => {
                  this.setAudioVolume(val);
                }}
              ></el-slider>
            </span>
          </div>
          <div
            class="book-info"
            style={{
              background: this.getCover(this.readingBook.coverUrl, true)
            }}
          >
            <div class="book-cover">
              <img v-lazy={this.getCover(this.readingBook.coverUrl)} />
            </div>
            <div class="book-intro">
              <div class="title">{this.title}</div>
              <div class="subtitle">
                {this.readingBook.name}
                {this.readingBook.author ? "•" : ""}
                {this.readingBook.author}
              </div>
            </div>
          </div>
        </div>
      );
    },
    renderVideo() {
      return <div ref="dplayer"></div>;
    },
    initDplayer() {
      if (this.$refs.dplayer) {
        if (this.dplayer) {
          this.dplayer.destroy();
        }
        // 视频内容为 JSON 配置：{"video":{"url":"..."},"danmaku":{"id":"...","api":"..."},"subtitle":{...}}
        // 或纯视频地址
        let options = { video: { url: this.content } };
        try {
          options = JSON.parse(this.content);
          if (typeof options === "string") {
            options = { video: { url: this.content } };
          }
        } catch (error) {
          //
        }
        this.dplayer = new DPlayer({
          container: this.$refs.dplayer,
          ...options,
          preventClickToggle: true,
          autoplay: true
        });
        this.dplayer.on("durationchange", () => {
          if (this.startTime) {
            this.dplayer.seek(this.startTime);
            this.startTime = null;
          }
        });
        this.dplayer.on("timeupdate", () => {
          this.currentTime = this.dplayer.video.currentTime | 0;
          this.$emit("updateProgress");
        });
      } else {
        setTimeout(() => {
          this.initDplayer();
        }, 50);
      }
    },
    renderEpub() {
      if (!this.content.startsWith("/book-assets")) {
        return null;
      }
      return (
        <ShadowIframe
          src={this.$store.getters.apiRoot + this.content}
          renderType={this.isEpubIframe ? "iframe" : "shadowDom"}
        />
      );
    },
    iframeEvent(event, data) {
      switch (event) {
        case "inited":
          this.iframeStyle = {};
          this.setIframeStyle();
          if (this.isSlideRead) {
            this.syncIframeWidth();
          } else {
            this.syncIframeHeight();
          }
          break;
        case "load":
          setTimeout(() => {
            this.$emit("iframeLoad");
            this.$emit("epubLocationChange", data);
          }, 100);
          break;
        case "setHeight":
          this.iframeSize.scrollHeight = data;
          if (!this.isSlideRead) {
            this.iframeStyle = {
              ...this.iframeStyle,
              height: Math.max(data, this.windowSize.height * 0.8) + "px"
            };
            this.$emit("contentChange");
          }
          break;
        case "setWidth":
          this.iframeSize.scrollWidth = data;
          if (this.isSlideRead) {
            this.$emit("contentChange");
          }
          break;
        case "click":
          this.$emit("epubClick", data);
          break;
        case "clickHash":
          this.$emit("epubClickHash", data);
          break;
        case "keydown":
          this.$emit("epubKeydown", data);
          break;
        case "previewImageList":
          this.$store.commit("setPreviewImageIndex", data.imageIndex);
          this.$store.commit("setPreviewImgList", data.imageList);
          break;
        case "clickA":
          this.$emit("epubLocationChange", data);
          break;
        case "touchstart":
        case "touchmove":
        case "touchend":
          this.$emit("epubTouch", { event, data });
          break;
        default:
          break;
      }
    },
    initIframe() {
      window.addEventListener("message", event => {
        if (
          this.$refs.iframe &&
          event.source === this.$refs.iframe.contentWindow
        ) {
          let message;
          try {
            message = JSON.parse(event.data);
          } catch (error) {
            return;
          }
          this.iframeEvent(message.event, message.data);
        }
      });
    },
    syncIframeHeight() {
      this.ensure(() => {
        this.$refs.iframe.syncIframeHeight();
      }, () => this.$refs.iframe);
    },
    syncIframeWidth() {
      this.ensure(() => {
        this.$refs.iframe.syncIframeWidth();
      }, () => this.$refs.iframe);
    },
    ensure(callback, check) {
      if (check && check()) {
        callback && callback();
      } else {
        setTimeout(() => {
          this.ensure(callback, check);
        }, 10);
      }
    },
    transformX(x) {
      this.ensure(() => {
        this.$refs.iframe.transformX(x);
      }, () => this.$refs.iframe);
    },
    syncCSSProperty() {
      this.ensure(() => {
        this.$refs.iframe.syncCSSProperty();
      }, () => this.$refs.iframe);
    },
    setIframeStyle() {
      if (!this.$refs.iframe) {
        setTimeout(() => {
          this.setIframeStyle();
        }, 10);
        return;
      }
      let bodyStyle = "";
      for (const i in this.containerStyle) {
        if (Object.hasOwnProperty.call(this.containerStyle, i)) {
          bodyStyle +=
            i.replace(/([A-Z])/g, v => "-" + v.toLowerCase()) +
            ":" +
            this.containerStyle[i] +
            " !important;";
        }
      }
      let pStyle = "";
      for (const i in this.pStyle) {
        if (Object.hasOwnProperty.call(this.pStyle, i)) {
          pStyle +=
            i.replace(/([A-Z])/g, v => "-" + v.toLowerCase()) +
            ":" +
            this.pStyle[i] +
            " !important;";
        }
      }
      pStyle +=
        "font-family: " + this.containerStyle.fontFamily + " !important;";
      pStyle += "font-size: " + this.containerStyle.fontSize + " !important;";
      pStyle +=
        "font-weight: " + this.containerStyle.fontWeight + " !important;";
      pStyle += "color: " + this.containerStyle.color + " !important;";
      let imgStyle = "";
      if (this.isSlideRead) {
        imgStyle =
          "\n          height: 100%;\n          " +
          "columns: calc(100vw - 16px - var(--horizontal-padding, 0px)) 1;\n" +
          "          column-gap: calc(16px + var(--horizontal-padding, 0px));\n        ";
      }
      let imgRules = "";
      if (this.isSlideRead) {
        imgRules = "\n          break-inside: avoid;\n        ";
      }
      this.sendToIframe("setStyle", {
        style: `
        *::-webkit-scrollbar {
          display: none;
          width: 0 !important;
          height: 0 !important;
        }
        *:focus {
          outline: none !important;
        }
        html {
          ${this.isSlideRead ? "height: 100%;" : ""}
          min-height: 100%;
        }
        body {
          margin: 0 !important;
          ${bodyStyle}
        }
        .reading {
          color: red !important;
        }
        body p {
          ${pStyle}
        }
        body h1, body h2, body h3, body h4 {
          font-family: ${this.containerStyle.fontFamily} !important;
          font-weight: ${this.containerStyle.fontWeight} !important;
          color: ${this.containerStyle.color} !important;
        }
        img, body img {
          display: block;
          max-width: ${this.isSlideRead ? "100vw" : "100%"} !important;
          height: auto !important;
          ${imgStyle}
          ${imgRules}
        }`
      });
    },
    sendToIframe(event, data) {
      if (!this.$refs.iframe) {
        setTimeout(() => {
          this.sendToIframe(event, data);
        }, 10);
        return;
      }
      this.$refs.iframe &&
        this.$refs.iframe.contentWindow &&
        this.$refs.iframe.contentWindow.postMessage(
          JSON.stringify({
            event,
            ...data
          }),
          "*"
        );
    },
    formatTime(val) {
      if (!val) {
        return "00:00";
      }
      const pad = v => (v >= 10 ? "" + v : "0" + v);
      if (val < 60) {
        return "00:" + pad(val);
      } else if (val < 3600) {
        const m = Math.round(val / 60);
        const s = val % 60;
        return pad(m) + ":" + pad(s);
      } else {
        const h = Math.round(val / 3600);
        const m = Math.round(val / 3600 / 60);
        const s = val % 60;
        return pad(h) + ":" + pad(m) + ":" + pad(s);
      }
    },
    seekTime(val) {
      if (!isNaN(val) && val !== Infinity) {
        if (this.$refs.audio) {
          this.$refs.audio.currentTime = val;
        }
      }
    },
    setAudioVolume(val) {
      if (!isNaN(val) && val !== Infinity) {
        this.audioVolume = val;
        if (this.$refs.audio) {
          this.$refs.audio.volume = val / 100;
        }
      }
    },
    ensureSeekTime(val) {
      this.startTime = val;
    },
    toggle() {
      if (this.playing) {
        this.$refs.audio && this.$refs.audio.pause();
      } else {
        this.play();
      }
    },
    play(init) {
      if (!this.$refs.audio) {
        setTimeout(() => {
          this.play(init);
        }, 10);
        return;
      }
      if (init) {
        this.$refs.audio.load();
        this.computeDuration();
      }
      if (!init || this.autoPlay) {
        this.$refs.audio.play();
      }
    },
    prevChapter() {
      this.autoPlay = true;
      this.$emit("prevChapter");
    },
    nextChapter() {
      this.autoPlay = true;
      this.$emit("nextChapter");
    },
    computeDuration() {
      if (!this.$refs.audio) {
        setTimeout(() => {
          this.computeDuration();
        }, 100);
        return;
      }
      let duration = this.$refs.audio.duration;
      if (
        this.$refs.audio.readyState >= 1 &&
        !isNaN(duration) &&
        duration !== Infinity &&
        duration
      ) {
        this.audioDuration = parseInt(duration);
        this.$refs.audio.playbackRate = this.currentSpeed;
        this.$refs.audio.currentTime = this.startTime;
        // 有时会失败（看浏览器）
        if (this.autoPlay) {
          this.$refs.audio.play();
        }
      } else {
        setTimeout(() => {
          this.computeDuration();
        }, 50);
      }
    },
    onProgress() {
      // 记录缓存进度。触发事件包括缓存数据更新时的 progress 事件，以及各种播放动作会触发的 playing 事件
    },
    onTimeupdate() {
      if (this.$refs.audio) {
        this.currentTime = this.$refs.audio.currentTime | 0;
      }
      this.$emit("updateProgress");
    },
    onPlay() {
      this.playing = true;
    },
    onPause() {
      this.playing = false;
    },
    onEnd() {
      this.playing = false;
      this.currentTime = 0;
      this.audioDuration = 0;
      this.autoPlay = true;
      this.$emit("nextChapter");
    },
    onError(event) {
      // console.log(arguments);
      this.$message.error(event.toString());
      this.playing = false;
    },
    onSeeked() {},
    onSeeking() {},
    audioEvent() {
      // console.log("audioEvent", arguments);
    },
    onsuspend() {
      // console.log("onsuspend", arguments);
    },
    onCanPlay() {
      // console.log("onCanPlay", arguments);
    },
    onWaiting() {
      // console.log("onWaiting", arguments);
    },
    loadCustomFontFamil() {
      if (this.currentCustomFontFamily) {
        loadFont(
          this.currentCustomFontFamily.name,
          this.currentCustomFontFamily.url
        );
      }
    }
  }
};

export const stylusStyleScoped = `
p {
  display: block;
  word-wrap: break-word;
  word-break: break-all;
  text-indent: 2em;
}
p.reading {
  color: red !important;
}
h3 {
  font-size: 28px;
  line-height: 1.2;
  margin: 1em 0;
  text-align: center;
}
h3.reading {
  color: red !important;
}
.volume-chapter {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;

  .volume-content {
    text-align: center;
  }

  .volume-tag {
    text-align: right;
  }
}
.content-audio {
  margin: 0 auto;
  width: 100%;

  .book-cover {

    img {
      max-width: 200px;
      margin: 0 auto;
      display: block;
    }
  }

  .book-progress {
    padding: 25px 15px;
    display: flex;
    flex-direction: row;
    align-items: center;

    .progress-tip {
      padding-top: 5px;
      padding-bottom: 5px;
      font-size: 14px;
      width: 45px;
    }

    .progress-container {
      flex: 1;
      margin-left: 10px;
      margin-right: 10px;
    }

    .total-time {
      text-align: right;
    }
  }

  .book-operation {
    padding: 0px 15px 25px;
    display: flex;
    flex-direction: row;
    justify-content: space-around;

    i {
      display: inline-block;
      cursor: pointer;
      font-size: 24px;
      line-height: 1;
    }
  }

  .book-info {
    padding: 10px 15px;
    display: flex;
    flex-direction: row;
    align-items: center;

    .book-cover {
      width: 50px;

      img {
        width: 100%;
        max-height: 100%;
      }
    }

    .book-intro {
      flex: 1;
      padding-left: 15px;

      .title {
        font-size: 16px;
      }

      .subtitle {
        margin-top: 5px;
        font-size: 14px;
      }
    }
  }
}
.epub-iframe {
  border: none;
  width: 100%;
  min-height: calc(var(--vh, 1vh) * 50);
  // pointer-events: none;
}
`;

export const stylusStyle = `
.content-body {
  img {
    width: 100%;
    max-width: 100vw;
    display: block;
  }
}
.day {
  .content-audio {
    .book-operation {
      color: #222;
    }

    .book-intro {
      .title {
        color: #121212;
      }
      .subtitle {
        color: #666;
      }
    }
  }
}
.night {
  .content-audio {
    .book-operation {
      color: #888;
    }

    .book-intro {
      .title {
        color: #888;
      }
      .subtitle {
        color: #666;
      }
    }
  }
}
`;
