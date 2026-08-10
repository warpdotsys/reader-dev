import { mapGetters } from "vuex";
import Axios from "../plugins/axios";
const buildURL = require("axios/lib/helpers/buildURL");

export const BookCover = {
  name: "BookCover",
  template: `
  <el-dialog
    title="书籍封面"
    :visible.sync="show"
    :width="dialogSmallWidth"
    :fullscreen="$store.state.miniInterface"
    :class="
      isWebApp && !$store.getters.isNight ? 'status-bar-light-bg-dialog' : ''
    "
    :before-close="cancel"
  >
    <div class="custom-dialog-title" slot="title">
      <span class="el-dialog__title"
        >书籍封面
        <span
          class="float-right span-btn"
          @click="searchBookSourceByEventStream"
          ><i v-if="loadingMore" class="el-icon-loading"></i>
          {{ loadingMore ? "加载中..." : "加载更多" }}
        </span>
      </span>
    </div>
    <div v-if="show" class="book-cover-container">
      <div v-if="coverList.length" class="cover-list">
        <div
          v-for="(cover, index) in coverList"
          :key="'cover-' + index"
          class="book-cover-wrapper"
          @click="setCover(cover)"
        >
          <el-image class="book-cover" :src="cover" fit="cover" lazy>
            <img
              slot="error"
              class="el-image__inner"
              style="object-fit: cover"
              :src="noCover"
              alt=""
            />
          </el-image>
        </div>
      </div>
      <div v-else class="empty-notice"> 暂无其它来源 </div>
    </div>
  </el-dialog>
`,
  model: {
    prop: "show",
    event: "setShow"
  },
  data() {
    return {
      coverList: [] as string[],
      lastIndex: -1,
      loadingMore: false,
      noCover: require("../assets/imgs/noCover.jpeg")
    };
  },
  props: ["show", "book"],
  computed: {
    ...mapGetters(["dialogSmallWidth", "dialogTop"]),
    showBookInfo: {
      get() {
        return this.$store.state.showBookInfo;
      },
      set(val: any) {
        this.$store.commit("setShowBookInfo", val);
      }
    },
    coverListMap() {
      return this.coverList.reduce((map, item) => {
        map[item] = item;
        return map;
      }, {});
    }
  },
  watch: {
    show(isVisible) {
      if (isVisible) {
        this.coverList = [];
        this.lastIndex = -1;
        this.loadingMore = false;
        this.getBookSource();
      }
    }
  },
  methods: {
    cancel() {
      this.$emit("setShow", false);
    },
    getBookCoverUrl(book: any) {
      return book.customCoverUrl || book.coverUrl;
    },
    getBookSource(refresh?: any) {
      Axios.post(this.api + "/getAvailableBookSource", {
        url: this.book.bookUrl,
        refresh: refresh ? 1 : 0
      }).then(
        res => {
          this.loading = false;
          if (res.data.isSuccess) {
            this.coverList = (res.data.data || []).map(item => {
              return item.coverUrl;
            });
          }
        },
        error => {
          this.loading = false;
          this.$message.error(
            "获取书籍来源信息失败 " + (error && error.toString())
          );
          throw error;
        }
      );
    },
    searchBookSourceByEventStream() {
      const close = () => {
        try {
          if (
            this.searchEventSource &&
            this.searchEventSource.readyState != this.searchEventSource.CLOSED
          ) {
            this.searchEventSource.close();
          }
          this.searchEventSource = null;
        } catch (error) {
        /* ignore */
      }
      };
      if (this.loadingMore) {
        close();
        this.loadingMore = false;
        return;
      }
      const params = {
        accessToken: this.$store.state.token,
        concurrentCount: this.$store.state.searchConfig.concurrentCount,
        url: this.book.bookUrl,
        lastIndex: this.lastIndex
      };
      this.loadingMore = true;
      const url = buildURL(this.api + "/searchBookSourceSSE", params);
      close();
      this.searchEventSource = new EventSource(url, {
        withCredentials: true
      });
      this.searchEventSource.addEventListener("error", event => {
        this.loadingMore = false;
        close();
        try {
          if (event.data) {
            const data = JSON.parse(event.data);
            if (data && data.errorMsg) {
              this.$message.error(data.errorMsg);
            }
          }
        } catch (error) {
        /* ignore */
      }
      });
      const startLength = this.coverList.length;
      this.searchEventSource.addEventListener("end", event => {
        this.loadingMore = false;
        close();
        try {
          if (event.data) {
            const data = JSON.parse(event.data);
            if (data && data.lastIndex) {
              this.lastIndex = data.lastIndex;
            }
          }
          if (this.coverList.length === startLength) {
            this.$message.error("没有更多啦");
          }
        } catch (error) {
        /* ignore */
      }
      });
      this.searchEventSource.addEventListener("message", event => {
        try {
          if (event.data) {
            const data = JSON.parse(event.data);
            if (data && data.lastIndex) {
              this.lastIndex = data.lastIndex;
            }
            if (data && data.data) {
              this.coverList = [].concat(
                this.coverList,
                data.data
                  .filter(item => !this.coverListMap[item.coverUrl])
                  .map(item => item.coverUrl)
              );
            }
          }
        } catch (error) {
        /* ignore */
      }
      });
    },
    async setCover(coverUrl: any) {
      const res = await this.$confirm(
        "确认要将封面设置为所选择的图片吗？",
        "提示",
        {
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          type: "warning"
        }
      ).catch(() => {
        return false;
      });
      if (!res) {
        return false;
      }
      const book = { ...this.book, coverUrl: coverUrl };
      this.saveBook(book);
    },
    saveBook(book: any) {
      Axios.post(this.api + "/saveBook", book).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("操作成功");
            if (this.showBookInfo.bookUrl === book.bookUrl) {
              this.showBookInfo = res.data.data;
            }
            this.$store.commit("updateShelfBook", res.data.data);
          }
        },
        error => {
          this.$message.error("操作失败" + (error && error.toString()));
        }
      );
    }
  }
};

export const style = `
.book-cover-container {
  max-height: calc(var(--vh, 1vh) * 70 - 114px);
  overflow-y: auto;

  .book-cover-wrapper {
    display: inline-block;
    width: 25%;
    box-sizing: border-box;
    padding: 3px;
    text-align: center;
    vertical-align: top;
    margin-bottom: 10px;
    cursor: pointer;
    position: relative;
  }

  .empty-notice {
    min-height: 40px;
    text-align: center;
  }
}

@media screen and (max-width: 750px) {
  .book-cover-container {
    max-height: calc(var(--vh, 1vh) * 100 - 94px) !important;
  }
}
`;
