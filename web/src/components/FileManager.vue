<template>
  <el-dialog
    :title="title || '文件管理'"
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
    <div class="custom-dialog-title" slot="title">
      <span class="el-dialog__title"
        >{{ title || '文件管理' }}
        <span class="float-right span-btn" @click="upload">上传</span>
        <span class="float-right span-btn" @click="mkdir">新建目录</span>
        <span class="float-right span-btn" @click="resolveBooks" v-if="!resolveMode">解析书籍</span>
        <span class="float-right span-btn" @click="resolveAndImportBooks" v-if="!resolveMode">一键导入</span>
        <input
          ref="bookRef"
          type="file"
          multiple="multiple"
          @change="onFileChange($event)"
          style="display:none"
        />
      </span>
    </div>
    <div class="source-container table-container">
      <el-table
        ref="fileTable"
        :data="filteredFileList"
        :height="dialogContentHeight"
        @selection-change="fileSelection = $event"
      >
        <el-table-column
          type="selection"
          width="25"
          :fixed="$store.state.miniInterface"
          :selectable="row => !row.toParent"
        >
        </el-table-column>
        <el-table-column
          property="name"
          min-width="150px"
          label="文件名"
          :fixed="$store.state.miniInterface"
        >
          <template slot-scope="scope">
            <span v-if="!scope.row.isDirectory && !resolveMode">{{ scope.row.name }}</span>
            <span v-if="resolveMode">{{ scope.row.name }}</span>
            <el-link
              type="primary"
              v-if="scope.row.isDirectory && !resolveMode"
              @click="showFileList(scope.row.path)"
              >{{ scope.row.name }}</el-link
            >
          </template>
        </el-table-column>
        <el-table-column
          property="size"
          label="大小"
          :formatter="formatTableField"
          min-width="100px"
          v-if="!resolveMode"
        ></el-table-column>
        <el-table-column
          property="lastModified"
          label="修改时间"
          :formatter="formatTableField"
          width="120px"
          v-if="!resolveMode"
        ></el-table-column>
        <el-table-column
          property="book.author"
          label="作者"
          min-width="100px"
          v-if="resolveMode"
          :filters="authorList"
          :filter-method="filterAuthor"
        ></el-table-column>
        <el-table-column
          label="状态"
          width="80px"
          v-if="resolveMode"
          column-key="importStatus"
          :filters="[
            { text: '已导入', value: 'imported' },
            { text: '未导入', value: 'unImport' },
            { text: '可导入', value: 'canImport' },
            { text: '目录', value: 'directory' }
          ]"
          :filter-method="filterHandler"
        >
          <template slot-scope="scope">
            <span v-if="scope.row.isImported" style="color:#67c23a">已导入</span>
            <span v-else-if="scope.row.canImport">可导入</span>
            <span v-else-if="scope.row.isDirectory">目录</span>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="140px" v-if="!resolveMode">
          <template slot="header">
            <el-input
              v-model="keyword"
              size="mini"
              placeholder="输入关键字搜索"
            />
          </template>
          <template slot-scope="scope">
            <el-button
              type="text"
              @click="operateFile(scope.row, 'deleteFile')"
              style="color: #f56c6c"
              v-if="!scope.row.toParent"
              >删除</el-button
            >
            <el-button
              type="text"
              @click="operateFile(scope.row, 'editFile')"
              v-if="!scope.row.toParent && !scope.row.isDirectory"
              >编辑</el-button
            >
            <el-button
              type="text"
              @click="operateFile(scope.row, 'downloadFile')"
              v-if="!scope.row.toParent && !scope.row.isDirectory"
              >下载</el-button
            >
            <el-button
              type="text"
              @click="operateFile(scope.row, 'importFromFile')"
              v-if="canImport(scope.row)"
              >导入</el-button
            >
            <el-button
              type="text"
              @click="operateFile(scope.row, 'restoreFromFile')"
              v-if="scope.row.name && scope.row.name.endsWith('.zip')"
              >恢复</el-button
            >
          </template>
        </el-table-column>
        <el-table-column label="操作" width="80px" v-if="resolveMode">
          <template slot-scope="scope">
            <el-button
              type="text"
              @click="operateFile(scope.row, 'importFromFile')"
              v-if="scope.row.canImport && !scope.row.isImported"
              >导入</el-button
            >
          </template>
        </el-table-column>
      </el-table>
    </div>
    <div slot="footer" class="dialog-footer">
      <el-button
        type="primary"
        size="medium"
        class="float-left"
        @click="deleteFileList"
        v-if="!resolveMode"
        >批量删除</el-button
      >
      <el-button
        type="primary"
        size="medium"
        class="float-left"
        @click="importFromFile(true)"
        >批量导入</el-button
      >
      <span class="check-tip">已选择 {{ fileSelection.length }} 个</span>
      <el-button size="medium" @click="cancel">取消</el-button>
    </div>
  </el-dialog>
