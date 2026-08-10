export const template = `  <div
    class="chapter-wrapper"
    :style="bodyTheme"
    :class="{
      night: isNight,
      day: !isNight,
      'mini-interface': $store.state.miniInterface
    }"
    ref="chapterWrapperRef"
  >
    <div class="tool-bar" :style="leftBarTheme">
      <div class="tools">
        <div
          class="tool-icon"
          @click="toShelf"
          :style="$store.state.miniInterface ? { order: -1 } : {}"
        >
          <div class="iconfont">
            &#58920;
          </div>
          <div class="icon-text">首页</div>
        </div>
        <el-popover
          placement="right"
          :width="popperWidth"
          trigger="click"
          :visible-arrow="false"
          v-model="popBookShelfVisible"
          popper-class="popper-component"
        >
          <BookShelf
            ref="popBookShelf"
            class="popup"
            :visible="popBookShelfVisible"
            @changeBook="changeBook"
            @toShelf="toShelf"
          />
          <div class="tool-icon" slot="reference">
            <div class="iconfont">
              &#58892;
            </div>
            <div class="icon-text">书架</div>
          </div>
        </el-popover>
        <el-popover
          placement="right"
          :width="popperWidth"
          trigger="click"
          :visible-arrow="false"
          v-model="popBookSourceVisible"
          popper-class="popper-component"
        >
          <BookSource
            ref="popBookSource"
            class="popup"
            :visible="popBookSourceVisible"
            @changeBookSource="changeBookSource()"
            @close="popBookSourceVisible = false"
          />

          <div class="tool-icon" slot="reference">
            <div class="tool-el-icon">
              <i class="el-icon-menu"></i>
            </div>
            <div class="icon-text">书源</div>
          </div>
        </el-popover>
        <el-popover
          placement="right"
          :width="popperWidth"
          trigger="click"
          :visible-arrow="false"
          v-model="popCataVisible"
          popper-class="popper-component"
        >
          <PopCata
            @gotoChapter="gotoChapter"
            ref="popCata"
            class="popup"
            @refresh="refreshCatalog"
            :visible="popCataVisible"
            @close="popCataVisible = false"
          />

          <div class="tool-icon" slot="reference">
            <div class="iconfont">
              &#58905;
            </div>
            <div class="icon-text">目录</div>
          </div>
        </el-popover>
        <el-popover
          placement="right"
          :width="popperWidth"
          trigger="click"
          :visible-arrow="false"
          v-model="readSettingsVisible"
          popper-class="popper-component"
        >
          <ReadSettings
            class="popup"
            :visible="readSettingsVisible"
            @close="readSettingsVisible = false"
            @showClickZone="showClickZone = true"
            @readMethodChange="beforeReadMethodChange"
          />

          <div class="tool-icon" slot="reference">
            <div class="iconfont">
              &#58971;
            </div>
            <div class="icon-text">设置</div>
          </div>
        </el-popover>
        <div
          class="tool-icon"
          @click="toTop(0)"
          v-if="!$store.state.miniInterface"
        >
          <div class="iconfont">
            &#58914;
          </div>
          <div class="icon-text">顶部</div>
        </div>
        <div
          class="tool-icon"
          @click="toBottom(0)"
          v-if="!$store.state.miniInterface"
        >
          <div class="iconfont">
            &#58915;
          </div>
          <div class="icon-text">底部</div>
        </div>
      </div>
    </div>
    <div class="read-bar" :style="rightBarTheme">
      <div class="float-btn-zone">
        <div class="float-left-btn-zone">
          <div
            class="float-btn"
            :style="popupAbsoluteBtnStyle"
            @click="showBookmarkDialog"
            v-if="!isAudio && !isVideo"
          >
            <i class="el-icon-collection-tag"></i>
          </div>
          <div
            class="float-btn"
            :style="popupAbsoluteBtnStyle"
            @click="showSearchBookContentDialog"
            v-if="!isAudio && !isVideo"
          >
            <i class="el-icon-search"></i>
          </div>
          <div
            class="float-btn"
            :style="popupAbsoluteBtnStyle"
            @click="showReadingBookInfo"
          >
            <i class="el-icon-info"></i>
          </div>
          <div
            class="float-btn"
            :style="popupAbsoluteBtnStyle"
            @click="toTop(0)"
            v-if="
              $store.state.miniInterface && !isSlideRead && !isAudio && !isVideo
            "
          >
            <i class="el-icon-top"></i>
          </div>
          <div
            class="float-btn"
            :style="popupAbsoluteBtnStyle"
            @click="toBottom(0)"
            v-if="
              $store.state.miniInterface && !isSlideRead && !isAudio && !isVideo
            "
          >
            <i class="el-icon-bottom"></i>
          </div>
        </div>
        <div class="float-right-btn-zone">
          <div
            class="float-btn"
            :style="popupAbsoluteBtnStyle"
            @click="toggleEditContent"
            v-if="!isAudio && !isVideo && !isEpub && !isCbz && !isPdf"
            :class="isEditContent ? 'editing' : ''"
          >
            <i class="el-icon-edit"></i>
          </div>
          <div
            class="float-btn"
            :style="popupAbsoluteBtnStyle"
            @click="showCacheContent"
            v-if="!isAudio && !isVideo && !isEpub"
          >
            <i class="el-icon-download"></i>
          </div>
          <div
            class="float-btn"
            :style="popupAbsoluteBtnStyle"
            @click="readOriginal"
            v-if="isPdf"
          >
            <i class="el-icon-reading"></i>
          </div>
          <div
            class="float-btn"
            :style="popupAbsoluteBtnStyle"
            @click="refreshContent"
          >
            <i class="el-icon-refresh-right"></i>
          </div>
          <div
            class="float-btn"
            :style="popupAbsoluteBtnStyle"
            @click="toggleAutoReading()"
            v-if="!isAudio && !isVideo"
            :class="autoReading ? 'auto-reading' : ''"
          >
            <i class="el-icon-view"></i>
          </div>
          <div
            class="float-btn"
            :style="popupAbsoluteBtnStyle"
            @click="showReadBar = !showReadBar"
            v-if="!isAudio && !isVideo"
          >
            <i class="el-icon-headset"></i>
          </div>
          <div
            class="float-btn"
            :style="popupAbsoluteBtnStyle"
            @click="toogleNight"
          >
            <i class="el-icon-moon" v-if="!isNight"></i>
            <i class="el-icon-sunny" v-else></i>
          </div>
        </div>
      </div>
      <div
        class="progress"
        v-if="$store.state.miniInterface && !isAudio && !isVideo"
      >
        <div class="progress-bar">
          <el-slider
            v-model="currentPage"
            :min="1"
            :max="totalPages"
            :show-tooltip="false"
            @change="showPage"
            @input="progressValue = $event"
          ></el-slider>
        </div>
        <span class="progress-tip">{{ formatProgressTip() }}</span>
      </div>
      <div class="cache-content-zone" v-if="showCacheContentZone">
        <div>
          缓存章节
        </div>
        <div
          class="cache-content-btn"
          v-show="!isCachingContent"
          @click="cacheChapterContent(50)"
        >
          后面50章
        </div>
        <div
          class="cache-content-btn"
          v-show="!isCachingContent"
          @click="cacheChapterContent(100)"
        >
          后面100章
        </div>
        <div
          class="cache-content-btn"
          v-show="!isCachingContent"
          @click="cacheChapterContent(true)"
        >
          后面全部
        </div>
        <div class="caching-tip" v-show="isCachingContent">
          {{ cachingContentTip }}
        </div>
        <div
          class="caching-cancel-btn"
          v-show="isCachingContent"
          @click="cancelCaching"
        >
          <i class="el-icon-close"></i>
        </div>
      </div>
      <div class="tools">
        <div class="tool-icon progress-text" @click="showCacheContent">
          <span v-if="$store.state.miniInterface">阅读进度: </span>
          {{ readingProgress }}
        </div>
        <div
          class="tool-icon"
          @click="toLastChapter()"
          :style="$store.state.miniInterface ? { order: -1 } : {}"
        >
          <div class="iconfont">
            &#58920;
          </div>
          <span v-if="$store.state.miniInterface">上一章</span>
        </div>
        <div class="tool-icon" @click="toNextChapter()">
          <span v-if="$store.state.miniInterface">下一章</span>
          <div class="iconfont">
            &#58913;
          </div>
        </div>
      </div>
    </div>
    <div class="read-bar" :style="readBarTheme">
      <div class="reader-bar-inner">
        <div class="operate-bar">
          <div class="close-btn" @click="exitRead">
            <i class="el-icon-close"></i>
          </div>
          <div class="center">
            <span class="ctrl-btn" @click="speechPrev">上一段</span>
            <span class="play-pause-btn" @click="toggleSpeech">
              <i
                class="el-icon-video-pause"
                :style="popupAbsoluteBtnStyle"
                v-if="speechSpeaking"
              ></i>
              <i
                class="el-icon-video-play"
                :style="popupAbsoluteBtnStyle"
                v-else
              ></i>
            </span>
            <span class="ctrl-btn" @click="speechNext">下一段</span>
          </div>
          <div
            class="collapse-btn"
            @click="showSpeechConfig = !showSpeechConfig"
          >
            <i class="el-icon-bottom" v-if="showSpeechConfig"></i>
            <i class="el-icon-top" v-else></i>
          </div>
        </div>
        <div class="setting-item" v-if="showSpeechConfig">
          <div class="setting-title">
            朗读方式
            <span class="float-right" @click="showHttpTTSDialog"
              >HttpTTS 管理</span
            >
          </div>
          <div class="setting-value">
            <div class="voice-list">
              <el-select
                class="voice-select"
                v-model="ttsType"
                size="mini"
                filterable
                placeholder="请选择朗读方式"
              >
                <el-option
                  v-for="(item, index) in ttsTypeList"
                  :key="'tts-type-' + index"
                  :label="item.name"
                  :value="item.value"
                ></el-option>
              </el-select>
            </div>
          </div>
          <div class="setting-oneline" v-if="ttsType !== 'local'">
            <span class="setting-title">连读优化：</span>
            <el-switch
              v-model="cacheTTSAudio"
              active-color="#13ce66"
              inactive-color="#ff4949"
              :active-value="true"
              :inactive-value="false"
            ></el-switch>
          </div>
          <div class="setting-title">语音库</div>
          <div class="setting-value">
            <div class="voice-list">
              <el-select
                class="voice-select"
                v-model="voiceName"
                size="mini"
                filterable
                placeholder="请选择语音库"
              >
                <el-option
                  v-for="(voice, index) in voiceList"
                  :key="'search-type-' + index"
                  :label="voice.LocalName || voice.name"
                  :value="voice.name"
                ></el-option>
              </el-select>
            </div>
          </div>
        </div>
        <div class="setting-item" v-if="showSpeechConfig">
          <div class="setting-title">语音设置</div>
          <div class="setting-value">
            <div class="progress">
              <span class="progress-tip">语速</span>
              <div class="progress-bar">
                <el-slider
                  v-model="speechRate"
                  :min="0.5"
                  :max="2"
                  :step="0.1"
                  :show-tooltip="false"
                  @change="changeSpeechRate"
                ></el-slider>
              </div>
              <span class="setting-btn" @click="changeSpeechRate(1)">重置</span>
            </div>
            <div class="progress" v-show="ttsType !== 'httpTTS'">
              <span class="progress-tip">语调</span>
              <div class="progress-bar">
                <el-slider
                  v-model="speechPitch"
                  :min="0"
                  :max="2"
                  :step="0.1"
                  :show-tooltip="false"
                  @change="changeSpeechPitch"
                ></el-slider>
              </div>
              <span class="setting-btn" @click="changeSpeechPitch(1)"
                >重置</span
              >
            </div>
            <div class="progress">
              <span class="progress-tip">定时</span>
              <div class="progress-bar">
                <el-slider
                  v-model="speechMinutes"
                  :min="0"
                  :max="180"
                  :step="1"
                  :show-tooltip="false"
                  @change="changeSpeechMinutes"
                ></el-slider>
              </div>
              <span class="setting-btn">{{ speechMinutes }}分钟</span>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div
      class="chapter"
      ref="content"
      :class="chapterClass"
      :style="chapterTheme"
    >
      <div
        class="click-zone"
        v-if="showClickZone"
        :style="!isSlideRead ? { position: 'fixed' } : {}"
      >
        <div :style="showPrevPageStyle"><span>点击前一页</span></div>
        <div :style="showMenuZoneStyle"><span>点击显示菜单</span></div>
        <div :style="showNextPageStyle"><span>点击后一页</span></div>
        <div class="close-btn" @click="showClickZone = false">关闭</div>
        <div :style="showPrevPageExtraStyle"></div>
        <div :style="showNextPageExtraStyle"></div>
      </div>
      <div class="top-bar" ref="top">
        {{ $store.state.miniInterface ? title : "" }}
        <span
          class="right-timestr"
          v-if="$store.state.miniInterface && !isSlideRead"
          >{{ timeStr }}</span
        >
      </div>
      <div
        class="content"
        @touchstart="handleTouchStart"
        @touchmove="handleTouchMove"
        @touchend="handleTouchEnd"
        @click="handlerClick"
      >
        <div class="content-inner" v-if="show" :style="contentInnerStyle">
          <Content
            class="book-content"
            :title="title"
            :content="content"
            :showContent="show"
            :error="error"
            :style="contentStyle"
            :showChapterList="showChapterList"
            :isScrollRead="isScrollRead"
            :isSlideRead="isSlideRead"
            :isEditContent="isEditContent"
            ref="bookContentRef"
            @prevChapter="toLastChapter"
            @nextChapter="toNextChapter"
            @updateProgress="saveReadingPosition"
            @iframeLoad="$emit('iframeLoad')"
            @contentChange="onContentChange"
            @epubClick="eventHandler"
            @epubLocationChange="epubLocationChangeHandler"
            @epubClickHash="epubClickHash"
            @epubKeydown="keydownHandler($event, true)"
            @epubTouch="epubTouch"
          />
        </div>
      </div>
      <div class="bottom-bar" ref="bottom">
        <audio
          ref="ttsAudio"
          preload="preload"
          src
          autoplay
          @ended="onTTSAudioEnded"
          @error="onTTSAudioError"
          @play="onTTSAudioPlay"
          @pause="onTTSAudioPause"
        ></audio>
        <span v-if="isSlideRead">{{
          \`第\${currentPage}/\${totalPages}页 \${readingProgress}\`
        }}</span>
        <span v-if="isSlideRead">{{ timeStr }}</span>
        <span
          class="bottom-btn"
          v-if="show && !isSlideRead && !error && !isScrollRead"
          @click="toNextChapter()"
          >加载下一章</span
        >
      </div>
    </div>
  </div>`;

