<template>
  <el-dialog
    title="HttpTTS管理"
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
      <span class="el-dialog__title">HttpTTS管理</span>
    </div>
    <div class="source-container table-container">
      <el-table
        :data="$store.state.httpTTS"
        :height="dialogContentHeight"
        @selection-change="localSelection = $event"
      >
        <el-table-column
          type="selection"
          width="25"
          :fixed="$store.state.miniInterface"
        >
        </el-table-column>
        <el-table-column
          property="name"
          min-width="150px"
          label="名称"
          :fixed="$store.state.miniInterface"
        >
        </el-table-column>
        <el-table-column property="url" label="链接" min-width="150px">
        </el-table-column>
        <el-table-column label="操作" width="100px">
          <template slot-scope="scope">
            <el-button type="text" @click="editItem(scope.row)"
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
          @click="deleteReplaceRules"
          >批量删除<span v-if="localSelection.length"> ({{ localSelection.length }})</span></el-button
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
          @click="addNew"
          >添加</el-button
        >
        <input
          ref="fileRef"
          type="file"
          @change="onFileChange($event)"
          style="display:none"
        />
      </div>
    </div>
  </el-dialog>
</template>

<script>
import { mapGetters } from "vuex";
import Axios from "../plugins/axios";
import eventBus from "../plugins/eventBus";

export default {
  model: {
    prop: "show",
    event: "setShow"
  },
  name: "HttpTTS",
  data() {
    return {
      localSelection: []
    };
  },
  props: ["show"],
  computed: {
    ...mapGetters(["dialogWidth", "dialogTop", "dialogContentHeight"])
  },
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
    cancel() {
      this.$emit("setShow", false);
    },
    async deleteReplaceRules() {
      if (!this.localSelection.length) {
        this.$message.error("请选择需要删除的HttpTTS");
        return;
      }
      const res = await this.$confirm(
        "确认要删除所选择的HttpTTS吗?",
        "提示",
        {
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          type: "warning"
        }
      ).catch(() => false);
      if (!res) return;
      Axios.post(this.api + "/httpTTS/deleteMulti", this.localSelection).then(
        res => {
          if (res.data.isSuccess) {
            this.localSelection = [];
            this.$message.success("删除HttpTTS成功");
            this.$root.$children[0].loadHttpTTS(true);
          }
        },
        error => {
          this.$message.error(
            "删除HttpTTS失败 " + (error && error.toString())
          );
        }
      );
    },
    editItem(row) {
      this.showEditor({ ...row }, "编辑HttpTTS");
    },
    addNew() {
      this.showEditor(
        { name: "新增HttpTTS", url: "", contentType: "", header: "" },
        "新增HttpTTS"
      );
    },
    showEditor(item, title) {
      eventBus.$emit(
        "showEditor",
        title,
        JSON.stringify(item, null, 4),
        (content, closeEditor) => {
          try {
            const data = JSON.parse(content);
            if (!data.name) {
              this.$message.error("名称不能为空");
              return;
            }
            if (!data.url) {
              this.$message.error("链接不能为空");
              return;
            }
            Axios.post(this.api + "/httpTTS/save", data).then(
              res => {
                if (res.data.isSuccess) {
                  closeEditor();
                  this.$message.success("保存成功");
                  this.$root.$children[0].loadHttpTTS(true);
                }
              },
              error => {
                this.$message.error(
                  "保存失败 " + (error && error.toString())
                );
              }
            );
          } catch (e) {
            this.$message.error("必须是JSON格式");
          }
        }
      );
    },
    uploadFile() {
      this.$refs.fileRef.dispatchEvent(new MouseEvent("click"));
    },
    onFileChange(event) {
      const rawFile = event.target.files && event.target.files[0];
      if (!rawFile) {
        return;
      }
      const reader = new FileReader();
      reader.onload = e => {
        const data = e.target.result;
        try {
          const list = JSON.parse(data);
          if (Array.isArray(list) && list.length) {
            this.comfirmImport(list);
          }
        } catch (error) {
          this.$message.error("HttpTTS文件错误");
        }
      };
      reader.onerror = () => {
        let param = new FormData();
        param.append("file", rawFile);
        Axios.post(this.api + "/readSourceFile", param, {
          headers: { "Content-Type": "multipart/form-data" }
        }).then(
          res => {
            if (res.data.isSuccess) {
              let list = [];
              res.data.data.forEach(v => {
                try {
                  const data = JSON.parse(v);
                  if (Array.isArray(data)) {
                    list = list.concat(data);
                  }
                } catch (error) {
                  //
                }
              });
              if (list.length) {
                this.comfirmImport(list);
              } else {
                this.$message.error("HttpTTS文件错误");
              }
            }
          },
          error => {
            this.$message.error(
              "读取HttpTTS文件内容失败 " + (error && error.toString())
            );
          }
        );
      };
      reader.readAsText(rawFile);
      this.$refs.fileRef.value = null;
    },
    async comfirmImport(list) {
      const res = await this.$confirm(
        `确认要导入文件中的${list.length}条HttpTTS吗?`,
        "提示",
        {
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          type: "warning"
        }
      ).catch(() => false);
      if (!res) return;
      Axios.post(this.api + "/httpTTS/saveMulti", list).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("导入HttpTTS成功");
            this.$root.$children[0].loadHttpTTS(true);
          }
        },
        error => {
          this.$message.error(
            "导入HttpTTS失败 " + (error && error.toString())
          );
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
</style>
