import { mapGetters } from "vuex";
import Axios from "../plugins/axios";
import eventBus from "../plugins/eventBus";

export const Bookmark = {
  model: {
    prop: "show",
    event: "setShow"
  },
  name: "Bookmark",
  template: `
  <el-dialog
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
      <span class="el-dialog__title">{{ book ? book.name : "" }} 书签管理 </span>
      <span class="title-center">
        <el-input
          v-model="search"
          size="mini"
          placeholder="输入关键字搜索"
          class="search-input"
        ></el-input>
      </span>
    </div>
    <div class="source-container table-container">
      <el-table
        :data="showList"
        :height="dialogContentHeight - 42"
        @selection-change="localSelection = $event"
        @sort-change="sortChange"
      >
        <el-table-column
          type="selection"
          width="25"
          :fixed="$store.state.miniInterface"
        >
        </el-table-column>
        <el-table-column
          min-width="150px"
          label="书籍"
          sortable="custom"
          :fixed="$store.state.miniInterface"
        >
          <template slot-scope="scope">
            {{ scope.row.bookName }} - {{ scope.row.bookAuthor }}
          </template>
        </el-table-column>
        <el-table-column
          property="chapterName"
          label="章节"
          min-width="150px"
          sortable="custom"
        >
        </el-table-column>
        <el-table-column property="bookText" label="内容" min-width="150px">
        </el-table-column>
        <el-table-column property="content" label="备注" min-width="150px">
        </el-table-column>
        <el-table-column label="操作" width="100px">
          <template slot-scope="scope">
            <el-button
              v-if="book"
              type="text"
              @click="showBookmark(scope.row)"
              >跳转</el-button
            >
            <el-button type="text" @click="editBookmark(scope.row)"
              >编辑</el-button
            >
          </template>
        </el-table-column>
      </el-table>
    </div>
    <div slot="footer" class="dialog-footer">
      <div>
        <el-button
          type="primary"
          size="medium"
          class="float-left"
          @click="deleteBookmarks"
          >批量删除
          <span v-if="localSelection.length"> ({{ localSelection.length }})</span></el-button
        >
        <el-button
          type="primary"
          size="medium"
          class="float-left"
          @click="uploadFile"
          >导入</el-button
        >
        <el-button
          type="primary"
          size="medium"
          class="float-left"
          @click="addBookmark"
          >添加</el-button
        >
        <input
          ref="fileRef"
          type="file"
          @change="onFileChange($event)"
          style="display:none"
        />
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
      localSelection: [],
      search: "",
      pagination: { page: 1, size: 25 },
      sortable: { prop: "", order: null }
    };
  },
  computed: {
    ...mapGetters(["dialogWidth", "dialogTop", "dialogContentHeight"]),
    bookmarkList() {
      if (this.book && this.book.name) {
        return this.$store.state.bookmarks.filter(
          v => v.bookName === this.book.name && v.bookAuthor === this.book.author
        );
      }
      return this.$store.state.bookmarks;
    },
    filterList() {
      return this.bookmarkList.filter(
        v =>
          !this.search ||
          v.bookName.toLowerCase().includes(this.search.toLowerCase()) ||
          v.bookAuthor.toLowerCase().includes(this.search.toLowerCase()) ||
          v.chapterName.toLowerCase().includes(this.search.toLowerCase()) ||
          v.bookText.toLowerCase().includes(this.search.toLowerCase()) ||
          v.content.toLowerCase().includes(this.search.toLowerCase())
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
      return offset > this.filterList.length
        ? []
        : this.filterList.slice(
            offset,
            Math.min(offset + this.pagination.size, this.filterList.length)
          );
    }
  },
  props: ["show", "book"],
  watch: {
    show(isVisible) {
      if (isVisible) {
        //
      }
    }
  },
  methods: {
    formatTableField(row, column, cellValue) {
      switch (column.property) {
        default:
          return cellValue;
      }
    },
    sortChange({ prop, order }) {
      this.sortable = { prop, order };
    },
    cancel() {
      this.$emit("setShow", false);
    },
    addBookmark() {
      eventBus.$emit("addBookmark");
    },
    async deleteBookmarks() {
      if (!this.localSelection.length) {
        this.$message.error("请选择需要删除的书签");
        return;
      }
      const res = await this.$confirm("确认要删除所选择的书签吗?", "提示", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning"
      }).catch(() => {
        return false;
      });
      if (!res) {
        return;
      }
      Axios.post(this.api + "/deleteBookmarks", this.localSelection).then(
        res => {
          if (res.data.isSuccess) {
            this.localSelection = [];
            this.$message.success("删除书签成功");
            this.$root.$children[0].loadBookmarks(true);
          }
        },
        error => {
          this.$message.error("删除书签失败 " + (error && error.toString()));
        }
      );
    },
    editBookmark(row) {
      eventBus.$emit("showBookmarkForm", { ...row }, false);
    },
    uploadFile() {
      this.$refs.fileRef.dispatchEvent(new MouseEvent("click"));
    },
    onFileChange(event) {
      const rawFile = event.target.files && event.target.files[0];
      const reader = new FileReader();
      reader.onload = e => {
        const data = e.target.result;
        try {
          const bookmarkList = JSON.parse(data);
          if (Array.isArray(bookmarkList) && bookmarkList.length) {
            this.comfirmImport(bookmarkList);
          }
        } catch (error) {
          this.$message.error("书签文件错误");
        }
      };
      reader.onerror = () => {
        // FileReader 读取出错，只能上传读取了
        let param = new FormData();
        param.append("file", rawFile);
        Axios.post(this.api + "/readSourceFile", param, {
          headers: { "Content-Type": "multipart/form-data" }
        }).then(
          res => {
            if (res.data.isSuccess) {
              let bookmarkList = [];
              res.data.data.forEach(v => {
                try {
                  const data = JSON.parse(v);
                  if (Array.isArray(data)) {
                    bookmarkList = bookmarkList.concat(data);
                  }
                } catch (error) {
                  //
                }
              });
              if (bookmarkList.length) {
                this.comfirmImport(bookmarkList);
              } else {
                this.$message.error("书签文件错误");
              }
            }
          },
          error => {
            this.$message.error(
              "读取书签文件内容失败 " + (error && error.toString())
            );
          }
        );
      };
      reader.readAsText(rawFile);
      this.$refs.fileRef.value = null;
    },
    async comfirmImport(bookmarkList) {
      const res = await this.$confirm(
        `确认要导入文件中的${bookmarkList.length}条书签吗?`,
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
      Axios.post(this.api + "/saveBookmarks", bookmarkList).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("导入书签成功");
            this.$root.$children[0].loadBookmarks(true);
          }
        },
        error => {
          this.$message.error("导入书签失败 " + (error && error.toString()));
        }
      );
    },
    showBookmark(bookmark) {
      eventBus.$emit("showBookmark", bookmark);
      this.cancel();
    }
  }
};

export const BookmarkStyle = `
.float-left {
  float: left;
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