import PopCata from "../components/PopCatalog.vue";
import ReadSettings from "../components/ReadSettings.vue";
import BookSource from "../components/BookSource.vue";
import BookShelf from "../components/BookShelf.vue";
import Content from "../components/Content.vue";
import Axios from "../plugins/axios";
import jump from "../plugins/jump";
import Animate from "../plugins/animate";
import { setCache, getCache } from "../plugins/cache";
import { simplized, traditionalized } from "../plugins/chinese";
import { ttsVoiceList } from "../plugins/ttsVoices";
import { isOnlyWhitespace, stripWhitespace } from "../plugins/ttsWhitespace";
import {
  cacheFirstRequest,
  LimitResquest,
  editDistance
} from "../plugins/helper";
import { defaultReplaceRule, defaultBookmark } from "../plugins/config.js";
import eventBus from "../plugins/eventBus";
// eslint-disable-next-line no-useless-escape
const symboRegex = /[\u2000-\u206F\u2E00-\u2E7F\\'!"#$%&\(\)*+,-\./:;<=>?@\[\]^_`{\|}~，。？《》；：、«]/g;

export const Reader = {
  components: {
    PopCata,
    BookSource,
    BookShelf,
    Content,
    ReadSettings
  },
  mounted() {
    window.readerPage = this;
    this.speechAvalable =
      window.speechSynthesis && window.speechSynthesis.getVoices;
    if (this.speechAvalable) {
      this.fetchVoiceList();
      if (window.speechSynthesis.onvoiceschanged !== undefined) {
        window.speechSynthesis.onvoiceschanged = this.fetchVoiceList;
      }
    }
    window.addEventListener("unload", this.saveReadingPosition);
    eventBus.$on("showSearchContent", data => {
      if (this._inactive) {
        return;
      }
      if (this.chapterIndex === data.chapterIndex) {
        this.showMatchKeyword(data);
        return;
      }
      if (this.isScrollRead) {
        this.scrollStartChapterIndex = data.chapterIndex;
        this.computeShowChapterList().then(() => {
          this.showMatchKeyword(data);
        });
        return;
      }
      this.onNextContentRendered(() => {
        this.$nextTick(() => {
          this.showMatchKeyword(data);
        });
      });
      this.getContent(data.chapterIndex);
    });
    eventBus.$on("showBookmark", bookmark => {
      if (this._inactive) {
        return;
      }
      if (this.chapterIndex === bookmark.chapterIndex) {
        this.showBookmark(bookmark);
        return;
      }
      if (this.isScrollRead) {
        this.scrollStartChapterIndex = bookmark.chapterIndex;
        this.computeShowChapterList().then(() => {
          this.showBookmark(bookmark);
        });
        return;
      }
      this.onNextContentRendered(() => {
        this.$nextTick(() => {
          this.showBookmark(bookmark);
        });
      });
      this.getContent(bookmark.chapterIndex);
    });
    // 书签弹窗的"添加"按钮
    eventBus.$on("addBookmark", () => {
      if (!this._inactive) {
        this.addBookmark();
      }
    });
  },
  activated() {
    this.init();
    window.addEventListener("keydown", this.keydownHandler);
    if (this.title) {
      document.title =
        this.$store.getters.readingBook.name + " - " + this.title;
    } else {
      document.title = this.$store.getters.readingBook.name;
    }
    this.formatTime();
    this.timer = setInterval(() => {
      this.formatTime();
    }, 5000);
    this.unwatchFn = this.$store.watch(
      state => state.config,
      () => {
        this.$nextTick(() => {
          this.computePages(() => {
            if (this.currentPage > this.totalPages) {
              this.showPage(this.totalPages, 0);
            }
          });
        });
      },
      {
        deep: true
      }
    );
    window.addEventListener("scroll", this.scrollHandler);
    try {
      this.releaseWakeLockFn = this.wakeLock();
    } catch (e) {
      //
    }
    this.$Lazyload.$on("loaded", this.lazyloadHandler);
    this.setMobileScrollBarHidden(this.shouldHideMobileScrollBar);
  },
  deactivated() {
    this.saveBookProgress();
    this.startSavePosition = false;
    this.lastReadingBook = this.$store.getters.readingBook;
    this.timer && clearInterval(this.timer);
    window.removeEventListener("keydown", this.keydownHandler);
    window.removeEventListener("scroll", this.scrollHandler);
    this.unwatchFn && this.unwatchFn();
    this.releaseWakeLockFn && this.releaseWakeLockFn();
    this.$Lazyload.$off("loaded", this.lazyloadHandler);
    this.setMobileScrollBarHidden(false);
  },
  beforeDestroy() {
    this.setMobileScrollBarHidden(false);
  },
  watch: {
    chapterName(to) {
      this.title = to;
    },
    content() {
      this.contentStyle = {};
      this.transformX = 0;
      this.currentPage = 1;
      this.$nextTick(() => {
        this.computePages();
        this.saveReadingPosition();
      });
      if (this.isEpub) {
        this.$once("iframeLoad", () => {
          this.computePages();
        });
      }
    },
    readSettingsVisible() {
      //
    },
    title(title) {
      if (title) {
        document.title = this.$store.getters.readingBook.name + " - " + title;
      } else {
        document.title = this.$store.getters.readingBook.name;
      }
    },
    isSlideRead(val) {
      if (!val) {
        this.contentStyle = {};
        this.transformX = 0;
        if (this.isEpub) {
          this.$refs.bookContentRef.transformX(0);
        }
      }
      this.$nextTick(() => {
        this.computePages(() => {
          if (this.currentParagraph) {
            this.showParagraph(this.currentParagraph, true);
          } else {
            this.showPage(this.currentPage, 0);
          }
        });
      });
    },
    isScrollRead(val) {
      if (val) {
        this.scrollStartChapterIndex = this.chapterIndex;
        this.computeShowChapterList();
      }
    },
    windowSize() {
      // 在移动端滚动模式下，地址栏显示/隐藏会导致 innerHeight 变化
      // 但我们使用固定的 --vh 变量，所以不需要重新计算页面
      if (this.isScrollRead && this.$store.state.miniInterface) {
        return;
      }
      this.$nextTick(() => {
        this.computePages(() => {
          this.showPage(this.currentPage, 0);
        });
      });
    },
    loginAuth(val) {
      if (val) {
        this.init(true);
      }
    },
    showReadBar(val) {
      if (val) {
        this.showToolBar = false;
      }
    },
    readingBook(val, oldVal) {
      if (val && val.bookUrl !== (oldVal || {}).bookUrl) {
        this.startSavePosition = false;
        this.autoShowPosition();
      }
    },
    currentPage(val, oldVal) {
      // 还剩两页的时候，预读下一章节
      const readingBook = this.$store.getters.readingBook || {};
      if (val !== oldVal && val >= this.totalPages - 2 && readingBook.catalog) {
        if (readingBook.index < readingBook.catalog.length - 1) {
          if (!this.isScrollRead) {
            if (!this.preCaching) {
              this.preCaching = true;
              this.getBookContent(
                this.$store.getters.readingBook.index + 1,
                {
                  timeout: 1000 * this.$store.getters.config.chapterRequestTimeout,
                  silent: true
                },
                false,
                true
              ).then(() => {
                this.preCaching = false;
              });
            }
          }
        }
      }
    },
    filterRules() {
      if (this.isScrollRead) {
        this.computeShowChapterList();
      } else {
        this.content = this.filterContent(this.originalContent);
      }
    },
    chineseFont() {
      this.title = this.filterContent(this.title);
      this.content = this.filterContent(this.originalContent);
      this.computeShowChapterList();
    },
    shouldHideMobileScrollBar(val) {
      this.setMobileScrollBarHidden(val);
    }
  },
  data() {
    return {
      title: "",
      originalContent: "",
      content: "",
      error: false,
      popCataVisible: false,
      readSettingsVisible: false,
      popBookSourceVisible: false,
      popBookShelfVisible: false,
      showToolBar: true,
      book: null,
      show: false,
      contentStyle: {},
      currentPage: 1,
      totalPages: 1,
      transformX: 0,
      transforming: false,
      showLastPage: false,
      showClickZone: false,
      timeStr: "",
      progressValue: 1,

      speechAvalable: false,
      showReadBar: false,
      localVoiceList: [],
      speechSpeaking: false,
      showSpeechConfig: true,
      ttsTypeList: [
        { name: "浏览器", value: "local" },
        { name: "大声朗读", value: "edge" },
        { name: "HttpTTS", value: "httpTTS" }
      ],
      currentTTSCacheKey: null,

      currentParagraph: null,

      startSavePosition: false,

      showCacheContentZone: false,
      isCachingContent: false,
      cachingContentTip: "",

      autoReading: false,
      showChapterList: [],

      scrollStartChapterIndex: 0,
      showNextChapterSize: 1,
      showPrevChapterSize: 0,

      speechMinutes: 0,
      speechEndTime: 0,
      isEditContent: false,
      contentInnerStyle: {}
    };
  },
  computed: {
    readingBook() {
      return this.$store.getters.readingBook || {};
    },
    catalog() {
      return (this.$store.getters.readingBook || {}).catalog || [];
    },
    chapterIndex() {
      return ((this.$store.getters.readingBook || {}).index || 0) | 0;
    },
    windowSize() {
      return this.$store.state.windowSize;
    },
    config() {
      return this.$store.getters.config;
    },
    theme() {
      return this.config.theme;
    },
    animateMSTime() {
      return this.config.animateMSTime;
    },
    isNight() {
      return this.$store.getters.isNight;
    },
    bodyTheme() {
      return {
        background: this.$store.getters.currentThemeConfig.body
      };
    },
    isSlideRead() {
      return this.autoReading ||
        this.showReadBar ||
        this.isVideo ||
        this.isAudio
        ? false
        : this.$store.getters.isSlideRead;
    },
    isScrollRead() {
      return (
        !this.isEpub &&
        !this.isAudio &&
        !this.isVideo &&
        !this.isSlideRead &&
        (this.config.readMethod === "上下滚动" ||
          this.config.readMethod === "上下滚动2")
      );
    },
    shouldHideMobileScrollBar() {
      return this.isScrollRead && this.$store.state.miniInterface;
    },
    chapterClass() {
      if (this.isSlideRead) {
        return "slide-reader";
      }
      if (this.isEpub) {
        return this.config.epubMode === "iframe" ? "epub-iframe" : "epub-dom";
      }
      if (this.isCarToon) {
        return "cartoon";
      }
      if (this.isAudio) {
        return "audio";
      }
      if (this.isVideo) {
        return "video";
      }
      return "";
    },
    chapterTheme() {
      let readingStyle = this.showReadBar
        ? { paddingBottom: (this.showSpeechConfig ? 280 : 80) + "px" }
        : {};
      if (typeof this.$store.getters.currentThemeConfig.content === "string") {
        return {
          ...readingStyle,
          background: this.$store.getters.currentThemeConfig.content,
          width: this.readWidth
        };
      } else {
        return {
          ...readingStyle,
          ...this.$store.getters.currentThemeConfig.content,
          width: this.readWidth
        };
      }
    },
    leftBarTheme() {
      return {
        background: this.$store.getters.currentThemeConfig.popup,
        marginLeft: this.$store.state.miniInterface
          ? 0
          : -(this.readWidthConfig / 2 + 68) + "px",
        display:
          this.$store.state.miniInterface && !this.showToolBar
            ? "none"
            : "block"
      };
    },
    rightBarTheme() {
      return {
        background: this.$store.getters.currentThemeConfig.popupPure,
        marginRight: this.$store.state.miniInterface
          ? 0
          : -(this.readWidthConfig / 2 + 52) + "px",
        display:
          this.$store.state.miniInterface && !this.showToolBar
            ? "none"
            : "block"
      };
    },
    readBarTheme() {
      return {
        background: this.$store.getters.currentThemeConfig.popupPure,
        marginRight: this.$store.state.miniInterface
          ? 0
          : -(this.readWidthConfig / 2) + "px",
        zIndex: 200,
        display:
          (this.speechAvalable || this.ttsType !== "local") && this.showReadBar
            ? "block"
            : "none",
        width: this.$store.state.miniInterface ? "100vw" : "500px"
      };
    },
    readWidth() {
      if (!this.$store.state.miniInterface) {
        return this.readWidthConfig - 130 + "px";
      } else {
        return this.windowSize.width + "px";
      }
    },
    readWidthConfig() {
      var width = this.$store.getters.config.readWidth;
      while (width > this.$store.state.windowSize.width - 140) {
        width -= 20;
      }
      return width;
    },
    popperWidth() {
      if (!this.$store.state.miniInterface) {
        return this.readWidthConfig - 33;
      } else {
        return this.windowSize.width - 33;
      }
    },
    readingProgress() {
      if (this.catalog && this.catalog.length) {
        return (
          parseInt(((this.chapterIndex + 1) * 100) / this.catalog.length) + "%"
        );
      } else {
        return "";
      }
    },
    showPrevPageStyle() {
      if (this.isSlideRead && this.$store.getters.config.clickMethod !== "固定模式") {
        // 左半部
        return {
          left: 0,
          top: 0,
          bottom: 0,
          right: this.windowSize.width / 2 + "px",
          background: "#43987324",
          paddingRight: this.windowSize.width * 0.2 + "px"
        };
      } else {
        // 上半部
        return {
          left: 0,
          top: 0,
          right: 0,
          bottom:
            (this.$store.getters.config.clickMethod === "固定模式"
              ? this.windowSize.height * 0.7
              : this.windowSize.height / 2) + "px",
          background: "#43987324"
        };
      }
    },
    showMenuZoneStyle() {
      return {
        top: this.windowSize.height * 0.3 + "px",
        bottom: this.windowSize.height * 0.3 + "px",
        left: this.windowSize.width * 0.3 + "px",
        right: this.windowSize.width * 0.3 + "px",
        background: "#636060",
        zIndex: 10
      };
    },
    showNextPageStyle() {
      if (this.isSlideRead && this.$store.getters.config.clickMethod !== "固定模式") {
        // 右半部
        return {
          right: 0,
          top: 0,
          bottom: 0,
          left: this.windowSize.width / 2 + "px",
          background: "#6b1a7324",
          paddingLeft: this.windowSize.width * 0.2 + "px"
        };
      } else {
        // 下半部
        return {
          left: 0,
          bottom: 0,
          right: 0,
          top:
            (this.$store.getters.config.clickMethod === "固定模式"
              ? this.windowSize.height * 0.7
              : this.windowSize.height / 2) + "px",
          background: "#6b1a7324"
        };
      }
    },
    showNextPageExtraStyle() {
      if (this.$store.getters.config.clickMethod === "固定模式") {
        return {
          top: this.windowSize.height * 0.3 + "px",
          bottom: this.windowSize.height * 0.3 + "px",
          right: 0,
          left: this.windowSize.width * 0.7 + "px",
          background: "#6b1a7324",
          paddingRight: this.windowSize.width * 0.2 + "px"
        };
      }
      return { display: "none" };
    },
    showPrevPageExtraStyle() {
      if (this.$store.getters.config.clickMethod === "固定模式") {
        return {
          top: this.windowSize.height * 0.3 + "px",
          bottom: this.windowSize.height * 0.3 + "px",
          left: 0,
          right: this.windowSize.width * 0.7 + "px",
          background: "#43987324",
          paddingRight: this.windowSize.width * 0.2 + "px"
        };
      }
      return { display: "none" };
    },
    loginAuth() {
      return this.$store.state.loginAuth;
    },
    filterRules() {
      return this.$store.state.filterRules;
    },
    httpTTSList() {
      return this.$store.state.httpTTS;
    },
    themeBtnStyle() {
      return {
        background: this.$store.getters.currentThemeConfig.popupPure
      };
    },
    popupAbsoluteBtnStyle() {
      return {
        background: this.$store.getters.currentThemeConfig.popupPure
      };
    },
    voiceName: {
      get() {
        return this.$store.state.speechVoiceConfig.voiceName;
      },
      set(val) {
        if (val !== this.$store.state.speechVoiceConfig.voiceName) {
          if (this.speechSpeaking) {
            this.restartSpeech();
          }
        }
        this.$store.commit("setSpeechVoiceConfig", {
          ...this.$store.state.speechVoiceConfig,
          voiceName: val
        });
      }
    },
    speechRate: {
      get() {
        return this.$store.state.speechVoiceConfig.speechRate;
      },
      set(val) {
        if (val !== this.$store.state.speechVoiceConfig.speechRate) {
          if (this.speechSpeaking) {
            this.restartSpeech();
          }
        }
        this.$store.commit("setSpeechVoiceConfig", {
          ...this.$store.state.speechVoiceConfig,
          speechRate: val
        });
      }
    },
    speechPitch: {
      get() {
        return this.$store.state.speechVoiceConfig.speechPitch;
      },
      set(val) {
        if (val !== this.$store.state.speechVoiceConfig.speechPitch) {
          if (this.speechSpeaking) {
            this.restartSpeech();
          }
        }
        this.$store.commit("setSpeechVoiceConfig", {
          ...this.$store.state.speechVoiceConfig,
          speechPitch: val
        });
      }
    },
    ttsType: {
      get() {
        return this.$store.state.speechVoiceConfig.ttsType;
      },
      set(val) {
        if (
          val !== this.$store.state.speechVoiceConfig.ttsType &&
          this.speechSpeaking
        ) {
          this.restartSpeech();
        }
        this.$store.commit("setSpeechVoiceConfig", {
          ...this.$store.state.speechVoiceConfig,
          ttsType: val
        });
      }
    },
    cacheTTSAudio: {
      get() {
        return this.$store.state.speechVoiceConfig.cacheTTSAudio;
      },
      set(val) {
        this.$store.commit("setSpeechVoiceConfig", {
          ...this.$store.state.speechVoiceConfig,
          cacheTTSAudio: val
        });
      }
    },
    voiceList() {
      if (this.ttsType === "local") {
        return this.localVoiceList;
      }
      if (this.ttsType === "edge") {
        return ttsVoiceList;
      }
      return this.httpTTSList;
    },
    isCarToon() {
      return (
        !this.error &&
        !this.isEpub &&
        !this.isCbz &&
        !this.isPdf &&
        (this.content || "").indexOf("<img") >= 0
      );
    },
    isAudio() {
      return !this.error && (this.$store.getters.readingBook || {}).type === 1;
    },
    isVideo() {
      return !this.error && (this.$store.getters.readingBook || {}).type === 4;
    },
    isEpub() {
      const bookUrl = (this.$store.getters.readingBook || {}).bookUrl;
      return !this.error && bookUrl && bookUrl.toLowerCase().endsWith(".epub");
    },
    isEpubResolve() {
      return this.isEpub && this.config.epubMode !== "iframe";
    },
    isEpubIframe() {
      return this.isEpub && this.config.epubMode === "iframe";
    },
    isCbz() {
      const bookUrl = (this.$store.getters.readingBook || {}).bookUrl;
      return !this.error && bookUrl && bookUrl.toLowerCase().endsWith(".cbz");
    },
    isPdf() {
      const bookUrl = (this.$store.getters.readingBook || {}).bookUrl;
      return !this.error && bookUrl && bookUrl.toLowerCase().endsWith(".pdf");
    },
    scrollOffset() {
      // 两行 + 两个段间距
      return (
        this.$store.getters.config.fontSize *
          this.$store.getters.config.lineHeight *
          2 +
        this.$store.getters.config.fontSize *
          this.$store.getters.config.paragraphSpace *
          2
      );
    },
    formatedTitle() {
      return this.formatChinese(this.title);
    },
    chineseFont() {
      return this.config.chineseFont;
    },
    slideDistance() {
      return 16 + this.config.horizontalPadding;
    }
  },
  methods: {
    init(refresh) {
      const readingBook = this.$store.getters.readingBook || {};
      if (readingBook.bookUrl) {
        this.initBook(refresh);
        return;
      }
      const bookUrl =
        this.$route && this.$route.query && this.$route.query.bookUrl;
      if (bookUrl) {
        if (
          !(this.$store.getters.shelfBooks || []).length &&
          this.$root.$children[0] &&
          this.$root.$children[0].loadBookShelf
        ) {
          this.$root.$children[0]
            .loadBookShelf()
            .then(() => this.initBookFromUrl(refresh))
            .catch(() => this.initBookFromUrl(refresh));
          return;
        }
        this.initBookFromUrl(refresh);
        return;
      }
      {
        this.$message.error("缓存信息丢失,请在书架选择书籍！");
        this.title = "";
        this.content = "缓存信息丢失,请在书架选择书籍！";
        this.error = true;
        this.show = true;
      }
    },
    initBookFromUrl(refresh) {
      const bookUrl =
        this.$route && this.$route.query && this.$route.query.bookUrl;
      const shelfBook = (this.$store.getters.shelfBooks || []).find(
        book => book.bookUrl === bookUrl
      );
      if (!shelfBook) {
        this.$message.error("缓存信息丢失,请在书架选择书籍！");
        this.title = "";
        this.originalContent = "";
        this.content = "缓存信息丢失,请在书架选择书籍！";
        this.error = true;
        this.show = true;
        return;
      }
      this.$store.commit("setReadingBook", {
        ...shelfBook,
        name: shelfBook.name,
        bookUrl: shelfBook.bookUrl,
        index:
          shelfBook.index !== undefined
            ? shelfBook.index
            : shelfBook.durChapterIndex !== undefined
            ? shelfBook.durChapterIndex
            : 0,
        type: shelfBook.type,
        coverUrl: shelfBook.customCoverUrl || shelfBook.coverUrl,
        tocUrl: shelfBook.tocUrl,
        author: shelfBook.author,
        origin: shelfBook.origin,
        originName: shelfBook.originName,
        latestChapterTitle: shelfBook.latestChapterTitle,
        intro: shelfBook.intro
      });
      this.initBook(refresh);
    },
    initBook(refresh) {
      const readingBook = this.$store.getters.readingBook || {};
      if (!readingBook.bookUrl) {
        this.$message.error("请在书架选择书籍");
        return;
      }
      if (
        refresh ||
        !this.lastReadingBook ||
        this.lastReadingBook.bookUrl !== readingBook.bookUrl ||
        this.lastReadingBook.index !== readingBook.index
      ) {
        this.title = "";
        this.show = false;
        this.loading = this.$loading({
          target: this.$refs.content,
          lock: true,
          text: "正在获取内容",
          spinner: "el-icon-loading",
          background: "rgba(0,0,0,0)"
        });
        this.lastReadingBook = readingBook;
        this.autoShowPosition();
        this.loadCatalog(false, true);
      } else {
        if (this.isScrollRead) {
          this.scrollStartChapterIndex = this.chapterIndex;
          this.showPrevChapterSize = 0;
          this.computeShowChapterList().then(() => {
            this.autoShowPosition(true);
          });
        } else if (this.isEpubIframe) {
          this.autoShowPosition();
        } else {
          this.$nextTick(() => {
            this.autoShowPosition(true);
          });
        }
        setTimeout(() => {
          this.$store.commit("setReadingBook", this.lastReadingBook);
        }, 100);
      }
    },
    changeBook(book) {
      this.$message.info("换书成功");
      this.popBookShelfVisible = false;
      this.show = false;
      this.$store.commit("setReadingBook", book);
      this.$router.push({
        path: "/reader",
        query: { bookUrl: book.bookUrl }
      });
      this.loadCatalog(true, true);
    },
    changeBookSource() {
      this.popBookSourceVisible = false;
      this.show = false;
      this.tryRefresh = false;
      this.loadCatalog(true, true);
    },
    loadCatalog(refresh, init) {
      if (!this.api) {
        setTimeout(() => {
          if (this.loadCatalog) {
            this.loadCatalog(refresh);
          }
        }, 1000);
        return;
      }
      this.getCatalog(refresh).then(
        res => {
          if (res.data.isSuccess) {
            var book = Object.assign({}, this.$store.getters.readingBook);
            book.catalog = res.data.data;
            this.$store.commit("setReadingBook", book);
            this.$emit("loadCatalog");
            var index = book.index || 0;
            this.getContent(index);
          } else {
            if (init) {
              this.title = "";
              this.originalContent = "获取章节目录失败！\n" + res.data.errorMsg;
              this.content = "获取章节目录失败！\n" + res.data.errorMsg;
              this.error = true;
              this.show = true;
              this.$emit("showContent");
            }
            this.loading.close();
          }
        },
        error => {
          this.loading.close();
          this.$message.error(
            "获取书籍目录列表 " + (error && error.toString())
          );
        }
      );
    },
    getCatalog(refresh) {
      const params = {
        url: this.$store.getters.readingBook.bookUrl,
        refresh: refresh ? 1 : 0
      };
      if (this.$route.query.search) {
        // 来自搜索结果，请求需要带上 书源链接
        params.bookSourceUrl = this.$store.getters.readingBook.origin;
      }
      return cacheFirstRequest(
        () => Axios.post(this.api + "/getChapterList", params),
        this.$store.getters.readingBook.name +
          "_" +
          this.$store.getters.readingBook.author +
          "@" +
          this.$store.getters.readingBook.bookUrl +
          "@chapterList",
        refresh
      );
    },
    refreshCatalog() {
      return this.loadCatalog(true);
    },
    getBookContent(chapterIndex, options, refresh, cache) {
      return this.$root.$children[0].getBookContent(
        chapterIndex,
        options,
        refresh,
        cache
      );
    },
    saveBookContent(url, index, content) {
      const cacheKey =
        "localCache@" +
        this.$store.getters.readingBook.name +
        "_" +
        this.$store.getters.readingBook.author +
        "@" +
        url +
        "@chapterContent-" +
        index;
      return Axios.post(this.api + "/saveBookContent", {
        url,
        index,
        content
      })
        .then(res => {
          if (res.data.isSuccess) {
            return window.$cacheStorage
              .removeItem(cacheKey)
              .catch(() => {})
              .then(() => {
                this.$message.success("保存成功！注意：刷新章节内容将会失效");
              });
          }
          return res;
        })
        .catch(error => {
          this.$message.error(
            "保存章节内容失败 " + (error && error.toString())
          );
          throw error;
        });
    },
    refreshContent() {
      this.autoShowPosition();
      this.getContent(this.$store.getters.readingBook.index, true);
    },
    gotoChapter(index) {
      if (typeof this.$store.getters.readingBook.catalog[index] === "undefined") {
        this.$message.error("章节信息错误");
        return;
      }
      let book = { ...this.$store.getters.readingBook };
      book.index = index;
      this.$store.commit("setReadingBook", book);
      if (this.isScrollRead) {
        this.scrollStartChapterIndex = index;
        this.computeShowChapterList(true);
        return;
      }
      this.getContent(index);
    },
    getContent(index, refresh) {
      //展示进度条
      this.show = false;
      this.contentInnerStyle = {};
      if (!this.loading || !this.loading.visible) {
        this.loading = this.$loading({
          target: this.$refs.content,
          lock: true,
          text: refresh ? "正在刷新内容" : "正在获取内容",
          spinner: "el-icon-loading",
          background: "rgba(0,0,0,0)"
        });
      }
      let bookUrl = this.$store.getters.readingBook.bookUrl;
      try {
        // 保存阅读进度
        let book = { ...this.$store.getters.readingBook };
        book.index = index;
        this.$store.commit("setReadingBook", book);
      } catch (error) {
        // eslint-disable-next-line no-console
        console.error(error);
      }
      //强制滚回顶层
      this.toTop(0);
      // 如果超出目录范围，尝试刷新目录
      if (!this.$store.getters.readingBook.catalog[index]) {
        if (this.tryRefresh) {
          this.tryRefresh = false;
          this.content = "获取章节内容失败，请更新目录！";
          this.error = true;
          this.show = true;
          this.$emit("showContent");
          this.loading.close();
        } else {
          this.tryRefresh = true;
          this.refreshCatalog();
        }
        return;
      }
      let chapterName = this.$store.getters.readingBook.catalog[index].title;
      let chapterIndex = this.$store.getters.readingBook.catalog[index].index;
      this.title = chapterName;
      if (this.isScrollRead) {
        this.scrollStartChapterIndex = chapterIndex;
      }
      this.getBookContent(chapterIndex, {}, refresh).then(
        res => {
          if (
            bookUrl !== this.$store.getters.readingBook.bookUrl ||
            index !== this.$store.getters.readingBook.index
          ) {
            // 已经换书或者换章节了
            return;
          }
          if (res.data.isCache) {
            // 命中缓存，此时发送进度保存请求
            this.saveBookProgress();
          }
          if (res.data.isSuccess) {
            let data = res.data.data;
            this.originalContent = data;
            this.content = this.filterContent(data);
            this.addChapterContentToCache({
              bookUrl,
              index: index,
              title: chapterName,
              content: res.data.data,
              error: false
            });
            this.loading.close();
            this.error = false;
            this.show = true;
            this.$emit("showContent");
          } else {
            this.content = "获取章节内容失败！\n" + res.data.errorMsg;
            this.addChapterContentToCache({
              bookUrl,
              index: index,
              title: chapterName,
              content: "获取章节内容失败！\n" + res.data.errorMsg,
              error: true
            });
            this.error = true;
            this.show = true;
            this.$emit("showContent");
            this.loading.close();
          }
          if (this.isScrollRead) {
            this.computeShowChapterList(true);
          }
        },
        error => {
          if (
            bookUrl !== this.$store.getters.readingBook.bookUrl ||
            index !== this.$store.getters.readingBook.index
          ) {
            // 已经换书或者换章节了
            return;
          }
          this.content = "获取章节内容失败！\n" + (error && error.toString());
          this.addChapterContentToCache({
            bookUrl,
            index: index,
            title: chapterName,
            content: "获取章节内容失败！\n" + (error && error.toString()),
            error: true
          });
          this.error = true;
          this.show = true;
          this.$emit("showContent");
          this.loading.close();
          this.$message.error(
            "获取章节内容失败 " + (error && error.toString())
          );
          if (this.isScrollRead) {
            this.computeShowChapterList(true);
          }
          throw error;
        }
      );
    },
    toggleEditContent() {
      if (this.isEditContent) {
        this.isEditContent = false;
        this.saveContent();
      } else {
        this.isEditContent = true;
      }
    },
    async saveContent() {
      const res = await this.$confirm("是否保存章节内容?", "提示", {
        confirmButtonText: "保存",
        cancelButtonText: "取消",
        type: "warning"
      }).catch(() => {
        return false;
      });
      if (!res) {
        this.autoShowPosition();
        this.getContent(this.$store.getters.readingBook.index);
        return;
      }
      const content = this.getTextContent(this.getContentParagraphContainer());
      const cacheKey =
        "localCache@" +
        this.$store.getters.readingBook.name +
        "_" +
        this.$store.getters.readingBook.author +
        "@" +
        this.$store.getters.readingBook.bookUrl +
        "@chapterContent-" +
        this.chapterIndex;
      return Axios.post(this.api + "/saveBookContent", {
        url: this.$store.getters.readingBook.bookUrl,
        index: this.chapterIndex,
        content
      })
        .then(res => {
          if (res.data.isSuccess) {
            return window.$cacheStorage
              .removeItem(cacheKey)
              .catch(() => {})
              .then(() => {
                this.$message.success("保存成功！注意：刷新章节内容将会失效");
              });
          }
          return res;
        })
        .catch(error => {
          this.$message.error(
            "保存章节内容失败 " + (error && error.toString())
          );
        });
    },
    getTextContent(element) {
      if (!element) {
        return "";
      }
      if (element.nodeName === "H3") {
        return "";
      }
      if (element.nodeName === "P") {
        return element.innerText;
      }
      if (element.nodeName === "IMG") {
        const clone = element.cloneNode(true);
        const errorSrc = clone.getAttribute("data-error");
        clone.removeAttribute("data-error");
        clone.removeAttribute("data-src");
        clone.removeAttribute("lazy");
        clone.setAttribute("src", errorSrc);
        return clone.outerHTML;
      }
      if (element.nodeName === "#text") {
        return element.nodeValue;
      }
      let text = "";
      for (let i = 0; i < element.childNodes.length; i++) {
        if (i > 0 && element.childNodes[i].nodeName !== "#text") {
          text += "\n";
        }
        text += this.getTextContent(element.childNodes[i]);
      }
      return text.replace(/^\n*/, "").replace(/\n*$/, "");
    },
    filterContent(content) {
      if (this.isEpub || this.isAudio || this.isVideo || this.isPdf) {
        return content;
      }
      if (!content) {
        return content;
      }
      try {
        this.filterRules.forEach(rule => {
          if (
            typeof rule.isEnabled !== "undefined" &&
            rule.isEnabled === false
          ) {
            return;
          }
          const scope = (rule.scope || "*").split(";");
          if (
            scope[0] === "*" ||
            scope[0] === this.$store.getters.readingBook.name
          ) {
            if (
              scope.length == 1 ||
              (scope.length > 1 &&
                scope[1] === this.$store.getters.readingBook.bookUrl)
            ) {
              if (rule.isRegex) {
                content = content.replace(
                  new RegExp(rule.pattern, "ig"),
                  rule.replacement
                );
              } else {
                content = content.replaceAll(rule.pattern, rule.replacement);
              }
            }
          }
        });
      } catch (error) {
        //
      }
      content = content.replace(/\\n+/g, "\n");
      content = this.formatChinese(content);
      return content;
    },
    loadShowChapter(index, refresh) {
      if (
        !refresh &&
        this.chapterContentCache &&
        this.chapterContentCache[this.readingBook.bookUrl] &&
        this.chapterContentCache[this.readingBook.bookUrl].chapters[index] &&
        !this.chapterContentCache[this.readingBook.bookUrl].chapters[index]
          .error
      ) {
        if (
          index >= this.chapterIndex - this.showPrevChapterSize &&
          index <= this.chapterIndex + this.showNextChapterSize
        ) {
          this.computeShowChapterList();
        }
        return Promise.resolve();
      }
      let bookUrl = this.$store.getters.readingBook.bookUrl;
      if (!this.$store.getters.readingBook.catalog) {
        return new Promise(resolve => {
          this.$once("loadCatalog", () => {
            this.loadShowChapter(index, refresh).then(resolve);
          });
        });
      }
      // 如果超出目录范围，尝试刷新目录
      if (!this.$store.getters.readingBook.catalog[index]) {
        return Promise.reject("章节不存在");
      }
      let chapterName = this.$store.getters.readingBook.catalog[index].title;
      let chapterIndex = this.$store.getters.readingBook.catalog[index].index;
      return this.getBookContent(chapterIndex, {}, refresh, true).then(
        res => {
          if (res.data.isSuccess) {
            this.addChapterContentToCache({
              bookUrl,
              index: index,
              title: chapterName,
              content: res.data.data,
              error: false
            });
          } else {
            this.addChapterContentToCache({
              bookUrl,
              index: index,
              title: chapterName,
              content: "获取章节内容失败！\n" + res.data.errorMsg,
              error: true
            });
          }
        },
        error => {
          this.addChapterContentToCache({
            bookUrl,
            index: index,
            title: chapterName,
            content: "获取章节内容失败！\n" + (error && error.toString()),
            error: true
          });
          throw error;
        }
      );
    },
    addChapterContentToCache(chapter) {
      const MAX_CACHED_BOOKS = 3;
      const bookUrl = this.readingBook.bookUrl;
      if (!this.chapterContentCache) {
        this.chapterContentCache = {};
      }
      if (!this.chapterContentCache[bookUrl]) {
        const keys = Object.keys(this.chapterContentCache);
        if (keys.length >= MAX_CACHED_BOOKS) {
          delete this.chapterContentCache[keys[0]];
        }
        this.chapterContentCache[bookUrl] = { chapters: {} };
      }
      const cache = this.chapterContentCache[bookUrl];
      if (
        typeof cache.chapters[chapter.index] === "undefined" ||
        !chapter.error ||
        cache.chapters[chapter.index].error
      ) {
        chapter.isVolume = !!(this.readingBook.catalog[chapter.index] || {})
          .isVolume;
        cache.chapters[chapter.index] = chapter;
      }
    },
    computeShowChapterList(reset) {
      const cache =
        this.chapterContentCache &&
        this.chapterContentCache[this.readingBook.bookUrl];
      if (!cache) {
        return new Promise(resolve => {
          setTimeout(() => {
            this.computeShowChapterList(reset).then(resolve);
          }, 10);
        });
      }
      if (!this.isScrollRead) {
        return Promise.resolve();
      }
      const list = [];
      let startIndex = this.scrollStartChapterIndex;
      if (typeof startIndex !== "number") {
        startIndex = this.chapterIndex;
      }
      if (this.config.readMethod === "上下滚动2") {
        startIndex = this.chapterIndex - this.showPrevChapterSize;
      }
      const waitPromise = [];
      for (
        let i = startIndex;
        i <= this.chapterIndex + this.showNextChapterSize;
        i++
      ) {
        if (!cache.chapters[i]) {
          waitPromise.push(this.loadShowChapter(i));
          continue;
        }
        list.push({
          ...cache.chapters[i],
          content: this.filterContent(cache.chapters[i].content)
        });
      }
      if (waitPromise.length) {
        return Promise.all(waitPromise).then(() => {
          this.computeShowChapterList(reset);
        });
      }
      let needRestore = true;
      if (
        this.showChapterList.length &&
        list.length &&
        this.showChapterList[0].index === list[0].index
      ) {
        needRestore = false;
      }
      const scrollAnchor =
        !reset && needRestore ? this.captureScrollAnchor() : null;
      const shouldRestoreFromCache =
        !reset &&
        needRestore &&
        !scrollAnchor &&
        this.config.readMethod === "上下滚动2";
      this.saveReadingPosition();
      // 暂停记录位置
      this.startSavePosition = false;
      // 记录当前章节
      this.showChapterList = list;
      this.$nextTick(() => {
        this.restoreScrollAnchor(scrollAnchor);
        this.computePages(() => {
          if (reset) {
            // 切换上下章节，滚动到顶部
            this.toTop(0);
            this.startSavePosition = true;
          } else if (shouldRestoreFromCache) {
            this.autoShowPosition(true);
          } else {
            this.startSavePosition = true;
          }
        });
      });
    },
    saveBookProgress(index) {
      return Axios.post(
        this.api + "/saveBookProgress",
        {
          url: this.$store.getters.readingBook.bookUrl,
          index: index !== undefined ? index : this.chapterIndex
        },
        {
          silent: true
        }
      ).catch(() => {});
    },
    toTop(interval) {
      if (this.$store.state.miniInterface) {
        this.scrollContent(
          -(document.documentElement.scrollTop || document.body.scrollTop),
          interval
        );
      } else {
        jump(this.$refs.top, { duration: interval });
      }
    },
    toBottom(interval) {
      jump(this.$refs.bottom, { duration: interval });
    },
    toNextChapter(onError) {
      if (
        !this.$store.getters.readingBook ||
        !this.$store.getters.readingBook.bookUrl ||
        !this.$store.getters.readingBook.catalog
      ) {
        onError && onError();
        return;
      }
      let index = this.$store.getters.readingBook.index;
      index++;
      if (
        typeof this.$store.getters.readingBook.catalog[index] !== "undefined"
      ) {
        if (this.isScrollRead) {
          this.scrollStartChapterIndex = index;
          this.computeShowChapterList(true);
          return;
        }
        this.getContent(index);
      } else {
        onError && onError();
        this.$message.error("本章是最后一章");
      }
    },
    toLastChapter(onError) {
      if (
        !this.$store.getters.readingBook ||
        !this.$store.getters.readingBook.bookUrl ||
        !this.$store.getters.readingBook.catalog
      ) {
        onError && onError();
        return;
      }
      let index = this.$store.getters.readingBook.index;
      index--;
      if (
        typeof this.$store.getters.readingBook.catalog[index] !== "undefined"
      ) {
        if (this.isScrollRead) {
          this.scrollStartChapterIndex = index;
          this.computeShowChapterList(true);
          return;
        }
        this.getContent(index);
      } else {
        this.$message.error("本章是第一章");
        onError && onError();
      }
    },
    toShelf() {
      this.$router.push("/");
    },
    onContentChange() {
      this.computePages();
    },
    computePages(cb) {
      if (!this.$refs.bookContentRef || !this.$refs.bookContentRef.$el) {
        setTimeout(() => {
          this.computePages(cb);
        }, 30);
        return;
      }
      if (this.isSlideRead) {
        if (this.isEpub) {
          this.totalPages = Math.ceil(
            this.$refs.bookContentRef.iframeSize.scrollWidth /
              (this.windowSize.width - this.slideDistance)
          );
        } else {
          this.totalPages = Math.ceil(
            this.$refs.bookContentRef.$el.scrollWidth /
              (this.windowSize.width - this.slideDistance)
          );
        }
      } else {
        let pageHeight = this.windowSize.height - this.scrollOffset;
        this.totalPages = Math.ceil(
          this.$refs.bookContentRef.$el.scrollHeight / pageHeight
        );
        if (!this.isScrollRead) {
          // 最后一页的填充高度，让"加载下一章"按钮出现在最后一页
          const paddingBottom =
            pageHeight * this.totalPages -
            this.$refs.bookContentRef.$el.scrollHeight -
            68;
          this.contentInnerStyle = { "padding-bottom": paddingBottom + "px" };
        }
      }
      if (this.showLastPage) {
        this.showPage(this.totalPages, 0);
        this.showLastPage = false;
      }
      this.$emit("afterComputePages");
      cb && cb();
    },
    nextPage(moveX) {
      if (!this.show) {
        return;
      }
      if (this.transforming) {
        return;
      }
      if (this.isSlideRead) {
        if (this.currentPage < this.totalPages) {
          if (typeof moveX === "undefined") {
            this.transformX =
              -(this.windowSize.width - this.slideDistance) * (this.currentPage - 1);
          }
          this.currentPage += 1;
          this.transforming = true;
          this.transform(
            typeof moveX === "undefined"
              ? -(this.windowSize.width - this.slideDistance)
              : moveX,
            this.animateMSTime
          );
        } else {
          this.toNextChapter(() => {
            if (typeof moveX !== "undefined") {
              // 没有下一章，但是已经做了动画，恢复
              this.showPage(this.currentPage, 0);
            }
          });
        }
      } else {
        if (this.nearBottom(10)) {
          this.currentPage = 1;
          this.toNextChapter();
        } else {
          this.currentPage += 1;
          const moveY = this.windowSize.height - this.scrollOffset;
          this.transforming = true;
          this.scrollContent(moveY, this.animateMSTime);
        }
      }
    },
    prevPage(moveX) {
      if (!this.show) {
        return;
      }
      if (this.transforming) {
        return;
      }
      if (this.isSlideRead) {
        if (this.currentPage > 1) {
          if (typeof moveX === "undefined") {
            this.transformX =
              -(this.windowSize.width - this.slideDistance) * (this.currentPage - 1);
          }
          this.currentPage -= 1;
          this.transforming = true;
          this.transform(
            typeof moveX === "undefined"
              ? this.windowSize.width - this.slideDistance
              : moveX,
            this.animateMSTime
          );
        } else {
          this.onNextContentPagesInit(() => {
            this.showPage(this.totalPages, 0);
          });
          this.toLastChapter(() => {
            if (typeof moveX !== "undefined") {
              // 没有上一章，但是已经做了动画，恢复
              this.showPage(this.currentPage, 0);
            }
          });
        }
      } else {
        if (
          (document.documentElement.scrollTop || document.body.scrollTop) > 0
        ) {
          this.currentPage -= 1;
          const moveY = -this.windowSize.height + this.scrollOffset;
          this.transforming = true;
          this.scrollContent(moveY, this.animateMSTime);
        } else {
          this.currentPage = 1;
          this.onNextContentPagesInit(() => {
            this.showPage(this.totalPages, 0);
          });
          this.toLastChapter();
        }
      }
    },
    showPage(page, duration) {
      if (!this.show) {
        return;
      }
      this.currentPage = Math.min(page, this.totalPages);
      if (this.isSlideRead) {
        const moveX =
          -(this.windowSize.width - this.slideDistance) * (this.currentPage - 1) -
          this.transformX;
        this.transform(
          moveX,
          typeof duration === "undefined" ? this.animateMSTime : duration
        );
      } else {
        const moveY =
          (this.windowSize.height - 10) * (this.currentPage - 1) -
          (document.documentElement.scrollTop || document.body.scrollTop);
        this.scrollContent(
          moveY,
          typeof duration === "undefined" ? this.animateMSTime : duration
        );
      }
    },
    transform(moveX, duration, reset) {
      if (reset) {
        this.transformX = 0;
      }
      const onEnd = () => {
        if (this.isEpub) {
          if (this.$refs.bookContentRef) {
            this.$refs.bookContentRef.transformX(this.transformX + moveX);
          }
        } else {
          this.contentStyle = {
            transform: `translateX(${this.transformX + moveX}px)`
          };
        }
        this.transformX += moveX;
        this.transforming = false;
        // 保存进度
        setTimeout(this.saveReadingPosition, duration);
      };
      if (!duration) {
        onEnd();
        return;
      }
      const timing = Animate.Utils.makeEaseInOut(
        Animate.Timings.power.bind(null, 3)
      );

      new Animate({
        duration: duration || 500,
        timing: timing,
        draw: progress => {
          if (this.isEpub) {
            if (this.$refs.bookContentRef) {
              this.$refs.bookContentRef.transformX(this.transformX + moveX * progress);
            }
          } else {
            this.contentStyle = {
              transform: `translateX(${this.transformX + moveX * progress}px)`
            };
          }
        },
        onEnd
      });
    },
    scrollContent(moveY, duration, isAccurate) {
      const lastScrollTop = isAccurate
        ? 0
        : document.documentElement.scrollTop || document.body.scrollTop;
      const onEnd = () => {
        document.documentElement.scrollTop = lastScrollTop + moveY;
        document.body.scrollTop = lastScrollTop + moveY;
        this.transforming = false;
        // 保存进度
        setTimeout(this.saveReadingPosition, duration);
      };
      if (!duration) {
        onEnd();
        return;
      }
      const timing = Animate.Utils.makeEaseInOut(
        Animate.Timings.power.bind(null, 3)
      );

      new Animate({
        duration: duration || 500,
        timing: timing,
        draw: progress => {
          document.documentElement.scrollTop = lastScrollTop + moveY * progress;
          document.body.scrollTop = lastScrollTop + moveY * progress;
        },
        onEnd
      });
    },
    nearBottom(distance) {
      // 与 JAR 全局 mixin 中的 nearBottom 一致
      return (
        document.documentElement.clientHeight +
          (document.documentElement.scrollTop || document.body.scrollTop) >=
        document.documentElement.scrollHeight - distance
      );
    },
    handlerClick(e) {
      if (this.isEpub) {
        return;
      }
      if (!this.lastTouch && !this.ignoreNextClick) {
        this.eventHandler(e);
      }
      this.ignoreNextClick = false;
    },
    handleTouchStart(e) {
      this.lastSelection = this.checkSelection();
      if (this.lastSelection) {
        return;
      }
      if (this.isAudio || this.isVideo) {
        return;
      }
      this.lastTouch = false;
      this.lastMoveX = false;
      if (e.touches && e.touches[0]) {
        this.lastTouch = e.touches[0];
      }
    },
    handleTouchMove(e) {
      if (this.checkSelection()) {
        return;
      }
      if (e.touches && e.touches[0] && this.lastTouch) {
        this.lastMoveY = e.touches[0].clientY - this.lastTouch.clientY;
        if (this.isSlideRead) {
          e.preventDefault && e.preventDefault();
          e.stopPropagation && e.stopPropagation();
          const moveX = e.touches[0].clientX - this.lastTouch.clientX;
          if (this.isEpub) {
            if (this.$refs.bookContentRef) {
              this.$refs.bookContentRef.transformX(this.transformX + moveX);
            }
          } else {
            this.contentStyle = {
              transform: `translateX(${this.transformX + moveX}px)`
            };
          }
          this.lastMoveX = moveX;
        }
      }
    },
    handleTouchEnd() {
      if (this.checkSelection(true)) {
        return;
      }
      if (this.lastSelection) {
        setTimeout(() => {
          this.showTextFilterPrompt(this.lastSelection);
          this.lastSelection = false;
        }, 200);
        return;
      }
      if (this.lastMoveX) {
        this.transformX += this.lastMoveX;
        if (this.lastMoveX > 0) {
          // 上一页
          this.prevPage(this.windowSize.width - this.slideDistance - this.lastMoveX);
        } else {
          // 下一页
          this.nextPage(
            -(this.windowSize.width - this.slideDistance) - this.lastMoveX
          );
        }
      } else if (Math.abs(this.lastMoveY) <= 3 && this.lastTouch) {
        if (!this.isEpub) {
          this.eventHandler(this.lastTouch);
        }
      }
      setTimeout(() => {
        this.lastTouch = false;
        this.lastMoveX = false;
        this.lastMoveY = false;
      }, 300);
    },
    epubTouch(data) {
      switch (data.event) {
        case "touchstart":
          this.handleTouchStart(data.data);
          break;
        case "touchmove":
          this.handleTouchMove(data.data);
          break;
        default:
          this.handleTouchEnd();
      }
    },
    epubClickHash(rect) {
      if (this.isSlideRead) {
        if (typeof rect.left !== "undefined") {
          this.showPage(
            this.currentPage +
              Math.round(rect.left / (this.windowSize.width - this.slideDistance)) +
              (rect.left > 0 ? 1 : 0),
            0
          );
        }
      } else if (typeof rect.top !== "undefined") {
        this.scrollContent(
          rect.top -
            (this.$store.state.miniInterface
              ? this.getFirstParagraphPos().bottom
              : 0) -
            (window.webAppDistance | 0) -
            (this.$store.state.safeArea.top | 0),
          0,
          true
        );
      }
    },
    epubLocationChangeHandler(url) {
      function getPathname(path) {
        const a = document.createElement("a");
        a.href = path;
        return decodeURIComponent(a.pathname);
      }
      url = getPathname(url);
      // 判断是否跳转了其他章节
      const currentChapter = this.catalog[this.chapterIndex];
      if (currentChapter) {
        const chapterPrefix = this.content.replace(currentChapter.url, "");
        const iframeUrlPath = url.replace(chapterPrefix, "");
        let newChapterIndex = -1;
        for (let i = 0; i < this.catalog.length; i++) {
          if (this.catalog[i].url === iframeUrlPath) {
            newChapterIndex = i;
            break;
          }
        }
        if (newChapterIndex >= 0) {
          let book = { ...this.$store.getters.readingBook };
          book.index = newChapterIndex;
          this.$store.commit("setReadingBook", book);
          this.title = this.$store.getters.readingBook.catalog[
            newChapterIndex
          ].title;
        }
      }
    },
    eventHandler(point) {
      if (this.checkSelection(true)) {
        // 选择文本
        this.ignoreNextClick = true;
        return;
      }
      if (
        this.popBookSourceVisible ||
        this.popBookShelfVisible ||
        this.popCataVisible ||
        this.readSettingsVisible
      ) {
        if (this.isEpub) {
          this.popBookSourceVisible = false;
          this.popBookShelfVisible = false;
          this.popCataVisible = false;
          this.readSettingsVisible = false;
        }
        return;
      }
      if (this.isAudio) {
        // 音频
        // 点击中部区域显示菜单
        if (!this.showReadBar) {
          this.showToolBar = !this.showToolBar;
        }
        return;
      }
      if (this.autoReading) {
        this.showToolBar = !this.showToolBar;
        return;
      }
      if (this.isEditContent) {
        this.showToolBar = !this.showToolBar;
        return;
      }
      // 根据点击位置判断操作
      const midX = this.windowSize.width / 2;
      const midY = this.windowSize.height / 2;
      const zoneWidth = this.windowSize.width * 0.2;
      const zoneHeight = this.windowSize.height * 0.2;
      if (this.isEpub && this.config.epubMode === "iframe") {
        point.clientY =
          point.clientY +
          45 -
          (document.documentElement.scrollTop || document.body.scrollTop);
      }
      if (
        Math.abs(point.clientY - midY) <= zoneHeight &&
        Math.abs(point.clientX - midX) <= zoneWidth
      ) {
        // 点击中部区域显示菜单
        if (!this.showReadBar) {
          this.showToolBar = !this.showToolBar;
        }
      } else {
        if (this.isVideo) {
          return;
        }
        if (this.$store.getters.config.clickMethod === "下一页") {
          // 全屏点击下一页
          this.showToolBar = false;
          this.nextPage();
          return;
        }
        if (this.$store.getters.config.clickMethod === "不翻页") {
          // 全屏点击不翻页
          this.showToolBar = !this.showToolBar;
          return;
        }
        if (this.$store.getters.config.clickMethod === "固定模式") {
          // 固定模式：上半部/左半部区域翻上一页，下半部/右半部区域翻下一页
          if (
            point.clientY < midY - zoneHeight ||
            (point.clientY > midY - zoneHeight && point.clientX < midX - zoneWidth)
          ) {
            this.showToolBar = false;
            this.prevPage();
          } else if (
            point.clientY > midY + zoneHeight ||
            (point.clientY < midY + zoneHeight && point.clientX > midX + zoneWidth)
          ) {
            this.showToolBar = false;
            this.nextPage();
          }
        } else if (this.isSlideRead) {
          if (point.clientX > midX) {
            // 点击右侧，下一页
            this.showToolBar = false;
            this.nextPage();
          } else if (point.clientX < midX) {
            // 点击左侧，上一页
            this.showToolBar = false;
            this.prevPage();
          }
        } else {
          if (point.clientY > midY) {
            // 点击下部，下一页
            this.showToolBar = false;
            this.nextPage();
          } else if (point.clientY < midY) {
            // 点击上部，上一页
            this.showToolBar = false;
            this.prevPage();
          }
        }
      }
    },
    keydownHandler(event, force) {
      if (
        this.popBookSourceVisible ||
        this.popBookShelfVisible ||
        this.popCataVisible ||
        this.readSettingsVisible ||
        this.showTextFilterPrompting
      ) {
        return;
      }
      if (!force && document.activeElement !== document.body) {
        return;
      }
      if (this.isAudio || this.isVideo) {
        return;
      }
      const keyCodeMap = {
        37: "ArrowLeft",
        38: "ArrowUp",
        39: "ArrowRight",
        40: "ArrowDown",
        27: "Escape",
        32: "Space",
        33: "PageUp",
        34: "PageDown",
        35: "End",
        36: "Home"
      };
      const eventKey =
        event.code === "Space" ? "Space" : event.key || keyCodeMap[event.keyCode];
      if (this.config.quickKeyMode !== "自定义") {
        switch (eventKey) {
          case "ArrowLeft":
            event.preventDefault && event.preventDefault();
            event.stopPropagation && event.stopPropagation();
            this.showToolBar = false;
            if (this.isSlideRead) {
              this.prevPage();
            } else {
              this.toLastChapter();
            }
            break;
          case "ArrowRight":
            event.preventDefault && event.preventDefault();
            event.stopPropagation && event.stopPropagation();
            this.showToolBar = false;
            if (this.isSlideRead) {
              this.nextPage();
            } else {
              this.toNextChapter();
            }
            break;
          case "ArrowUp":
          case "PageUp":
            if (this.isSlideRead || eventKey === "PageUp") {
              event.preventDefault && event.preventDefault();
              event.stopPropagation && event.stopPropagation();
              this.showToolBar = false;
              this.prevPage();
            }
            break;
          case "ArrowDown":
          case "PageDown":
            if (this.isSlideRead || eventKey === "PageDown") {
              event.preventDefault && event.preventDefault();
              event.stopPropagation && event.stopPropagation();
              this.showToolBar = false;
              this.nextPage();
            } else if (!this.isSlideRead && this.nearBottom(10)) {
              this.currentPage = 1;
              this.toNextChapter();
            }
            break;
          case "Escape":
            this.toShelf();
            break;
          case "Home":
            event.preventDefault && event.preventDefault();
            event.stopPropagation && event.stopPropagation();
            this.showToolBar = false;
            this.showPage(0, 0);
            break;
          case "End":
            event.preventDefault && event.preventDefault();
            event.stopPropagation && event.stopPropagation();
            this.showToolBar = false;
            this.showPage(this.totalPages, 0);
            break;
          case "Space":
            if (!this.isSlideRead) {
              event.preventDefault && event.preventDefault();
              event.stopPropagation && event.stopPropagation();
              this.showToolBar = false;
              if (this.nearBottom(10)) {
                this.currentPage = 1;
                this.toNextChapter();
              } else {
                this.scrollContent(this.windowSize.height / 2, this.animateMSTime);
              }
            }
            break;
        }
      } else {
        this.quickKeyHandle(eventKey, event);
      }
    },
    quickKeyHandle(key, event) {
      if (!this.config.quickKey || !this.config.quickKey[key]) {
        return;
      }
      const option = this.config.quickKey[key];
      switch (option) {
        case "上一页":
          event.preventDefault && event.preventDefault();
          event.stopPropagation && event.stopPropagation();
          this.showToolBar = false;
          this.prevPage();
          break;
        case "下一页":
          event.preventDefault && event.preventDefault();
          event.stopPropagation && event.stopPropagation();
          this.showToolBar = false;
          this.nextPage();
          break;
        case "上半页":
          if (!this.isSlideRead) {
            event.preventDefault && event.preventDefault();
            event.stopPropagation && event.stopPropagation();
            this.showToolBar = false;
            if (this.nearBottom(10)) {
              this.toLastChapter();
            } else {
              this.scrollContent(-this.windowSize.height / 2, this.animateMSTime);
            }
          }
          break;
        case "下半页":
          if (!this.isSlideRead) {
            event.preventDefault && event.preventDefault();
            event.stopPropagation && event.stopPropagation();
            this.showToolBar = false;
            if (this.nearBottom(10)) {
              this.currentPage = 1;
              this.toNextChapter();
            } else {
              this.scrollContent(this.windowSize.height / 2, this.animateMSTime);
            }
          }
          break;
        case "上一章":
          event.preventDefault && event.preventDefault();
          event.stopPropagation && event.stopPropagation();
          this.toLastChapter();
          break;
        case "下一章":
          event.preventDefault && event.preventDefault();
          event.stopPropagation && event.stopPropagation();
          this.currentPage = 1;
          this.toNextChapter();
          break;
        case "返回":
          this.toShelf();
          break;
        case "首页":
          event.preventDefault && event.preventDefault();
          event.stopPropagation && event.stopPropagation();
          this.showToolBar = false;
          this.showPage(0, 0);
          break;
        case "尾页":
          event.preventDefault && event.preventDefault();
          event.stopPropagation && event.stopPropagation();
          this.showToolBar = false;
          this.showPage(this.totalPages, 0);
          break;
        default:
          break;
      }
    },
    formatProgressTip(value) {
      return `第 ${value || this.progressValue}/${this.totalPages} 页`;
    },
    formatTime() {
      const now = new Date();
      const pad = v => (v >= 10 ? "" + v : "0" + v);
      this.timeStr = pad(now.getHours()) + ":" + pad(now.getMinutes());
    },
    checkSelection(show) {
      let text = "";
      if (this.isEpubIframe) {
        // epub iframe 模式，从 iframe 内部获取选中文本
        if (
          this.$refs.bookContentRef &&
          this.$refs.bookContentRef.$el &&
          this.$refs.bookContentRef.$el.contentWindow &&
          this.$refs.bookContentRef.$el.contentWindow.getSelection
        ) {
          text = this.$refs.bookContentRef.$el.contentWindow
            .getSelection()
            .toString();
        } else if (
          this.$refs.bookContentRef &&
          this.$refs.bookContentRef.$el &&
          this.$refs.bookContentRef.$el.contentWindow &&
          this.$refs.bookContentRef.$el.contentWindow.document &&
          this.$refs.bookContentRef.$el.contentWindow.document.selection &&
          this.$refs.bookContentRef.$el.contentWindow.document.selection.type !=
            "Control"
        ) {
          text = this.$refs.bookContentRef.$el.contentWindow.document.selection.createRange().text;
        }
      } else {
        if (window.getSelection) {
          text = window.getSelection().toString();
        } else if (document.selection && document.selection.type != "Control") {
          text = document.selection.createRange().text;
        }
      }
      if (text && show) {
        setTimeout(() => {
          if (
            this.$store.getters.config.selectionAction === "过滤弹窗" ||
            this.$store.getters.config.selectionAction === "操作弹窗"
          ) {
            this.showTextOperate(text);
          }
        }, 200);
      }
      return text;
    },
    async showTextOperate(text) {
      if (this.isEditContent) {
        return;
      }
      const res = await this.$confirm(`请选择操作?`, "提示", {
        confirmButtonText: "添加过滤规则",
        cancelButtonText: "添加书签",
        type: "warning",
        closeOnClickModal: false,
        closeOnPressEscape: false,
        distinguishCancelAndClose: true
      }).catch(action => {
        return action === "close" ? "close" : false;
      });
      if (res === "close") {
        return;
      }
      if (res) {
        return this.showTextFilterPrompt(text);
      } else {
        return this.showAddBookmark(text);
      }
    },
    async showTextFilterPrompt(text) {
      if (this.showTextFilterPrompting) {
        return;
      }
      if (!text.replace(/^\s+/, "").replace(/\s+$/, "")) {
        return;
      }

      const replaceRule = Object.assign({}, defaultReplaceRule, {
        name: "文本替换",
        pattern: text,
        replacement: "",
        isRegex: false,
        isEnabled: true,
        scope:
          this.$store.getters.readingBook.name +
          ";" +
          this.$store.getters.readingBook.bookUrl
      });
      this.showTextFilterPrompting = true;
      eventBus.$emit("showReplaceRuleForm", replaceRule, true, () => {
        this.showTextFilterPrompting = false;
      });
    },
    async showAddBookmark(text) {
      if (this.showAddBookmarking) {
        return;
      }
      let pureText = text.replace(/^\s+/, "").replace(/\s+$/, "");
      const paragraph = this.getContentMatchParagraph(pureText, 1, 0.7);
      if (!paragraph) {
        this.$message.error("选择1-2段整段文字才能定位段落");
        return;
      }
      const paragraphLength = 5;
      const paragraphTextLength = 150;
      const paragraphList = [paragraph];
      let bookText = paragraph.innerText + "\n";
      if (
        paragraphList.length < paragraphLength &&
        bookText.length < paragraphTextLength
      ) {
        // 补全内容
        let paragraphIndex = -1;
        const list = this.getContentParagraphList();
        for (let i = 0; i < list.length; i++) {
          if (paragraphIndex > 0 && i > paragraphIndex) {
            paragraphList.push(list[i]);
            bookText += list[i].innerText + "\n";
            if (
              paragraphList.length >= paragraphLength ||
              bookText.length >= paragraphTextLength
            ) {
              break;
            }
          } else if (paragraphList[paragraphList.length - 1] === list[i]) {
            paragraphIndex = i;
          }
        }
      }
      this.showAddBookmarkForm(bookText);
    },
    addBookmark() {
      // 从当前段落开始取 5 段内容
      const current = this.getCurrentParagraph();
      const list = this.getContentParagraphList();
      let bookText = "";
      let paragraphIndex = -1;
      for (let i = 0; i < list.length; i++) {
        if (current === list[i]) {
          paragraphIndex = i;
        }
        if (paragraphIndex >= 0 && i >= paragraphIndex) {
          bookText += list[i].innerText + "\n";
          if (i >= paragraphIndex + 5) {
            break;
          }
        }
      }
      this.showAddBookmarkForm(bookText);
    },
    showAddBookmarkForm(bookText) {
      bookText = bookText.replace(/\\n*$/, "");
      const bookmark = Object.assign({}, defaultBookmark, {
        bookName: this.$store.getters.readingBook.name,
        bookAuthor: this.$store.getters.readingBook.author,
        chapterIndex: this.chapterIndex,
        chapterPos: this.currentPage,
        chapterName: this.title,
        bookText: bookText,
        content: ""
      });
      this.showAddBookmarking = true;
      eventBus.$emit("showBookmarkForm", bookmark, true, () => {
        this.showAddBookmarking = false;
      });
    },
    toogleNight() {
      if (this.isNight) {
        this.$store.commit("setNightTheme", false);
      } else {
        this.$store.commit("setNightTheme", true);
      }
    },
    fetchVoiceList() {
      this.localVoiceList = window.speechSynthesis.getVoices().sort((a, b) => {
        if (a.lang.startsWith("zh-") && b.lang.startsWith("zh-")) {
          return a.lang > b.lang ? 1 : a.lang < b.lang ? -1 : 0;
        } else if (a.lang.startsWith("zh-")) {
          return -1;
        } else if (b.lang.startsWith("zh-")) {
          return 1;
        }
        return a.lang > b.lang ? 1 : a.lang < b.lang ? -1 : 0;
      });
    },
    changeSpeechRate(rate) {
      this.speechRate = rate;
    },
    changeSpeechPitch(pitch) {
      this.speechPitch = pitch;
    },
    changeSpeechMinutes(minute) {
      this.speechMinutes = minute;
      if (minute) {
        this.speechEndTime = new Date().getTime() + minute * 60 * 1000;
      } else {
        this.speechEndTime = 0;
      }
    },
    showHttpTTSDialog() {
      eventBus.$emit("showHttpTTSDialog");
    },
    startSpeech() {
      if (this.error || !this.voiceName) {
        return;
      }
      if (window.speechSynthesis && window.speechSynthesis.speaking) {
        try {
          window.speechSynthesis.cancel();
        } catch (error) {
          // 浏览器实现可能在取消时抛错
        }
      }
      if (this.speechEndTime > 0 && new Date().getTime() > this.speechEndTime) {
        this.$message.info("定时关闭朗读");
        this.speechEndTime = 0;
        return;
      }
      if (this.ttsType !== "local") {
        return this.speechCurrentByTTS();
      }
      if (!window.speechSynthesis) {
        return;
      }
      this.speechCurrent();
    },
    async speechCurrentByTTS() {
      const paragraph = this.getCurrentParagraph();
      if (!paragraph) {
        this.speechNext();
        return;
      }
      let text = paragraph.innerText;
      if (isOnlyWhitespace(text)) {
        setTimeout(() => this.speechNext(), 300);
        return;
      }
      text = stripWhitespace(text);
      if (!text) {
        this.speechNext();
        return;
      }
      const { config, cacheKey } = this.getTTSConfig(text);
      this.currentTTSCacheKey = cacheKey;
      let src = this.api + "/book/tts?" + this.serializeTTSConfig(config);
      if (this.cacheTTSAudio) {
        src = (await this.getCachedTTSAudioURL(cacheKey)) || src;
      }
      if (this.$refs.ttsAudio) {
        this.$refs.ttsAudio.src = src;
        this.$refs.ttsAudio.load();
        this.$refs.ttsAudio.play().catch(() => {});
      }
      this.showParagraph(paragraph, true);
      paragraph.className = "reading";
    },
    serializeTTSConfig(config) {
      return Object.keys(config)
        .map(
          key =>
            encodeURIComponent(key) +
            "=" +
            encodeURIComponent(config[key] == null ? "" : config[key])
        )
        .join("&");
    },
    getTTSConfig(text) {
      let pitch = this.speechPitch;
      let rate = this.speechRate;
      if (this.ttsType === "textToSpeechCn") {
        pitch = parseInt(50 * (pitch - 1));
        rate = parseInt(200 * (rate - 1));
      } else if (this.ttsType === "edge") {
        pitch = parseInt(50 * (pitch - 1));
      }
      const config = {
        text,
        type: this.ttsType,
        voice: this.voiceName,
        pitch: "" + pitch,
        rate: "" + rate,
        accessToken: this.$store.state.token
      };
      const book = this.$store.getters.readingBook || {};
      const cacheText =
        "" + text + config.type + config.voice + config.pitch + config.rate;
      return {
        config,
        cacheKey:
          "localCache@ttsData@" +
          book.name +
          "_" +
          book.author +
          "@" +
          cacheText.MD5(32)
      };
    },
    onTTSAudioPlay() {
      this.speechSpeaking = true;
      this.skipAutoNext = false;
      if (this.cacheTTSAudio) {
        this.cacheNextParagraphTTSAudio(1);
        this.cacheNextParagraphTTSAudio(2);
        this.cacheNextParagraphTTSAudio(3);
      }
    },
    onTTSAudioPause() {
      this.speechSpeaking = false;
      this.skipAutoNext = false;
    },
    onTTSAudioEnded() {
      if (this.currentTTSCacheKey) {
        window.$cacheStorage
          .removeItem(this.currentTTSCacheKey)
          .catch(() => {});
      }
      if (this.skipAutoNext) {
        this.skipAutoNext = false;
        this.speechSpeaking = false;
      } else {
        this.speechNext();
      }
    },
    onTTSAudioError(event) {
      if (this.speechSpeaking) {
        if (event.error || event.name) {
          this.$message.error(
            `朗读错误:  ${event.type || ""}  ${event.error ||
              event.name ||
              event.toString()}`
          );
        }
        this.speechSpeaking = false;
      }
    },
    async getCachedTTSAudioURL(cacheKey) {
      try {
        const base64 = await window.$cacheStorage.getItem(cacheKey).catch(() => {
          return false;
        });
        if (!base64) {
          return null;
        }
        return this.base64ToBlob(base64, "audio/mpeg");
      } catch (error) {
        return null;
      }
    },
    async cacheNextParagraphTTSAudio(distance) {
      const paragraph = this.getNextParagraph(distance);
      if (!paragraph) {
        return;
      }
      let text = paragraph.innerText;
      if (isOnlyWhitespace(text)) {
        return this.cacheNextParagraphTTSAudio(distance + 1);
      }
      text = stripWhitespace(text);
      if (!text) {
        return this.cacheNextParagraphTTSAudio(distance + 1);
      }
      const { config, cacheKey } = this.getTTSConfig(text);
      try {
        if (await window.$cacheStorage.getItem(cacheKey)) {
          return;
        }
        await Axios.post(
          this.api + "/book/tts",
          { ...config, base64: "1" },
          { silent: true }
        ).then(res => {
          if (res.data && res.data.isSuccess && res.data.data) {
            return window.$cacheStorage
              .setItem(cacheKey, res.data.data)
              .catch(() => {});
          }
          return null;
        });
      } catch (error) {
        // 预缓存失败不影响当前朗读
      }
    },
    async base64ToBlob(base64, type) {
      const binary = window.atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
      }
      const blob = new Blob([bytes], { type });
      // iPhone/iPad 上 createObjectURL 无法用于 audio 播放，退回 dataURL
      if (
        window.navigator.userAgent.indexOf("iPhone") >= 0 ||
        window.navigator.userAgent.indexOf("iPad") >= 0
      ) {
        return new Promise((resolve, reject) => {
          const reader = new FileReader();
          reader.onload = () => resolve(reader.result);
          reader.onerror = reject;
          reader.readAsDataURL(blob);
        });
      }
      return window.URL.createObjectURL(blob);
    },
    speechCurrent() {
      if (!window.speechSynthesis) {
        return;
      }
      const voice = this.voiceList.find(v => v.name === this.voiceName);
      if (!voice) {
        return;
      }
      const paragraph = this.getCurrentParagraph();
      if (!paragraph) {
        this.speechNext();
        return;
      }
      const text = stripWhitespace(paragraph.innerText);
      if (!text) {
        this.speechNext();
        return;
      }
      if (window.speechSynthesis.speaking) {
        try {
          window.speechSynthesis.cancel();
        } catch (error) {
          // 浏览器实现可能在取消时抛错
        }
      }

      this.utterance = new SpeechSynthesisUtterance(text);

      this.utterance.onstart = () => {
        this.speechSpeaking = true;
        this.skipAutoNext = false;
      };
      this.utterance.onend = () => {
        // 下一段
        if (!this.skipAutoNext) {
          this.speechNext();
        } else {
          this.skipAutoNext = false;
          this.speechSpeaking = false;
        }
      };
      this.utterance.onerror = event => {
        if (
          event.error !== "canceled" &&
          event.error !== "interrupted" &&
          (event.error || event.name)
        ) {
          this.$message.error(
            `朗读错误:  ${event.type || ""}  ${event.error ||
              event.name ||
              event.toString()}`
          );
        }
        setTimeout(() => {
          this.speechSpeaking = window.speechSynthesis.speaking || false;
        }, 200);
      };
      this.utterance.voice = voice;
      this.utterance.pitch = this.speechPitch;
      this.utterance.rate = this.speechRate;

      this.showParagraph(paragraph, true);
      paragraph.className = "reading";
      this.speechSpeaking = true;
      window.speechSynthesis.speak(this.utterance);
    },
    stopSpeech(clearCurrent) {
      try {
        this.skipAutoNext = true;
        if (this.ttsType === "local") {
          window.speechSynthesis.cancel();
        } else if (this.$refs.ttsAudio) {
          this.$refs.ttsAudio.pause();
        }
        if (clearCurrent) {
          const current = this.getCurrentParagraph();
          if (current) {
            current.className = "";
          }
        }
      } catch (error) {
        //
      }
    },
    restartSpeech() {
      this.stopSpeech();
      setTimeout(() => {
        this.startSpeech();
      }, 100);
    },
    toggleSpeech() {
      this.speechSpeaking ? this.stopSpeech(true) : this.startSpeech();
    },
    speechPrev() {
      if (
        this.speechSpeaking ||
        (window.speechSynthesis && window.speechSynthesis.speaking)
      ) {
        this.stopSpeech();
      }
      const current = this.getCurrentParagraph();
      const prev = this.getPrevParagraph(1);
      if (prev) {
        this.showParagraph(prev, true);
        current && (current.className = "");
        prev.className = "reading";
        this.startSpeech();
      } else {
        // 上一章
        this.onNextContentRendered(() => {
          setTimeout(() => {
            this.startSpeech();
          }, 100);
        });
        this.toLastChapter();
      }
    },
    speechNext() {
      if (
        this.speechSpeaking ||
        (window.speechSynthesis && window.speechSynthesis.speaking)
      ) {
        this.stopSpeech();
      }
      const current = this.getCurrentParagraph();
      const next = this.getNextParagraph(1);
      if (next) {
        this.showParagraph(next, true);
        current && (current.className = "");
        next.className = "reading";
        this.startSpeech();
      } else {
        // 下一章
        this.onNextContentRendered(() => {
          setTimeout(() => {
            this.startSpeech();
          }, 100);
        });
        this.toNextChapter();
      }
    },
    getCurrentParagraph() {
      if (!this.$refs.bookContentRef || !this.$refs.bookContentRef.$el) {
        return null;
      }
      const readingEle = this.getContentParagraphContainer().querySelectorAll(
        ".reading"
      );
      let currentParagraph = null;
      if (!readingEle.length) {
        // 没有正在读的段落，遍历找到当前页面的第一段
        const list = this.getContentParagraphList();
        for (let i = 0; i < list.length; i++) {
          const elePos = list[i].getBoundingClientRect();
          if (this.isSlideRead) {
            // 段尾出现在视野里
            if (elePos.right > 0) {
              currentParagraph = list[i];
              break;
            }
          } else {
            // 段尾出现在视野里
            let bottom = elePos.bottom;
            if (this.isEpubIframe) {
              bottom -= document.documentElement.scrollTop || document.body.scrollTop;
            }
            if (
              bottom >
              50 +
                (window.webAppDistance | 0) +
                (this.$store.state.safeArea.top | 0)
            ) {
              currentParagraph = list[i];
              break;
            }
          }
        }
      } else {
        currentParagraph = readingEle[0];
      }
      return currentParagraph;
    },
    getPrevParagraph(distance) {
      distance = distance || 1;
      const current = this.getCurrentParagraph();
      if (
        !current ||
        !this.$refs.bookContentRef ||
        !this.$refs.bookContentRef.$el
      ) {
        return null;
      }
      const list = this.getContentParagraphList();
      for (let i = 0; i < list.length; i++) {
        if (i > 0 && current === list[i]) {
          return list[i - distance] || null;
        }
      }
      return null;
    },
    getNextParagraph(distance) {
      distance = distance || 1;
      const current = this.getCurrentParagraph();
      if (
        !current ||
        !this.$refs.bookContentRef ||
        !this.$refs.bookContentRef.$el
      ) {
        return null;
      }
      const list = this.getContentParagraphList();
      for (let i = 0; i < list.length; i++) {
        if (current === list[i]) {
          return list[i + distance] || null;
        }
      }
      return null;
    },
    exitRead() {
      const current = this.getCurrentParagraph();
      this.stopSpeech(true);
      this.showReadBar = false;
      this.showParagraph(current);
    },
    showParagraph(paragraph, scroll) {
      if (!paragraph) {
        return;
      }
      if (this.isSlideRead) {
        // 跳转位置
        this.$nextTick(() => {
          const pos = paragraph.getBoundingClientRect();
          if (pos.left > this.windowSize.width - this.slideDistance) {
            this.showPage(
              Math.round(pos.left / (this.windowSize.width - this.slideDistance)) + 1,
              0
            );
          }
        });
      } else if (scroll) {
        // 跳转位置
        this.$nextTick(() => {
          const pos = paragraph.getBoundingClientRect();
          let top = pos.top;
          if (this.isEpubIframe) {
            top -= document.documentElement.scrollTop || document.body.scrollTop;
          }
          this.scrollContent(
            top -
              (this.$store.state.miniInterface
                ? this.getFirstParagraphPos().bottom
                : 0) -
              (window.webAppDistance | 0) -
              (this.$store.state.safeArea.top | 0),
            0
          );
        });
      }
    },
    getFirstParagraphPos() {
      return this.$refs.top.getBoundingClientRect();
    },
    captureScrollAnchor() {
      if (
        !this.isScrollRead ||
        !this.$refs.bookContentRef ||
        !this.$refs.bookContentRef.$el
      ) {
        return null;
      }
      const paragraph = this.getCurrentParagraph();
      if (!paragraph || !paragraph.dataset) {
        return null;
      }
      const chapter = this.findChapterElement(paragraph);
      if (
        !chapter ||
        !chapter.dataset ||
        typeof chapter.dataset.index === "undefined"
      ) {
        return null;
      }
      if (typeof paragraph.dataset.pos === "undefined") {
        return null;
      }
      return {
        chapterIndex: +chapter.dataset.index,
        paragraphPos: +paragraph.dataset.pos,
        offsetTop: paragraph.getBoundingClientRect().top
      };
    },
    restoreScrollAnchor(anchor) {
      if (
        !anchor ||
        !this.isScrollRead ||
        !this.$refs.bookContentRef ||
        !this.$refs.bookContentRef.$el
      ) {
        return;
      }
      const container = this.$refs.bookContentRef.$el;
      const chapter = container.querySelector(
        `.chapter-content[data-index="${anchor.chapterIndex}"]`
      );
      if (!chapter) {
        return;
      }
      const paragraph = chapter.querySelector(
        `[data-pos="${anchor.paragraphPos}"]`
      );
      if (!paragraph) {
        return;
      }
      const newTop = paragraph.getBoundingClientRect().top;
      const rawDelta = newTop - anchor.offsetTop;
      const scrollElement =
        document.scrollingElement || document.documentElement || document.body;
      const currentScroll =
        scrollElement.scrollTop ||
        document.documentElement.scrollTop ||
        document.body.scrollTop ||
        0;
      const viewportHeight =
        (window.visualViewport && window.visualViewport.height) ||
        window.innerHeight ||
        this.windowSize.height ||
        0;
      const maxScroll = Math.max(
        0,
        (scrollElement.scrollHeight ||
          document.documentElement.scrollHeight ||
          document.body.scrollHeight ||
          0) - viewportHeight
      );
      const clampedTarget = Math.max(
        0,
        Math.min(currentScroll + rawDelta, maxScroll)
      );
      const adjustment = clampedTarget - currentScroll;
      if (Math.abs(adjustment) > 1) {
        // Keep the paragraph the user is reading at the same viewport offset.
        this.scrollContent(adjustment, 0);
      }
    },
    findChapterElement(node) {
      let current = node;
      while (current && current !== this.$refs.bookContentRef.$el) {
        if (
          current.classList &&
          current.classList.contains("chapter-content")
        ) {
          return current;
        }
        current = current.parentNode;
      }
      return null;
    },
    scrollHandler() {
      const scrollTop =
        document.documentElement.scrollTop || document.body.scrollTop;
      if (!this.isSlideRead) {
        this.currentPage = Math.round(
          (scrollTop + this.windowSize.height) /
            (this.windowSize.height - this.scrollOffset)
        );
      }
      if (this.isScrollRead) {
        const lastScrollTop = this.lastScrollTop || 0;
        if (lastScrollTop > 0 && scrollTop == 0) {
          // 往上滚动到顶，不做处理
        } else if (
          scrollTop >
            document.documentElement.scrollHeight - 2 * this.windowSize.height && // 倒数第二页
          !this.preCaching &&
          this.startSavePosition
        ) {
          // 往下滚动到 倒数第二页
          this.preCaching = true;
          let nextIndex = this.chapterIndex + 1;
          if (this.showChapterList.length) {
            nextIndex =
              this.showChapterList[this.showChapterList.length - 1].index + 1;
          }
          this.showNextChapterSize = nextIndex - this.chapterIndex;
          this.loadShowChapter(nextIndex)
            .then(() => {
              this.computeShowChapterList();
              this.preCaching = false;
            })
            .catch(() => {
              this.preCaching = false;
            });
        }
      }
      this.lastScrollTop = scrollTop;
      this.scrollTimer && clearTimeout(this.scrollTimer);
      this.scrollTimer = setTimeout(this.saveReadingPosition, 100);
    },
    beforeReadMethodChange() {
      this.currentParagraph = this.getCurrentParagraph();
    },
    // 只会在进入的时候调用
    showPosition(pos, callback) {
      if (!this.$refs.bookContentRef) {
        setTimeout(() => {
          this.showPosition(pos, callback);
        }, 10);
        return;
      }
      if (this.isAudio || this.isVideo) {
        // seek
        this.$refs.bookContentRef.ensureSeekTime(pos);
        callback && callback();
      } else if (this.isEpub || this.isCarToon) {
        // 跳转
        this.scrollContent(pos, 0, true);
        callback && callback();
      } else {
        const list = this.getContentParagraphList(true);
        for (let i = 0; i < list.length; i++) {
          if (
            list[i].dataset &&
            typeof list[i].dataset.pos !== "undefined" &&
            +list[i].dataset.pos >= pos
          ) {
            this.showParagraph(list[i], true);
            break;
          }
        }
        callback && callback();
      }
    },
    saveReadingPosition() {
      try {
        if (this.error || !this.startSavePosition) {
          return;
        }
        // 保存页码
        setCache(
          "bookChapterPage@" +
            this.$store.getters.readingBook.name +
            "_" +
            this.$store.getters.readingBook.author,
          this.currentPage
        );
        let position = null;
        if (this.isAudio || this.isVideo) {
          position = this.$refs.bookContentRef
            ? this.$refs.bookContentRef.currentTime
            : 0;
        } else if (this.isEpub || this.isCarToon) {
          position =
            document.documentElement.scrollTop || document.body.scrollTop;
        } else {
          // 更新当前章节 和 当前段落
          if (this.preCaching) {
            return;
          }
          this.currentParagraph = this.getCurrentParagraph();
          if (this.currentParagraph) {
            // 找到最近的 .chapter-content
            let currentChapter = this.currentParagraph;
            while (currentChapter.className.indexOf("chapter-content") < 0) {
              currentChapter = currentChapter.parentNode;
              if (currentChapter === this.$refs.bookContentRef.$el) {
                break;
              }
            }
            if (currentChapter) {
              if (
                currentChapter.dataset &&
                typeof currentChapter.dataset.index !== "undefined"
              ) {
                const chapterIndex = +currentChapter.dataset.index;
                if (chapterIndex != this.$store.getters.readingBook.index) {
                  let book = { ...this.$store.getters.readingBook };
                  book.index = chapterIndex;
                  this.$store.commit("setReadingBook", book);
                  // 保存阅读进度
                  this.saveBookProgress(chapterIndex);
                  this.title = this.$store.getters.readingBook.catalog[
                    chapterIndex
                  ].title;
                }
              }
              position = currentChapter.innerText.indexOf(
                this.currentParagraph.innerText
              );
            }
          }
        }
        if (position !== null) {
          setCache(
            "bookChapterProgress@" +
              this.$store.getters.readingBook.name +
              "_" +
              this.$store.getters.readingBook.author,
            position
          );
        }
      } catch (error) {
        //
      }
    },
    autoShowPosition(immediate) {
      const handler = () => {
        setTimeout(() => {
          this.startSavePosition = true;
        }, 2000);
        if (this.error) {
          return;
        }
        const lastPosition = getCache(
          "bookChapterProgress@" +
            this.$store.getters.readingBook.name +
            "_" +
            this.$store.getters.readingBook.author
        );
        if (lastPosition && +lastPosition) {
          this.$nextTick(() => {
            this.showPosition(+lastPosition, () => {
              this.startSavePosition = true;
            });
          });
        } else {
          // 没有段落进度，尝试恢复页码
          const lastPage = getCache(
            "bookChapterPage@" +
              this.$store.getters.readingBook.name +
              "_" +
              this.$store.getters.readingBook.author
          );
          if (lastPage && +lastPage) {
            this.$nextTick(() => {
              this.computePages(() => {
                this.showPage(+lastPage, 0);
              });
            });
          }
        }
      };
      if (immediate) {
        handler();
      } else {
        this.onNextContentRendered(handler);
      }
    },
    onNextContentRendered(callback) {
      if (this.isEpub) {
        this.$once("iframeLoad", () => {
          this.$nextTick(() => {
            callback && callback();
          });
        });
      } else if (this.isCarToon) {
        this.$once("lazyload", () => {
          this.$nextTick(() => {
            callback && callback();
          });
        });
      } else {
        this.$once("showContent", () => {
          this.$nextTick(() => {
            callback && callback();
          });
        });
      }
    },
    onNextContentPagesInit(callback) {
      if (this.isEpub) {
        this.$once("iframeLoad", () => {
          this.$nextTick(() => {
            callback && callback();
          });
        });
        return;
      }
      let contentReady = false;
      this.$once("showContent", () => {
        contentReady = true;
      });
      const handler = () => {
        if (contentReady) {
          this.$off("afterComputePages", handler);
          callback && callback();
        }
      };
      this.$on("afterComputePages", handler);
    },
    wakeLock() {
      if ("WakeLock" in window && "request" in window.WakeLock) {
        let wakeLock = null;
        const requestWakeLock = () => {
          const controller = new AbortController();
          const signal = controller.signal;
          window.WakeLock.request("screen", { signal }).catch(e => {
            if (e.name === "AbortError") {
              //
            }
          });
          return controller;
        };

        wakeLock = requestWakeLock();

        const handleVisibilityChange = () => {
          if (wakeLock !== null && document.visibilityState === "visible") {
            wakeLock = requestWakeLock();
          }
        };

        document.addEventListener("visibilitychange", handleVisibilityChange);
        document.addEventListener("fullscreenchange", handleVisibilityChange);
        return () => {
          if (wakeLock != null) {
            wakeLock.abort();
            wakeLock = null;
          }
          document.removeEventListener(
            "visibilitychange",
            handleVisibilityChange
          );
          document.removeEventListener(
            "fullscreenchange",
            handleVisibilityChange
          );
        };
      } else if ("wakeLock" in navigator && "request" in navigator.wakeLock) {
        let wakeLock = null;
        const requestWakeLock = async () => {
          try {
            wakeLock = await navigator.wakeLock.request("screen");
            wakeLock.addEventListener("release", () => {
              //
            });
          } catch (e) {
            //
          }
        };
        requestWakeLock();
        const handleVisibilityChange = () => {
          if (wakeLock !== null && document.visibilityState === "visible") {
            requestWakeLock();
          }
        };
        document.addEventListener("visibilitychange", handleVisibilityChange);
        document.addEventListener("fullscreenchange", handleVisibilityChange);
        return () => {
          if (wakeLock != null) {
            wakeLock.release();
            wakeLock = null;
          }
          document.removeEventListener(
            "visibilitychange",
            handleVisibilityChange
          );
          document.removeEventListener(
            "fullscreenchange",
            handleVisibilityChange
          );
        };
      }
    },
    lazyloadHandler() {
      if (!this.isAudio && !this.isVideo) {
        this.computePages();
        this.$emit("lazyload");
      }
    },
    setMobileScrollBarHidden(hidden) {
      if (typeof document === "undefined") {
        return;
      }
      const body = document.body;
      const html = document.documentElement;
      if (!body || !html) {
        return;
      }
      if (hidden) {
        body.classList.add("mobile-scroll-read");
        html.classList.add("mobile-scroll-read");
      } else {
        body.classList.remove("mobile-scroll-read");
        html.classList.remove("mobile-scroll-read");
      }
    },
    showCacheContent() {
      this.showCacheContentZone = !this.showCacheContentZone;
    },
    cacheChapterContent(cacheCount) {
      let cacheChapterList = [];
      if (cacheCount === true) {
        cacheChapterList = cacheChapterList.concat(
          this.catalog.slice(this.chapterIndex + 1, this.catalog.length)
        );
      } else {
        cacheChapterList = cacheChapterList.concat(
          this.catalog.slice(
            this.chapterIndex + 1,
            Math.min(this.catalog.length, this.chapterIndex + 1 + cacheCount)
          )
        );
      }
      if (!cacheChapterList.length) {
        this.$message.error("不需要缓存");
        return;
      }
      this.isCachingContent = true;
      this.cachingContentTip = "正在缓存章节  0/" + cacheChapterList.length;
      this.cachingHandler = LimitResquest(2, handler => {
        this.cachingContentTip =
          "正在缓存章节  " +
          handler.requestCount +
          "/" +
          cacheChapterList.length;
        if (handler.isEnd()) {
          this.$message.success("缓存完成");
          this.isCachingContent = false;
          this.cachingContentTip = "";
        }
      });
      cacheChapterList.forEach(v => {
        this.cachingHandler(() => {
          return this.getBookContent(
            v.index,
            {
              timeout: 1000 * this.$store.getters.config.chapterRequestTimeout,
              silent: true
            },
            false,
            true
          );
        });
      });
    },
    cancelCaching() {
      if (this.cachingHandler && this.cachingHandler.cancel) {
        this.cachingHandler.cancel();
        this.isCachingContent = false;
        this.cachingContentTip = "";
      }
    },
    startAutoReading() {
      this.showToolBar = false;
      this.autoReading = true;
      this.$nextTick(() => {
        this.autoRead();
      });
    },
    autoRead() {
      if (!this.autoReading) {
        return;
      }
      if (this.showToolBar) {
        this.autoReadingTimer = setTimeout(() => {
          this.autoRead();
        }, 300);
        return;
      }
      if (this.config.autoReadingMethod === "像素滚动") {
        this.autoReadByPixel();
        return;
      }
      const current = this.getCurrentParagraph();
      const next = this.getNextParagraph();
      if (next) {
        current.className = "reading";
        next.className = "";
        // 计算当前段落
        let delayTime = this.config.autoReadingLineTime;
        try {
          const currentPos = current.getBoundingClientRect();
          delayTime =
            delayTime *
            Math.ceil(
              currentPos.height / this.config.fontSize / this.config.lineHeight
            );
        } catch (error) {
          //
        }
        this.autoReadingTimer = setTimeout(() => {
          current.className = "";
          next.className = "reading";
          this.showParagraph(next, true);

          setTimeout(() => {
            this.autoRead();
          }, 32);
        }, delayTime);
      } else {
        // 下一章
        this.onNextContentRendered(() => {
          setTimeout(() => {
            this.autoRead();
          }, 100);
        });
        this.toNextChapter(() => {
          this.autoReading = false;
        });
      }
    },
    autoReadByPixel() {
      if (!this.autoReading) {
        return;
      }
      if (this.showToolBar) {
        this.autoReadingTimer = setTimeout(() => {
          this.autoRead();
        }, 300);
        return;
      }
      if (this.config.autoReadingMethod !== "像素滚动") {
        this.autoRead();
        return;
      }
      const scrollTop =
        document.documentElement.scrollTop || document.body.scrollTop;
      if (
        scrollTop + this.windowSize.height <
        document.documentElement.scrollHeight
      ) {
        this.autoReadingTimer = setTimeout(() => {
          // 滚动
          this.scrollContent(this.config.autoReadingPixel, 0);
          this.autoReadByPixel();
        }, this.config.autoReadingLineTime);
      } else {
        // 下一章
        this.onNextContentRendered(() => {
          setTimeout(() => {
            this.autoReadByPixel();
          }, 100);
        });
        this.toNextChapter(() => {
          this.autoReading = false;
        });
      }
    },
    stopAutoReading() {
      if (this.autoReadingTimer) {
        clearInterval(this.autoReadingTimer);
      }
      this.autoReading = false;
      const current = this.getCurrentParagraph();
      current && (current.className = "");
    },
    toggleAutoReading() {
      if (this.autoReading) {
        this.stopAutoReading();
      } else {
        this.startAutoReading();
      }
    },
    showReadingBookInfo() {
      let book = { ...this.$store.getters.readingBook };
      const shelfBook = this.$store.getters.shelfBooks.find(
        v => v.bookUrl === book.bookUrl
      );
      book = Object.assign(book, shelfBook || {});
      eventBus.$emit("showBookInfoDialog", book);
    },
    formatChinese(text) {
      if (
        this.isEpub ||
        this.isAudio ||
        this.isVideo ||
        this.isCbz ||
        this.isCarToon
      ) {
        return text;
      }
      if (this.config.chineseFont === "简体") {
        return simplized(text);
      } else if (this.config.chineseFont === "繁体") {
        return traditionalized(text);
      }
      // 原文
      return text;
    },
    showSearchBookContentDialog() {
      let book = { ...this.$store.getters.readingBook };
      const shelfBook = this.$store.getters.shelfBooks.find(
        v => v.bookUrl === book.bookUrl
      );
      book = Object.assign(book, shelfBook || {});
      eventBus.$emit("showSearchBookContentDialog", book);
    },
    showMatchKeyword(data) {
      if (this._inactive) {
        return;
      }
      if (!this.$refs.bookContentRef) {
        setTimeout(() => {
          this.showMatchKeyword(data);
        }, 10);
        return;
      }
      if (this.isEpubResolve && !(this.$refs.bookContentRef.$el && this.$refs.bookContentRef.$el.shadowRoot)) {
        setTimeout(() => {
          this.showMatchKeyword(data);
        }, 10);
        return;
      }
      if (this.isEpubIframe) {
        try {
          if (
            !this.$refs.bookContentRef.$el ||
            !this.$refs.bookContentRef.$el.contentWindow ||
            !this.$refs.bookContentRef.$el.contentWindow.document ||
            !this.$refs.bookContentRef.$el.contentWindow.document.querySelectorAll("body")
          ) {
            setTimeout(() => {
              this.showMatchKeyword(data);
            }, 10);
            return;
          }
        } catch (error) {
          return;
        }
      }
      try {
        const list = this.getContentParagraphList(true);
        let matchCount = 0;
        for (let i = 0; i < list.length; i++) {
          const pContent = list[i].innerText;
          let startIndex = -1;
          let isFound = false;
          // eslint-disable-next-line no-constant-condition
          while (true) {
            startIndex = pContent.indexOf(data.query, startIndex + 1);
            if (startIndex >= 0) {
              matchCount++;
              if (matchCount === data.resultCountWithinChapter + 1) {
                isFound = true;
                this.showParagraph(list[i], true);
                break;
              }
            } else {
              break;
            }
          }
          if (isFound) {
            break;
          }
        }
      } catch (error) {
        // console.error(error);
      }
    },
    getParagraphListInView() {
      // 获取视口内的所有段落
      const list = this.getContentParagraphList();
      const paragraphList = [];
      for (let i = 0; i < list.length; i++) {
        const elePos = list[i].getBoundingClientRect();
        if (this.isSlideRead) {
          // 段尾出现在视野里
          if (elePos.right > 0 && elePos.left > 0) {
            paragraphList.push(list[i]);
          }
        } else {
          // 段尾出现在视野里
          if (
            elePos.bottom >
              50 +
                (window.webAppDistance | 0) +
                (this.$store.state.safeArea.top | 0) &&
            elePos.bottom < this.windowSize.height
          ) {
            paragraphList.push(list[i]);
          }
        }
      }
      return paragraphList;
    },
    showBookmarkDialog() {
      let book = { ...this.$store.getters.readingBook };
      const shelfBook = this.$store.getters.shelfBooks.find(
        v => v.bookUrl === book.bookUrl
      );
      book = Object.assign(book, shelfBook || {});
      eventBus.$emit("showBookmarkDialog", book);
    },
    getContentParagraphContainer() {
      if (!this.isEpub) {
        return this.$refs.bookContentRef.$el;
      }
      if (!this.isEpubIframe) {
        return this.$refs.bookContentRef.$el.shadowRoot;
      }
      try {
        return this.$refs.bookContentRef.$el.contentWindow.document;
      } catch (error) {
        return this.$refs.bookContentRef.$el;
      }
    },
    getContentParagraphList(showOnlyReadingChapter) {
      let list;
      if (this.isEpub) {
        list = this.getContentParagraphContainer().querySelectorAll(
          "h1,h2,h3,h4,p"
        );
      } else {
        list = this.getContentParagraphContainer().querySelectorAll(
          showOnlyReadingChapter ? ".reading-chapter h3,p" : "h3,p"
        );
      }
      return list;
    },
    getContentMatchParagraph(text, distance, minDistance) {
      distance = distance || 0.7;
      // 正则过滤标点符号后，近似匹配每一段内容
      let paragraphList = text
        .replace(/\\n+/g, "\n")
        .split(/\n+/)
        .map(v => v.replace(symboRegex, ""))
        .filter(v => v);
      try {
        const list = this.getContentParagraphList(true);
        let paragraph = null;
        for (let i = 0; i < list.length; i++) {
          let isMatch = true;
          let pos = 0;
          let startPos = i;
          for (let j = 0; j < paragraphList.length; j++) {
            // 过滤所有字符
            let content = null;
            while (i + pos < list.length) {
              content = list[i + pos].innerText.replace(symboRegex, "");
              if (!content.length) {
                pos++;
                startPos++;
              } else {
                break;
              }
            }
            if (!content) {
              // 说明没找到有内容的段落，终止匹配
              isMatch = false;
              break;
            }
            const paragraphDistance = editDistance(content, paragraphList[j]);
            if (paragraphDistance < distance) {
              isMatch = false;
              break;
            } else {
              pos++;
            }
          }
          if (isMatch) {
            paragraph = list[startPos];
            break;
          }
        }
        if (paragraph) {
          return paragraph;
        }
        if (distance - 0.1 >= minDistance) {
          return this.getContentMatchParagraph(
            text,
            distance - 0.1,
            minDistance
          );
        }
      } catch (error) {
        // eslint-disable-next-line no-console
        console.error(error);
      }
      return null;
    },
    showContentMatchParagraph(content) {
      if (this._inactive) {
        return;
      }
      const paragraph = this.getContentMatchParagraph(content, 1, 0.6);
      if (paragraph) {
        this.showParagraph(paragraph, true);
      } else {
        this.$message.error("无法定位内容所在段落");
      }
    },
    showBookmark(bookmark) {
      if (this._inactive) {
        return;
      }
      if (!this.$refs.bookContentRef) {
        setTimeout(() => {
          this.showBookmark(bookmark);
        }, 10);
        return;
      }
      if (
        this.isEpubResolve &&
        !(this.$refs.bookContentRef.$el && this.$refs.bookContentRef.$el.shadowRoot)
      ) {
        setTimeout(() => {
          this.showBookmark(bookmark);
        }, 10);
        return;
      }
      if (this.isEpubIframe) {
        try {
          if (
            !this.$refs.bookContentRef.$el ||
            !this.$refs.bookContentRef.$el.contentWindow ||
            !this.$refs.bookContentRef.$el.contentWindow.document ||
            !this.$refs.bookContentRef.$el.contentWindow.document.querySelectorAll("body")
          ) {
            setTimeout(() => {
              this.showBookmark(bookmark);
            }, 10);
            return;
          }
        } catch (error) {
          return;
        }
      }
      this.showContentMatchParagraph(bookmark.bookText);
    },
    readOriginal() {
      if (this.isPdf && this.$store.getters.readingBook.originName) {
        if (
          this.$store.getters.readingBook.originName.indexOf("localStore") >= 0 ||
          this.$store.getters.readingBook.originName.indexOf("webdav") >= 0
        ) {
          const url =
            this.api +
            "/file/download?" +
            this.serializeTTSConfig({
              home:
                this.$store.getters.readingBook.originName.indexOf(
                  "localStore"
                ) >= 0
                  ? "__LOCAL_STORE__"
                  : "__WEBDAV__",
              path: this.$store.getters.readingBook.originName.replace(
                /^.*(localStore|webdav)/g,
                ""
              ),
              stream: 1,
              accessToken: this.$store.state.token
            });
          window.open(url, "__blank");
        } else {
          const bookUrl = this.$store.getters.readingBook.bookUrl.replace(
            "storage/data",
            "book-assets"
          );
          window.open(bookUrl, "__blank");
        }
      }
    }
  }
};
export const style = `>>>.popper-component {
  margin-left: 10px;
}

.dplayer-quality-list {
  max-height: 45vh;
  overflow: auto !important;
}

.chapter-wrapper {
  padding: 0;
  flex-direction: column;
  align-items: center;

  >>>.no-point {
    pointer-events: none;
  }

  .tool-bar {
    position: fixed;
    top: 0;
    padding-top: 0;
    padding-top: constant(safe-area-inset-top) !important;
    padding-top: env(safe-area-inset-top) !important;
    left: 50%;
    z-index: 2001;

    .tools {
      display: flex;
      flex-direction: column;

      .tool-icon {
        font-size: 18px;
        width: 58px;
        height: 48px;
        text-align: center;
        padding-top: 12px;
        cursor: pointer;
        outline: none;

        .iconfont {
          font-family: iconfont;
          width: 16px;
          font-size: 16px;
          margin: 0 auto;
          height: 22px;
          line-height: 22px;
          vertical-align: middle;
        }

        .tool-el-icon {
          font-size: 18px;
          line-height: 22px;
          height: 22px;

          i {
            line-height: 22px;
          }
        }

        .icon-text {
          font-size: 12px;
        }
      }
    }
  }

  .read-bar {
    position: fixed;
    bottom: 0;
    right: 50%;
    z-index: 100;

    .progress {
      padding: 10px 36px;
      display: flex;
      justify-content: space-between;
      align-items: center;

      .progress-bar {
        flex: 1;
        padding: 0 10px;
      }

      .progress-tip {
        font-size: 14px;
        margin-left: 5px;
      }
    }

    .cache-content-zone {
      padding: 10px 36px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      font-size: 14px;
      position: absolute;
      right: 55px;
      width: 300px;
      background: inherit;

      .cache-content-btn {
        cursor: pointer;
      }
    }

    .float-left-btn-zone {
      position: absolute;
      bottom: 155px;
      left: 4px;
      right: auto;
      display: flex;
      flex-direction: column;

      .float-btn {
        line-height: 32px;
        width: 36px;
        height: 36px;
        border-radius: 100%;
        display: block;
        cursor: pointer;
        text-align: center;
        vertical-align: middle;
        pointer-events: all;
        margin-top: 20px;

        .el-icon-top, .el-icon-bottom, .el-icon-info, .el-icon-search, .el-icon-collection-tag {
          line-height: 36px;
        }
      }
    }

    .float-right-btn-zone {
      position: absolute;
      bottom: 155px;
      left: 4px;
      right: auto;
      display: flex;
      flex-direction: column;

      .float-btn {
        line-height: 32px;
        width: 36px;
        height: 36px;
        border-radius: 100%;
        display: block;
        cursor: pointer;
        text-align: center;
        vertical-align: middle;
        pointer-events: all;
        margin-top: 20px;

        .el-icon-refresh-right, .el-icon-headset, .el-icon-view, .el-icon-edit, .el-icon-download, .el-icon-reading {
          line-height: 36px;
        }
        .el-icon-moon {
          color: #121212;
          line-height: 34px;
        }
        .el-icon-sunny {
          color: #666;
          line-height: 34px;
        }
      }

      .auto-reading, .editing {
        color: red;
      }
    }

    .tools {
      display: flex;
      flex-direction: column;

      .tool-icon {
        font-size: 18px;
        width: 42px;
        height: 31px;
        padding-top: 12px;
        text-align: center;
        align-items: center;
        cursor: pointer;
        outline: none;
        margin-top: -1px;

        &.progress-text {
          font-size: 16px;
        }

        .iconfont {
          font-family: iconfont;
          width: 16px;
          height: 16px;
          font-size: 16px;
          margin: 0 auto 6px;
        }
      }
    }

    .reader-bar-inner {
      display: flex;
      flex-direction: column;
      padding-bottom: 10px;
      padding-bottom: calc(10px + constant(safe-area-inset-top));
      padding-bottom: calc(10px + env(safe-area-inset-top));
      padding-left: 5px;
      padding-right: 5px;

      .operate-bar {
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        padding: 10px 10px 0 10px;
        align-items: center;

        .close-btn, .collapse-btn {
          font-size: 22px;
          height: 35px;
          cursor: pointer;
        }

        .center {
          span {
            display: inline-block;
            cursor: pointer;
          }
          .play-pause-btn {
            font-size: 50px;
            margin-top: -40px;
            i {
              border-radius: 100%;
            }
          }
          .ctrl-btn {
            margin: 0px 15px;
          }
        }
      }

      .setting-item {
        display: flex;
        flex-direction: column;
        padding: 5px 10px;

        .setting-title {
          font-size: 14px;
        }

        .setting-btn {
          font-size: 14px;
          cursor: pointer;
          display: inline-block;
          margin-left: 5px;
        }

        .voice-list {
          display: flex;
          flex-direction: row;
          overflow-x: auto;
          padding: 5px 10px;

          .radio-group {
            white-space: nowrap;

            .radio-button {
              margin-right: 10px;

              .el-radio-button__inner {
                border-radius: 4px 4px 4px 4px;
              }
            }
          }

          .voice-select {
            margin: 0 auto;
            width: 240px;
          }
        }

        .progress {
          padding: 5px 10px;

          .progress-tip {
            margin-left: 0;
            margin-right: 5px;
          }
        }
      }
    }
  }

  .chapter-bar {
    .el-breadcrumb {
      .item {
        font-size: 14px;
        color: #606266;
      }
    }
  }

  .chapter {
    font-family: 'Microsoft YaHei', PingFangSC-Regular, HelveticaNeue-Light, 'Helvetica Neue Light', sans-serif;
    text-align: left;
    padding: 0 65px;
    min-height: 100vh;
    min-height: calc(var(--vh, 1vh) * 100);
    width: 670px;
    margin: 0 auto;
    background-size: cover;
    position: relative;

    >>>.el-icon-loading {
      font-size: 36px;
      color: #B5B5B5;
    }

    >>>.el-loading-text {
      font-weight: 500;
      color: #B5B5B5;
    }

    .click-zone {
      position: absolute;
      z-index: 120;
      top: 0;
      bottom: 0;
      left: 0;
      right: 0;
      background: #333;
      opacity: 0.8;
      color: #fff;
      font-size: 14px;
      pointer-events: none;

      div {
        position: absolute;
        text-align: center;
        display: flex;
        align-items: center;
        justify-content: center;
      }

      .close-btn {
        left: 0;
        right: 0;
        bottom: 20px;
        height: 45px;
        line-height: 45px;
        z-index: 10;
        padding: 0;
        cursor: pointer;
        pointer-events: all;
      }
    }

    .content {
      font-size: 18px;
      line-height: 1.8;
      overflow: hidden;
      font-family: 'Microsoft YaHei', PingFangSC-Regular, HelveticaNeue-Light, 'Helvetica Neue Light', sans-serif;

      .content-inner {
        min-height: calc(var(--vh, 1vh) * 80);
        padding-bottom: 25px;
        box-sizing: border-box;
      }
    }

    .bottom-bar, .top-bar {
      box-sizing: border-box;
    }
    .top-bar {
      height: 44px;
      padding: 10px;

      .right-timestr {
        display: inline-block;
        float: right;
      }
    }
    .bottom-bar {
      width: 100%;
      text-align: center;
      padding-bottom: 30px;
      .bottom-btn {
        font-size: 14px;
        cursor: pointer;
        display: inline-block;
        margin: 0 auto;
        padding: 10px 40px;
        width: 80%;
        box-sizing: border-box;
      }
    }
  }

  .chapter.audio {
    .top-bar, .bottom-bar {
      display: none;
    }
    .content-inner {
      height: calc(var(--vh, 1vh) * 100);
      margin-top: 0 !important;
      padding-top: 0 !important;
      padding-bottom: 0 !important;
      display: flex;
      align-items: center;
    }
  }

  .chapter.video {
    .top-bar, .bottom-bar {
      display: none;
    }
    .content-inner {
      height: calc(var(--vh, 1vh) * 100);
      margin-top: 0 !important;
      padding-top: 0 !important;
      padding-bottom: 0 !important;
      display: flex;
      align-items: center;

      .book-content {
        width: 100%;
        height: 100%;
      }
    }
  }
}

.day {
  >>>.popup {
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.12), 0 0 6px rgba(0, 0, 0, 0.04);
  }

  >>>.tool-icon {
    border: 1px solid rgba(0, 0, 0, 0.1);
    margin-top: -1px;
    color: #000;

    .icon-text {
      color: rgba(0, 0, 0, 0.4);
    }
  }

  >>>.progress-tip {
    color: rgba(0, 0, 0, 0.4);
  }

  >>>.cache-content-zone {
    color: rgba(0, 0, 0, 0.4);
  }

  >>>.float-left-btn-zone {
    color: #121212;
  }

  >>>.float-right-btn-zone {
    color: #121212;
  }

  >>>.reader-bar-inner {
    color: #121212;

    .setting-title {
      color: rgba(0, 0, 0, 0.8);
    }

    .setting-value {
      color: rgba(0, 0, 0, 0.4);
    }
  }

  >>>.chapter {
    border: 1px solid #d8d8d8;
    color: #262626;
  }

  .bottom-bar, .top-bar {
    color: rgba(0, 0, 0, 0.4);
  }

  >>>.el-slider__runway {
    background-color: #fff;
  }

  >>>.play-pause-btn {
    color: #409EFF;
  }
}

.night {
  >>>.popup {
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.48), 0 0 6px rgba(0, 0, 0, 0.16);
  }

  >>>.tool-icon {
    border: 1px solid #444;
    margin-top: -1px;
    color: #666;

    .icon-text {
      color: #666;
    }
  }

  >>>.progress-tip {
    color: #666;
  }

  >>>.cache-content-zone {
    color: #666;
  }

  >>>.float-left-btn-zone {
    color: #666;
  }

  >>>.float-right-btn-zone {
    color: #666;
  }

  >>>.reader-bar-inner {
    color: #666;
  }

  >>>.chapter {
    border: 1px solid #444;
    color: #666;
  }

  >>>.popper__arrow {
    background: #666;
  }

  .bottom-bar, .top-bar {
    color: #666;
  }

  >>>.el-slider__runway {
    background-color: #282828;
  }
  >>>.el-slider__bar {
    background-color: #185798;
  }
  >>>.el-slider__button {
    border: 2px solid #185798;
    background-color: #282828;
  }
  >>>.play-pause-btn {
    color: #185798;
  }
}

.chapter-wrapper {
  .read-bar {
    .float-btn-zone {
      position: absolute;
      bottom: 135px;
      left: 4px;

      .float-left-btn-zone {
        position: relative;
        left: auto;
        bottom: auto;
      }

      .float-right-btn-zone {
        position: relative;
        left: auto;
        bottom: auto;
        margin-bottom: 20px;
      }
    }

  }
}

.chapter-wrapper.mini-interface {
  padding: 0;
  position: relative;
  height: 100%;

  .tool-bar {
    left: 0;
    width: 100vw;
    margin-left: 0 !important;

    .tools {
      flex-direction: row;
      justify-content: space-around;
      .tool-icon {
        border: none;
      }
    }
  }

  .read-bar {
    right: 0;
    width: 100vw;
    margin-right: 0 !important;

    .cache-content-zone {
      position: relative;
      width: auto;
      right: 0;
      background: inherit;
    }

    .float-btn-zone {
      position: static;
      bottom: 0;
      left: 0;
    }

    .float-left-btn-zone {
      position: absolute;
      right: auto;
      left: 20px;
      bottom: 135px;
    }

    .float-right-btn-zone {
      position: absolute;
      left: auto;
      right: 20px;
      bottom: 135px;
    }

    .tools {
      flex-direction: row;
      justify-content: space-around;
      padding: 0 15px;
      height: 45px;
      .tool-icon {
        border: none;
        width: auto;
        padding: 0;
        height: 45px;
        line-height: 45px;
        .iconfont {
          display: inline-block;
        }
        span {
          vertical-align: middle;
        }
      }
    }
  }

  .chapter {
    width: 100vw !important;
    padding: 0 16px;
    box-sizing: border-box;
    border: none;
    text-align: justify;
    position: relative;

    .top-bar {
      position: fixed;
      top: 0;
      left: 0;
      width: 100vw;
      z-index: 50;
      background: inherit;
      height: 30px;
      height: calc(30px + constant(safe-area-inset-top));
      height: calc(30px + env(safe-area-inset-top));
      padding: 6px 16px;
      padding-top: calc(6px + constant(safe-area-inset-top));
      padding-top: calc(6px + env(safe-area-inset-top));
      font-size: 12px;
    }

    .content-inner {
      margin-top: 30px;
      margin-top: calc(30px + constant(safe-area-inset-top));
      margin-top: calc(30px + env(safe-area-inset-top));
      padding-top: 15px;
      padding-bottom: 15px;
    }
  }

  .chapter.cartoon {
    padding: 0;

    .content-inner {
      padding-top: 1px;
    }
  }

  .chapter.slide-reader {
    padding: 0;
    height: 100%;

    .bottom-bar {
      height: calc(24px + var(--bottom-padding, 0px));
      position: absolute;
      bottom: 0;
      padding: 0 16px;
      padding-left: calc(16px + var(--horizontal-padding, 0px));
      padding-right: calc(16px + var(--horizontal-padding, 0px));
      padding-bottom: calc(6px + var(--bottom-padding, 0px));
      display: flex;
      justify-content: space-between;
      font-size: 12px;
    }

    .top-bar {
      position: relative;
    }

    .content {
      position: absolute;
      overflow: visible;
      top: calc(30px + var(--top-padding, 0px));
      top: calc(30px + constant(safe-area-inset-top) + var(--top-padding, 0px));
      top: calc(30px + env(safe-area-inset-top) + var(--top-padding, 0px));
      bottom: calc(24px + var(--bottom-padding, 0px));
      left: 0;
      right: 0;
    }

    .content-inner {
      margin: 0;
      margin-left: calc(16px + var(--horizontal-padding, 0px));
      margin-right: calc(16px + var(--horizontal-padding, 0px));
      overflow: hidden;
      text-align: justify;
      padding: 0;
      height: 100%;
    }

    .book-content {
      height: 100%;

      img {
        break-inside: avoid;
      }
    }

    &:not(.epub-iframe) {
      .book-content {
        -webkit-columns: calc(100vw - 16px - var(--horizontal-padding, 0px)) 1;
        -webkit-column-gap: calc(16px + var(--horizontal-padding, 0px));
        columns: calc(100vw - 16px - var(--horizontal-padding, 0px)) 1;
        column-gap: calc(16px + var(--horizontal-padding, 0px));
      }
    }
  }
}
.chapter-wrapper.mini-interface::-webkit-scrollbar {
  width: 0 !important;
}
`;
export const style2 = `body.mobile-scroll-read,
html.mobile-scroll-read
  -ms-overflow-style none
  scrollbar-width none

body.mobile-scroll-read::-webkit-scrollbar,
html.mobile-scroll-read::-webkit-scrollbar
  width 0 !important
  height 0 !important
  display none

.voice-list {
  .el-radio-button__inner {
    border-radius: 4px !important;
    border-left: 1px solid #DCDFE6;
    box-shadow: none;
  }
}
.night-theme {
  .voice-list {
    .el-radio-button {
      box-shadow: none !important;
    }
    .el-radio-button__inner {
      background-color: #bbb;
      border-color: #bbb;
    }
    .el-radio-button__inner:hover {
      color: #185798;
    }
    .el-radio-button__orig-radio:checked+.el-radio-button__inner {
      background-color: #185798;
      border-color: #185798;
      color: #fff;
      box-shadow: none;
    }
  }
}
.kindle-page {
  .day {
    .tool-icon {
      border: 1px solid #fefefefe;

      .icon-text {
        color: #444;
      }
    }

    .progress-tip {
      color: #444;
    }

    .cache-content-zone {
      color: #444;
    }

    .reader-bar-inner {

      .setting-title {
        color: rgba(0, 0, 0, 0.8);
      }

      .setting-value {
        color: #444;
      }
    }

    .bottom-bar, .top-bar {
      color: #444;
    }
  }
}
`;