</template>

<script>
import { mapGetters } from "vuex";
import Axios from "../plugins/axios";
import eventBus from "../plugins/eventBus";
import { formatSize } from "../plugins/helper";
const buildURL = require("axios/lib/helpers/buildURL");

export default {
  model: {
    prop: "show",
    event: "setShow"
  },
  name: "FileManager",
  data() {
    return {
      currentPath: "/",
      fileList: [],
      keyword: "",
      searchWord: "",
      resolveMode: false,
      fileSelection: []
    };
  },
  props: ["show", "home", "title"],
  computed: {
    ...mapGetters(["dialogWidth", "dialogTop", "dialogContentHeight"]),
    localBook() {
      return this.$store.getters.shelfBooks.filter(
        e => e.origin === "loc_book"
      );
    },
    filteredFileList() {
      return this.fileList.filter(t => {
        return !this.keyword || t.name.toLowerCase().includes(this.keyword.toLowerCase());
      });
    },
    authorList() {
      if (!this.resolveMode) return [];
      let seen = {};
      return this.filteredFileList
        .map(e => e.book.author)
        .filter(t => {
          if (!t || seen[t]) return false;
          seen[t] = 1;
          return true;
        })
        .map(e => ({ text: e, value: e }));
    }
  },
  watch: {
    show(isVisible) {
      if (isVisible) {
        this.resolveMode = false;
        this.showFileList("/");
        this.$nextTick(() => {
          if (this.$refs.fileTable) {
            this.$refs.fileTable.clearFilter();
            this.$refs.fileTable.clearSelection();
            this.$refs.fileTable.clearSort();
          }
        });
      }
    }
  },
  methods: {
    formatTableField(row, column, cellValue) {
      switch (column.property) {
        case "createdAt":
        case "lastLoginAt":
        case "lastModified":
          return cellValue
            ? new Date(cellValue).format("yy-MM-dd hh:mm")
            : "";
        case "size":
          return row.isDirectory ? "" : formatSize(cellValue);
        default:
          return cellValue;
      }
    },
    canImport(row) {
      const path = row.path.toLowerCase();
      return (
        path.endsWith(".txt") ||
        path.endsWith(".epub") ||
        path.endsWith(".umd") ||
        path.endsWith(".cbz") ||
        path.endsWith(".pdf")
      );
    },
    cancel() {
      this.$emit("setShow", false);
    },
    sortMethod(a, b) {
      if (a.name === "..") return -1;
      if (b.name === "..") return 1;
      if (a.isDirectory && !b.isDirectory) return -1;
      if (!a.isDirectory && b.isDirectory) return 1;
      if (a.name > b.name) return 1;
      return -1;
    },
    filterHandler(value, row) {
      switch (value) {
        case "imported":
          return row.isImported;
        case "unImport":
          return !row.isDirectory && !row.isImported;
        case "canImport":
          return !row.isDirectory && row.canImport;
        case "directory":
          return row.isDirectory;
        default:
          return true;
      }
    },
    filterAuthor(value, row) {
      return row.book.author === value;
    },
    operate(action) {
      if (action === "importFromFile") return this.importFromFile(true);
      this[action].call(this);
    },
    operateFile(row, action) {
      this[action].call(this, row);
    },
    resolveBooks() {
      this.$confirm(
        "解析当前目录及子目录的所有书籍耗费资源较多，确认要继续?",
        "提示",
        {
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          type: "warning",
          beforeClose: (action, instance, done) => {
            if (action === "confirm") {
              instance.confirmButtonLoading = true;
              instance.confirmButtonText = "解析中...";
              Axios.post(this.api + "/file/parse", {
                path: this.currentPath,
                home: this.home
              }).then(
                res => {
                  if (res.data.isSuccess) {
                    this.fileList = res.data.data.map(t => ({
                      ...t,
                      isImported: !!this.localBook.find(
                        e => e.originName.indexOf(t.path) >= 0
                      ),
                      canImport: this.canImport(t)
                    }));
                    this.resolveMode = true;
                    this.$nextTick(() => {
                      if (this.$refs.fileTable) {
                        this.$refs.fileTable.doLayout();
                      }
                    });
                  }
                },
                error => {
                  this.$message.error(
                    "解析书籍失败 " + (error && error.toString())
                  );
                }
              ).then(() => {
                done();
                instance.confirmButtonLoading = false;
              });
            } else {
              done();
            }
          }
        }
      ).catch(() => false);
    },
    resolveAndImportBooks() {
      this.$confirm(
        "一键导入当前目录及子目录的所有书籍耗费资源较多，确认要继续?",
        "提示",
        {
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          type: "warning",
          beforeClose: (action, instance, done) => {
            if (action === "confirm") {
              instance.confirmButtonLoading = true;
              instance.confirmButtonText = "导入中...";
              Axios.post(
                this.api + "/file/parse",
                { path: this.currentPath, home: this.home, import: 1 },
                { timeout: 300000 }
              ).then(
                res => {
                  if (res.data.isSuccess) {
                    this.$root.$children[0].loadBookShelf();
                    this.$alert(
                      "成功导入" + res.data.data.length + "本书",
                      "导入结果"
                    );
                  }
                },
                error => {
                  this.$message.error(
                    "一键导入书籍失败 " + (error && error.toString())
                  );
                }
              ).then(() => {
                done();
                instance.confirmButtonLoading = false;
              });
            } else {
              done();
            }
          }
        }
      ).catch(() => false);
    },
    showFileList(path) {
      this.currentPath = path || "/";
      Axios.get(this.api + "/file/list", {
        params: {
          path: this.currentPath,
          home: this.home
        }
      }).then(
        res => {
          if (res.data.isSuccess) {
            res.data.data = res.data.data || [];
            if (this.currentPath !== "/") {
              const paths = this.currentPath
                .split("/")
                .filter(v => v);
              paths.pop();
              res.data.data.unshift({
                name: "..",
                isDirectory: true,
                toParent: true,
                path: "/" + paths.join("/")
              });
            }
            res.data.data.sort(this.sortMethod);
            this.fileList = res.data.data.map(e => ({
              ...e,
              isImported:
                !e.isDirectory &&
                !!this.localBook.find(
                  b => b.originName.indexOf(e.path) >= 0
                ),
              canImport: this.canImport(e)
            }));
          }
        },
        error => {
          this.$message.error(
            "加载文件列表失败 " + (error && error.toString())
          );
        }
      );
    },
    async deleteFileList() {
      if (!this.fileSelection.length) {
        this.$message.error("请选择需要删除的文件");
        return;
      }
      const res = await this.$confirm("确认要删除所选择的文件吗?", "提示", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning"
      }).catch(() => false);
      if (!res) return;
      Axios.post(this.api + "/file/deleteMulti", {
        path: this.fileSelection.map(e => e.path),
        home: this.home
      }).then(
        res => {
          if (res.data.isSuccess) {
            this.fileSelection = [];
            this.$message.success("删除文件成功");
            this.showFileList(this.currentPath);
          }
        },
        error => {
          this.$message.error("删除文件失败 " + (error && error.toString()));
        }
      );
    },
    async deleteFile(row) {
      const res = await this.$confirm(
        `确认要删除该${row.isDirectory ? "目录" : "文件"}吗?`,
        "提示",
        {
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          type: "warning"
        }
      ).catch(() => false);
      if (!res) return;
      Axios.post(this.api + "/file/delete", {
        path: row.path,
        home: this.home
      }).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("删除成功");
            this.showFileList(this.currentPath);
          }
        },
        error => {
          this.$message.error("删除失败 " + (error && error.toString()));
        }
      );
    },
    async editFile(row) {
      Axios.get(this.api + "/file/get", {
        params: { path: row.path, home: this.home }
      }).then(
        res => {
          if (res.data.isSuccess) {
            let content = res.data.data;
            try {
              content = JSON.stringify(JSON.parse(content), null, 4);
            } catch (e) {
              content = res.data.data;
            }
            eventBus.$emit(
              "showEditor",
              "编辑 " + row.name,
              content,
              async (newContent, closeEditor) => {
                try {
                  const confirmRes = await this.$confirm(
                    "随意修改内容有可能会造成数据错乱，确认要保存编辑后的内容吗?",
                    "提示",
                    {
                      confirmButtonText: "确定",
                      cancelButtonText: "取消",
                      type: "warning"
                    }
                  ).catch(() => false);
                  if (!confirmRes) return;
                  Axios.post(this.api + "/file/save", {
                    path: row.path,
                    home: this.home,
                    content: newContent
                  }).then(
                    res => {
                      if (res.data.isSuccess) {
                        closeEditor();
                      }
                    },
                    error => {
                      this.$message.error(
                        "保存文件内容失败 " + (error && error.toString())
                      );
                    }
                  );
                } catch (e) {
                  this.$message.error("内容必须是JSON格式");
                }
              }
            );
          }
        },
        error => {
          this.$message.error(
            "获取文件内容失败 " + (error && error.toString())
          );
        }
      );
    },
    async mkdir() {
      const res = await this.$prompt("", "新建目录", {
        inputValue: "",
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        inputValidator: val => !!val || "目录名不能为空"
      }).catch(() => false);
      if (!res) return;
      Axios.post(this.api + "/file/mkdir", {
        path: this.currentPath,
        name: res.value,
        home: this.home
      }).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("操作成功");
            this.showFileList(this.currentPath);
          }
        },
        error => {
          this.$message.error("操作失败 " + (error && error.toString()));
        }
      );
    },
    async importFromFile(row) {
      if (row === true) {
        if (!this.fileSelection.length) {
          this.$message.error("请选择需要加入书架的书籍");
          return;
        }
      }
      Axios.post(this.api + "/file/importPreview", {
        path:
          row === true
            ? this.fileSelection.map(e => e.path)
            : [row.path],
        home: this.home
      }).then(
        res => {
          if (res.data.isSuccess) {
            if (!res.data.data || !res.data.data.length) {
              this.$message.error("没有选择可导入的书籍");
              return;
            }
            setTimeout(() => {
              eventBus.$emit("importPreview", res.data.data);
            }, 0);
          }
        },
        error => {
          this.$message.error("请求失败 " + (error && error.toString()));
        }
      );
    },
    upload() {
      this.$refs.bookRef.dispatchEvent(new MouseEvent("click"));
    },
    downloadFile(row) {
      const url = buildURL(this.api + "/file/download", {
        home: this.home,
        path: row.path,
        accessToken: this.$store.state.token
      });
      window.open(url, "__blank");
    },
    onFileChange(event) {
      if (
        !event.target ||
        !event.target.files ||
        !event.target.files.length
      ) {
        return;
      }
      let param = new FormData();
      for (let i = 0; i < event.target.files.length; i++) {
        const file = event.target.files[i];
        param.append("file" + i, file);
      }
      param.append("home", this.home);
      param.append("path", this.currentPath);
      Axios.post(this.api + "/file/upload", param, {
        headers: { "Content-Type": "multipart/form-data" }
      }).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("上传文件成功");
            this.showFileList(this.currentPath);
          }
        },
        error => {
          this.$message.error("上传文件 " + (error && error.toString()));
        }
      );
      this.$refs.bookRef.value = null;
    },
    async restoreFromFile(row) {
      const res = await this.$confirm(
        "确认要从该压缩文件恢复书源、书架、分组、RSS订阅数据、替换规则、书签、用户配置和Webdav书籍吗?",
        "提示",
        {
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          type: "warning"
        }
      ).catch(() => false);
      if (!res) return;
      Axios.post(this.api + "/file/restore", {
        path: row.path,
        home: this.home
      }).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("恢复成功");
            this.$root.$children[0].init(true);
          }
        },
        error => {
          this.$message.error("恢复失败 " + (error && error.toString()));
        }
      );
    }
  }
};
</script>
<style lang="stylus" scoped>
.float-left {
  float: left;
}
.float-right {
  float: right;
}
</style>
