import { mapGetters } from "vuex";
import Axios from "../plugins/axios";
import eventBus from "../plugins/eventBus";

export const ReplaceRule = {
  model: {
    prop: "show",
    event: "setShow"
  },
  name: "ReplaceRule",
  template: `
  <el-dialog
    title="替换规则管理"
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
      <span class="el-dialog__title">替换规则管理</span>
    </div>
    <div class="source-container table-container">
      <el-table
        :data="$store.state.filterRules"
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
          label="规则名称"
          :fixed="$store.state.miniInterface"
        >
        </el-table-column>
        <el-table-column property="scope" label="替换范围" min-width="150px">
        </el-table-column>
        <el-table-column property="isEnabled" label="是否启用" min-width="80">
          <template slot-scope="scope">
            <el-switch
              v-model="scope.row.isEnabled"
              active-color="#13ce66"
              inactive-color="#ff4949"
              :active-value="true"
              :inactive-value="false"
              @change="toggleRuleEnabled(scope.row, $event)"
            >
            </el-switch>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="100px">
          <template slot-scope="scope">
            <el-button type="text" @click="editReplaceRule(scope.row)"
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
`,
  data() {
    return {
      localSelection: []
    };
  },
  computed: {
    ...mapGetters(["dialogWidth", "dialogTop", "dialogContentHeight"])
  },
  props: ["show"],
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
        this.$message.error("请选择需要删除的替换规则");
        return;
      }
      const res = await this.$confirm("确认要删除所选择的替换规则吗?", "提示", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning"
      }).catch(() => {
        return false;
      });
      if (!res) {
        return;
      }
      Axios.post(this.api + "/deleteReplaceRules", this.localSelection).then(
        res => {
          if (res.data.isSuccess) {
            this.localSelection = [];
            this.$message.success("删除替换规则成功");
            this.$root.$children[0].loadReplaceRules(true);
          }
        },
        error => {
          this.$message.error(
            "删除替换规则失败 " + (error && error.toString())
          );
        }
      );
    },
    toggleRuleEnabled(rule, isEnabled) {
      Axios.post("/saveReplaceRule", { ...rule, isEnabled }).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("修改成功");
            this.$root.$children[0].loadReplaceRules(true);
          }
        },
        error => {
          this.$message.error("修改失败 " + (error && error.toString()));
        }
      );
    },
    editReplaceRule(row) {
      eventBus.$emit("showReplaceRuleForm", { ...row }, false);
    },
    addNew() {
      eventBus.$emit(
        "showReplaceRuleForm",
        {
          name: "文本替换",
          pattern: "",
          replacement: "",
          isRegex: false,
          isEnabled: true,
          scope: this.$store.getters.readingBook
            ? this.$store.getters.readingBook.name +
              ";" +
              this.$store.getters.readingBook.bookUrl
            : "*"
        },
        false
      );
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
          const ruleList = JSON.parse(data as string);
          if (Array.isArray(ruleList) && ruleList.length) {
            this.comfirmImport(ruleList);
          }
        } catch (error) {
          this.$message.error("替换规则文件错误");
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
              let ruleList = [];
              res.data.data.forEach(v => {
                try {
                  const data = JSON.parse(v);
                  if (Array.isArray(data)) {
                    ruleList = ruleList.concat(data);
                  }
                } catch (error) {
                  //
                }
              });
              if (ruleList.length) {
                this.comfirmImport(ruleList);
              } else {
                this.$message.error("替换规则文件错误");
              }
            }
          },
          error => {
            this.$message.error(
              "读取替换规则文件内容失败 " + (error && error.toString())
            );
          }
        );
      };
      reader.readAsText(rawFile);
      this.$refs.fileRef.value = null;
    },
    async comfirmImport(ruleList) {
      const res = await this.$confirm(
        `确认要导入文件中的${ruleList.length}条替换规则吗?`,
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
      Axios.post(this.api + "/saveReplaceRules", ruleList).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("导入替换规则成功");
            this.$root.$children[0].loadReplaceRules(true);
          }
        },
        error => {
          this.$message.error(
            "导入替换规则失败 " + (error && error.toString())
          );
        }
      );
    }
  }
};

export const ReplaceRuleStyle = `
.float-left {
  float: left;
}
.dialog-footer {
  display: flex;
  flex-direction: row-reverse;
  flex-wrap: wrap;
  justify-content: space-between;
  .float-left {
    margin-right: 5px;
    margin-bottom: 5px;
  }
}
`;
