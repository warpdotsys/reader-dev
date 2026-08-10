import Long from "long";
import { mapGetters } from "vuex";
import Axios from "../plugins/axios";
import eventBus from "../plugins/eventBus";
const buildURL = require("axios/lib/helpers/buildURL");
import { LimitResquest } from "../plugins/helper";

export const BookManage: any = {
  model: {
    prop: "show",
    event: "setShow"
  },
  name: "BookManage",
  template: `
  <el-dialog
    title="书架管理"
    :visible.sync="show"
    :width="dialogWidth"
    :top="dialogTop"
    :fullscreen="$store.state.miniInterface"
    :class="
      isWebApp && !$store.getters.isNight ? 'status-bar-light-bg-dialog' : ''
    "
    v-if="$store.getters.isNormalPage"
    :before-close="cancel"
  >
    <div class="custom-dialog-title flex-title" slot="title">
      <span class="el-dialog__title">书架管理 </span>
      <span class="title-center">
        <el-input
          v-model="search"
          size="mini"
          placeholder="输入关键字搜索"
          class="search-input"
        ></el-input>
      </span>
      <span class="action-zone" v-show="!$store.state.miniInterface">
        <span class="float-right small-tip">❗️只能缓存文本内容</span>
      </span>
    </div>
    <div class="source-container table-container">
      <el-table
        :data="showList"
        :height="dialogContentHeight - 42"
        @selection-change="manageBookSelection = $event"
        @sort-change="sortChange"
      >
        <el-table-column
          type="selection"
          width="25"
          :selectable="isBookSelectable"
          :fixed="$store.state.miniInterface"
        >
        </el-table-column>
        <el-table-column
          property="name"
          label="书名名"
          min-width="100"
          sortable="custom"
          :fixed="$store.state.miniInterface"
        >
          <template slot-scope="scope">
            <el-button
              class="text-button"
              size="medium"
              type="text"
              @click="showBookInfo(scope.row)"
              >{{ scope.row.name }}</el-button
            >
          </template>
        </el-table-column>
        <el-table-column
          property="author"
          label="作者"
          sortable="custom"
          min-width="100"
        >
        </el-table-column>
        <el-table-column
          property="group"
          label="分组"
          min-width="120"
          :filters="bookGroupFilters"
          :filter-method="filterHandler"
        >
          <template slot-scope="scope">
            {{ renderBookGroup(scope.row) }}
          </template>
        </el-table-column>
        <el-table-column label="章节" min-width="120">
          <template slot-scope="scope">
            <span>共 {{ scope.row.totalChapterNum }} 章</span><br />
            <span v-if="scope.row.origin !== 'loc_book'">
              服务器缓存： {{ scope.row.cachedChapterCount || 0 }} 章 <br />
            </span>
            <span>浏览器缓存： {{ scope.row.localCacheCount }} 章</span>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="100px">
          <template slot-scope="scope">
            <el-button
              class="text-button"
              size="medium"
              type="text"
              @click="editBook(scope.row)"
              >编辑</el-button
            >
            <el-button
              class="text-button"
              size="medium"
              type="text"
              style="margin-left: 0"
              @click="setBookGroup(scope.row)"
              >分组</el-button
            >
            <el-dropdown
              :trigger="$store.state.touchable ? 'click' : 'hover'"
              @command="cacheBook(scope.row, $event)"
            >
              <el-button class="text-button" type="text" size="medium">
                <span v-if="isCaching(scope.row)">
                  <i class="el-icon-loading"></i> 缓存中
                </span>
                <span v-else>
                  缓存<i class="el-icon-arrow-down el-icon--right"></i>
                </span>
              </el-button>
              <el-dropdown-menu slot="dropdown">
                <el-dropdown-item
                  v-if="scope.row.origin !== 'loc_book'"
                  command="cacheBookOnServer"
                  >服务器缓存</el-dropdown-item
                >
                <el-dropdown-item
                  v-if="scope.row.origin !== 'loc_book'"
                  command="cacheBookSSE"
                  >缓存到服务器</el-dropdown-item
                >
                <el-dropdown-item command="cacheBookLocal"
                  >缓存到浏览器</el-dropdown-item
                >
                <el-dropdown-item
                  v-if="scope.row.origin !== 'loc_book'"
                  command="deleteBookCache"
                  >删除服务器缓存</el-dropdown-item
                >
                <el-dropdown-item command="deleteBookLocalCache"
                  >删除浏览器缓存</el-dropdown-item
                >
              </el-dropdown-menu>
            </el-dropdown>
            <el-dropdown
              :trigger="$store.state.touchable ? 'click' : 'hover'"
              @command="exportBook(scope.row, $event)"
            >
              <el-button class="text-button" type="text" size="medium">
                导出<i class="el-icon-arrow-down el-icon--right"></i>
              </el-button>
              <el-dropdown-menu slot="dropdown">
                <el-dropdown-item command="txt">导出为TXT</el-dropdown-item>
                <el-dropdown-item command="epub">导出为Epub</el-dropdown-item>
              </el-dropdown-menu>
            </el-dropdown>
          </template>
        </el-table-column>
      </el-table>
    </div>
    <div slot="footer" class="dialog-footer">
      <div>
        <el-dropdown
          class="float-left"
          :trigger="$store.state.touchable ? 'click' : 'hover'"
          @command="multiOperate"
        >
          <el-button type="primary" size="medium">
            操作
            <span v-if="manageBookSelection.length">
              ({{ manageBookSelection.length }})
            </span>
            <i class="el-icon-arrow-down el-icon--right"></i>
          </el-button>
          <el-dropdown-menu slot="dropdown">
            <el-dropdown-item command="deleteBookList"
              >批量删除</el-dropdown-item
            >
            <el-dropdown-item command="addBookGroupMulti"
              >添加分组</el-dropdown-item
            >
            <el-dropdown-item command="removeBookGroupMulti"
              >删除分组</el-dropdown-item
            >
          </el-dropdown-menu>
        </el-dropdown>
        <el-dropdown
          class="float-left"
          :trigger="$store.state.touchable ? 'click' : 'hover'"
          @command="cacheBookMulti"
        >
          <el-button type="primary" size="medium">
            缓存
            <span v-if="manageBookSelection.length">
              ({{ manageBookSelection.length }})
            </span>
            <i class="el-icon-arrow-down el-icon--right"></i>
          </el-button>
          <el-dropdown-menu slot="dropdown">
            <el-dropdown-item command="cacheBookOnServer"
              >服务器缓存</el-dropdown-item
            >
            <el-dropdown-item command="cacheBookSSE"
              >缓存到服务器</el-dropdown-item
            >
            <el-dropdown-item command="cacheBookLocal"
              >缓存到浏览器</el-dropdown-item
            >
            <el-dropdown-item command="deleteBookCache"
              >删除服务器缓存</el-dropdown-item
            >
            <el-dropdown-item command="deleteBookLocalCache"
              >删除浏览器缓存</el-dropdown-item
            >
          </el-dropdown-menu>
        </el-dropdown>
        <el-button v-if="multiOperating" size="medium" @click="cancelMulti"
          >取消批量操作</el-button
        >
      </div>
      <div class="source-pagination">
        <el-pagination
          :current-page.sync="pagination.page"
          :page-sizes="[25, 50, 100, 200, 300, 400, filterList.length]"
          :page-size.sync="pagination.size"
          layout="total, sizes, prev, pager, next"
          :total="filterList.length"
          :pager-count="$store.state.miniInterface ? 5 : 7"
        ></el-pagination>
      </div>
    </div>
  </el-dialog>
`,
  data() {
    return {
      bookList: [],
      manageBookSelection: [],
      search: "",
      pagination: { page: 1, size: 25 },
      sortable: { prop: "", order: null },
      multiOperating: false
    };
  },
  props: ["show"],
  computed: {
    ...mapGetters(["dialogWidth", "dialogTop", "dialogContentHeight"]),
    bookGroupList() {
      return this.$store.state.bookGroupList.filter(v => v.groupId > 0);
    },
    bookGroupFilters() {
      return [{ text: "未分组", value: 0 }].concat(
        this.bookGroupList.map(v => ({ text: v.groupName, value: v.groupId }))
      );
    },
    cachingBookList: {
      get() {
        return this.$store.state.cachingBookList;
      },
      set(val) {
        this.$store.commit("setCachingBookList", val);
      }
    },
    cachingBookMap() {
      const map = {};
      this.cachingBookList.map(v => {
        map[v.bookUrl] = true;
      });
      return map;
    },
    filterList() {
      return this.bookList.filter(
        v => !this.search || v.name.toLowerCase().includes(this.search.toLowerCase())
      );
    },
    sortList() {
      if (!this.sortable.prop || !this.sortable.order) {
        return this.filterList;
      }
      const list = [].concat(this.filterList);
      return list.sort((a, b) => {
        if (this.sortable.order !== "ascending") {
          const t = a;
          a = b;
          b = t;
        }
        return a[this.sortable.prop] > b[this.sortable.prop]
          ? 1
          : a[this.sortable.prop] < b[this.sortable.prop]
          ? -1
          : 0;
      });
    },
    showList() {
      const offset = (this.pagination.page - 1) * this.pagination.size;
      return offset > this.sortList.length
        ? []
        : this.sortList.slice(
            offset,
            Math.min(offset + this.pagination.size, this.sortList.length)
          );
    }
  },
  created() {
    window.cacheEventSource = window.cacheEventSource || {};
    window.cacheRequestHandle = window.cacheRequestHandle || {};
    window.bookManageComp = this;
  },
  watch: {
    show(isVisible) {
      if (isVisible) {
        this.loadBookCacheInfo();
      } else {
        this.manageBookSelection = [];
      }
    }
  },
  methods: {
    cancel() {
      this.$emit("setShow", false);
    },
    formatTableField(row, column, cellValue) {
      switch (column.property) {
        default:
          return cellValue;
      }
    },
    isBookSelectable() {
      return true;
    },
    sortChange({ prop, order }) {
      this.sortable = { prop, order };
    },
    async loadBookCacheInfo() {
      return Axios.get(this.api + "/getShelfBookWithCacheInfo").then(
        res => {
          if (res.data.isSuccess) {
            this.computeCachedCata(res.data.data).then(v => {
              this.bookList = v;
            });
          }
        },
        error => {
          this.$message.error(
            "获取书架信息失败 " + (error && error.toString())
          );
        }
      );
    },
    async deleteBookList() {
      if (!this.manageBookSelection.length) {
        this.$message.error("请选择需要删除的书籍");
        return;
      }
      const res = await this.$confirm("确认要删除所选择的书籍吗?", "提示", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning"
      }).catch(() => {
        return false;
      });
      if (!res) {
        return;
      }
      Axios.post(this.api + "/deleteBooks", this.manageBookSelection).then(
        res => {
          if (res.data.isSuccess) {
            this.manageBookSelection = [];
            this.$message.success("删除书籍成功");
            this.loadBookCacheInfo();
            this.$root.$children[0].loadBookShelf();
          }
        },
        error => {
          this.$message.error("删除书籍失败 " + (error && error.toString()));
        }
      );
    },
    async addBookGroupMulti() {
      return this.operateBookGroupMulti(true);
    },
    async removeBookGroupMulti() {
      return this.operateBookGroupMulti();
    },
    async operateBookGroupMulti(isAdd) {
      const operate = isAdd ? "添加" : "移除";
      if (!this.manageBookSelection.length) {
        this.$message.error("请选择需要" + operate + "分组的书籍");
        return;
      }
      const formData = { groupId: "" };
      const formItems = [
        {
          name: "groupId",
          label: operate + "分组",
          type: "select",
          placeholder: "请选择分组",
          multiple: false,
          options: this.bookGroupList.map(v => ({
            label: v.groupName,
            value: v.groupId
          }))
        }
      ];
      const res = await this.$msgbox({
        title: isAdd ? "批量添加分组" : "批量删除分组",
        message: this.renderForm(
          isAdd ? "addBookGroupMulti" : "removeBookGroupMulti",
          formData,
          formItems,
          value => {
            formData.groupId = 0 | value.groupId;
          }
        ),
        showCancelButton: true,
        confirmButtonText: "确定",
        cancelButtonText: "取消"
      }).catch(e => {
        return e === "close" && "close";
      });
      if (res !== "confirm") {
        return;
      }
      const group = this.bookGroupList.find(v => v.groupId === formData.groupId);
      if (!group) {
        this.$message.error("请选择分组");
        return;
      }
      const confirmed = await this.$confirm(
        isAdd
          ? `确认要将所选择的书籍添加到${group.groupName}分组吗?`
          : `确认要将所选择的书籍从${group.groupName}分组中移除吗?`,
        "提示",
        {
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          type: "warning"
        }
      ).catch(() => {
        return false;
      });
      if (!confirmed) {
        return;
      }
      Axios.post(
        this.api + (isAdd ? "/addBookGroupMulti" : "/removeBookGroupMulti"),
        {
          groupId: group.groupId,
          bookList: this.manageBookSelection
        }
      ).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("操作成功");
            this.loadBookCacheInfo();
            this.$root.$children[0].loadBookShelf();
          }
        },
        error => {
          this.$message.error("操作失败 " + (error && error.toString()));
        }
      );
    },
    renderBookGroup(book) {
      const groups = [];
      this.$store.state.bookGroupList.forEach(v => {
        if (
          v.groupId > 0 &&
          Long.fromNumber(v.groupId)
            .and(Long.fromNumber(book.group))
            .greaterThan(0)
        ) {
          groups.push(v.groupName);
        }
      });
      if (!groups.length) {
        groups.push("未分组");
      }
      return groups.join(" ");
    },
    filterHandler(value, row, column) {
      const property = column["property"];
      return row[property] === value;
    },
    showBookInfo(book) {
      eventBus.$emit("showBookInfoDialog", book);
    },
    editBook(book) {
      eventBus.$emit("editBook", book, false, () => {
        this.loadBookCacheInfo();
      });
    },
    setBookGroup(book) {
      this.$store.commit("setShowBookInfo", book);
      eventBus.$emit("showBookGroupDialog", true);
    },
    isCaching(book) {
      return (
        !!this.cachingBookMap[book.bookUrl] ||
        !!(window.cacheEventSource && window.cacheEventSource[book.bookUrl]) ||
        !!(window.cacheRequestHandle && window.cacheRequestHandle[book.bookUrl])
      );
    },
    cacheBook(book, command) {
      this[command](book);
    },
    cancelMulti() {
      if (this.multiOperating) {
        if (
          this.multiCachingHandler.cancel(),
          this.multiOperating === "cacheBookSSE" ||
            this.multiOperating === "cacheBookLocal"
        ) {
          for (let i = 0; i < this.lastSelection.length; i++) {
            if (this.isCaching(this.lastSelection[i])) {
              this[this.multiOperating](this.lastSelection[i]);
            }
          }
        }
        this.multiOperating = false;
        this.lastSelection = [];
        this.$message.info("批量操作已取消");
      }
    },
    multiOperate(command) {
      if (this.manageBookSelection.length) {
        this[command]();
      } else {
        this.$message.error("请选择需要操作的书籍");
      }
    },
    async cacheBookMulti(command) {
      if (this.multiOperating) {
        this.$message.error("正在批量操作，请取消后再试");
        return;
      }
      if (!this.manageBookSelection.length) {
        this.$message.error("请选择需要操作的书籍");
        return;
      }
      if (command === "deleteBookCache" || command === "deleteBookLocalCache") {
        const res = await this.$confirm(
          `确认要批量删除${
            command === "deleteBookCache" ? "服务器上" : "浏览器中"
          }所选书籍的缓存章节吗?`,
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
          return;
        }
      }
      if (command === "cacheBookOnServer") {
        const res = await this.$confirm(
          "确认要在服务器上批量缓存所选书籍的章节内容吗?",
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
          return;
        }
        this.cacheBookOnServer(this.manageBookSelection);
        return;
      }
      this.multiOperating = command;
      this.lastSelection = [].concat(this.manageBookSelection);
      this.multiCachingHandler = LimitResquest(2, handler => {
        if (handler.isEnd()) {
          this.multiOperating = false;
          this.lastSelection = [];
          this.$message.success("批量操作完成");
        }
      });
      for (let i = 0; i < this.manageBookSelection.length; i++) {
        this.multiCachingHandler(() => {
          return this[command](this.manageBookSelection[i]);
        });
      }
      this.$message.info("批量操作未完成/取消之前请勿关闭当前页面");
    },
    cacheBookSSE(book) {
      const self = this;
      return new Promise((resolve, reject) => {
        const tryClose = function() {
          try {
            if (
              window.cacheEventSource[book.bookUrl] &&
              window.cacheEventSource[book.bookUrl].readyState !==
                window.cacheEventSource[book.bookUrl].CLOSED
            ) {
              window.cacheEventSource[book.bookUrl].close();
            }
            window.cacheEventSource[book.bookUrl] = null;
            delete window.cacheEventSource[book.bookUrl];
            const index = self.cachingBookList.findIndex(
              v => v.bookUrl === book.bookUrl
            );
            self.cachingBookList.splice(index, 1);
            self.cachingBookList = [].concat(self.cachingBookList);
          } catch (error) {
            //
          }
        };
        if (self.isCaching(book)) {
          // 取消缓存
          self.$message.info("已取消缓存");
          reject("已取消缓存");
          if (window.cacheEventSource[book.bookUrl]) {
            tryClose();
          }
          return;
        }
        const params = {
          url: book.bookUrl,
          refresh: 0
        };
        const url = buildURL(self.api + "/cacheBookSSE", params);
        tryClose();
        self.cachingBookList = self.cachingBookList.concat([book]);
        window.cacheEventSource[book.bookUrl] = new EventSource(url, {
          withCredentials: true
        });
        window.cacheEventSource[book.bookUrl].addEventListener("error", e => {
          tryClose();
          try {
            if (e.data) {
              const result = JSON.parse(e.data);
              if (result && result.errorMsg) {
                self.$message.error(result.errorMsg);
                reject(result.errorMsg);
              }
            }
          } catch (error) {
            //
          }
        });
        window.cacheEventSource[book.bookUrl].addEventListener("end", e => {
          self.$message.info(book.name + "缓存到服务器完成");
          resolve(book.name + "缓存到服务器完成");
          tryClose();
          try {
            if (e.data) {
              // const result = JSON.parse(e.data);
              // console.log(result);
            }
          } catch (error) {
            //
          }
        });
        window.cacheEventSource[book.bookUrl].addEventListener("message", e => {
          try {
            if (e.data) {
              const result = JSON.parse(e.data);
              if (result && result.cachedCount) {
                const index = self.bookList.findIndex(
                  v => v.bookUrl === book.bookUrl
                );
                self.$set(self.bookList, index, {
                  ...book,
                  cachedChapterCount: result.cachedCount
                });
              }
            }
          } catch (error) {
            //
          }
        });
      });
    },
    cacheBookLocal(book) {
      const self = this;
      return new Promise((resolve, reject) => {
        if (self.isCaching(book)) {
          // 取消缓存
          self.$message.info("已取消缓存");
          reject("已取消缓存");
          if (window.cacheRequestHandle[book.bookUrl]) {
            window.cacheRequestHandle[book.bookUrl].cancel();
            window.cacheRequestHandle[book.bookUrl] = null;
            delete window.cacheRequestHandle[book.bookUrl];
          }
          return;
        }
        let isComputing = false;
        const computeCache = function() {
          self.computeCachedCata([book]).then(bookList => {
            isComputing = false;
            const index = self.bookList.findIndex(
              v => v.bookUrl === book.bookUrl
            );
            self.$set(self.bookList, index, {
              ...book,
              localCacheCount: bookList[0].localCacheCount
            });
          });
        };
        window.cacheRequestHandle[book.bookUrl] = LimitResquest(
          2,
          handler => {
            if (!isComputing) {
              isComputing = true;
              computeCache();
            }
            if (handler.isEnd()) {
              self.$message.success("缓存到浏览器完成");
              resolve("缓存到浏览器完成");
              computeCache();
            }
          }
        );
        for (let i = 0; i < book.totalChapterNum; i++) {
          (function(chapterIndex) {
            window.cacheRequestHandle[book.bookUrl](() => {
              return self.$root.$children[0].getBookContent(
                chapterIndex,
                {
                  timeout: 1000 * self.$store.getters.config.chapterRequestTimeout,
                  silent: true
                },
                false,
                true,
                book
              );
            });
          })(i);
        }
      });
    },
    cacheBookOnServer(book) {
      const books = Array.isArray(book) ? book : [book];
      Axios.post(this.api + "/cacheBookOnServer", {
        bookUrlList: books.map(v => v.bookUrl)
      }).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("提交缓存任务成功");
          }
        },
        error => {
          this.$message.error(
            "提交缓存任务失败 " + (error && error.toString())
          );
        }
      );
    },
    exportBook(book, type) {
      const url = buildURL(this.api + "/exportBook", {
        url: book.bookUrl,
        isEpub: type === "epub" ? 1 : 0,
        accessToken: this.$store.state.token
      });
      window.open(url, "__blank");
    },
    computeCachedCata(bookList, returnCacheMap) {
      const cachePrefixMap = {};
      bookList.forEach(book => {
        cachePrefixMap[book.bookUrl] = {
          key:
            "localCache@" +
            book.name +
            "_" +
            book.author +
            "@" +
            book.bookUrl +
            "@chapterContent-",
          map: {}
        };
      });
      return window.$cacheStorage
        .iterate(function(value, key) {
          for (const bookUrl in cachePrefixMap) {
            if (key.startsWith(cachePrefixMap[bookUrl].key)) {
              try {
                const index = parseInt(
                  key.replace(cachePrefixMap[bookUrl].key, "")
                );
                cachePrefixMap[bookUrl].map[index] = true;
              } catch (error) {
                //
              }
              break;
            }
          }
        })
        .then(() => {
          if (returnCacheMap) return cachePrefixMap;
          return bookList.map(v => {
            const cacheMap = cachePrefixMap[v.bookUrl].map;
            v.localCacheCount = Object.keys(cacheMap).length;
            return v;
          });
        })
        .catch(function() {
          if (returnCacheMap) return cachePrefixMap;
          return bookList.map(v => {
            v.localCacheCount = 0;
            return v;
          });
        });
    },
    deleteBookCache(book, confirmFlag) {
      const self = this;
      return new Promise(async (resolve, reject) => {
        if (confirmFlag) {
          const res = await self.$confirm(
            `确认要删除服务器上《${book.name}》的缓存章节吗?`,
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
            reject("canceled");
            return;
          }
        }
        Axios.post(self.api + "/deleteBookCache", {
          bookUrl: book.bookUrl
        }).then(
          res => {
            if (res.data.isSuccess) {
              self.$message.success("删除服务器缓存成功");
              self.loadBookCacheInfo();
              resolve("");
            }
          },
          error => {
            self.$message.error(
              "删除服务器缓存失败 " + (error && error.toString())
            );
            reject(error);
          }
        );
      });
    },
    deleteBookLocalCache(book, confirmFlag) {
      const self = this;
      return new Promise(async (resolve, reject) => {
        if (confirmFlag) {
          const res = await self.$confirm(
            `确认要删除浏览器中《${book.name}》的缓存章节吗?`,
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
            reject("canceled");
            return;
          }
        }
        self
          .computeCachedCata([book], true)
          .then(cacheMap => {
            const ops = [];
            for (const index in cacheMap[book.bookUrl].map) {
              const cacheKey = cacheMap[book.bookUrl].key + index;
              ops.push(window.$cacheStorage.removeItem(cacheKey));
            }
            return Promise.all(ops);
          })
          .then(() => {
            self.$message.success("删除浏览器缓存成功");
            resolve("删除浏览器缓存成功");
            self.computeCachedCata([].concat(self.bookList)).then(v => {
              self.bookList = v;
            });
          });
      });
    }
  }
};

export const stylusStyleScoped = `
.float-right {
  float: right;
}
.small-tip {
  font-size: 14px;
  margin-right: 10px;
}
.dialog-footer {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  justify-content: space-between;
  .float-left {
    margin-right: 5px;
    margin-bottom: 5px;
  }
}
.flex-title {
  display: flex;
  justify-content: space-between;
  .title-center {
    flex: 1;
    text-align: center;
  }
  .search-input {
    width: 200px;
    margin-top: -2px;
  }
}
.source-container {
  .text-button {
    padding: 3px 5px;
  }
  .source-pagination {
    margin-top: 5px;
    text-align: right;
  }
}
`;
