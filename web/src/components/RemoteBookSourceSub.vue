<template>
  <el-dialog
    title="书源订阅管理"
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
        >书源订阅管理
        <span class="float-right span-btn" @click="showForm(null)">新增</span>
      </span>
    </div>
    <div class="source-container table-container">
      <el-table
        :data="remoteBookSourceList"
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
          min-width="120px"
          label="名称"
          :fixed="$store.state.miniInterface"
        >
        </el-table-column>
        <el-table-column
          property="link"
          label="链接"
          min-width="200px"
        >
        </el-table-column>
        <el-table-column
          property="lastSyncTime"
          label="最后同步"
          :formatter="formatTableField"
          width="120px"
        >
        </el-table-column>
        <el-table-column label="操作" width="120px">
          <template slot-scope="scope">
            <el-button
              type="text"
              @click="showForm(scope.row, scope.$index)"
              >编辑</el-button
            >
            <el-button
              type="text"
              @click="sync(scope.row, scope.$index)"
              :loading="loadingIndex === scope.$index"
              >同步</el-button
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
        @click="deleteRemoteBookSourceSub"
        >批量删除</el-button
      >
      <span class="check-tip">已选择 {{ localSelection.length }} 个</span>
      <el-button size="medium" @click="cancel">取消</el-button>
    </div>
  </el-dialog>
</template>

<script>
import { mapGetters } from "vuex";
import Axios from "../plugins/axios";

export default {
  model: {
    prop: "show",
    event: "setShow"
  },
  name: "RemoteBookSourceSub",
  data() {
    return {
      localSelection: [],
      filePath: "remoteBookSourceSub.json",
      remoteBookSourceList: [],
      loadingIndex: -1
    };
  },
  props: ["show"],
  computed: {
    ...mapGetters(["dialogWidth", "dialogTop", "dialogContentHeight"])
  },
  watch: {
    show(isVisible) {
      if (isVisible) {
        this.loadRemoteBookSourceList();
      }
    }
  },
  methods: {
    formatTableField(row, column, cellValue) {
      switch (column.property) {
        case "lastSyncTime":
          return cellValue
            ? new Date(cellValue).format("yy-MM-dd hh:mm")
            : "";
        default:
          return cellValue;
      }
    },
    cancel() {
      this.$emit("setShow", false);
    },
    loadRemoteBookSourceList() {
      Axios.get("/file/get", {
        params: { path: this.filePath, home: "__HOME__" },
        silent: true
      })
        .then(res => {
          if (res.data.isSuccess && res.data.data) {
            try {
              const data = JSON.parse(res.data.data);
              if (Array.isArray(data)) {
                this.remoteBookSourceList = data;
              }
            } catch (e) {
              // ignore
            }
          }
        })
        .catch(() => null);
    },
    async deleteRemoteBookSourceSub() {
      if (!this.localSelection.length) {
        this.$message.error("请选择需要删除的书源订阅");
        return;
      }
      const res = await this.$confirm(
        "确认要删除所选择的书源订阅吗?",
        "提示",
        {
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          type: "warning"
        }
      ).catch(() => false);
      if (!res) return;
      const remaining = this.remoteBookSourceList.filter(
        t => !this.localSelection.includes(t)
      );
      this.saveData(remaining);
    },
    sync(item, index) {
      this.loadingIndex = index;
      Axios.post("/saveFromRemoteSource", { url: item.link }, { timeout: 600000 }).then(
        res => {
          this.loadingIndex = -1;
          this.$root.$children[0].loadBookSource(true);
          if (res.data.isSuccess) {
            this.remoteBookSourceList[index].lastSyncTime = new Date().getTime();
            this.saveData(this.remoteBookSourceList, true);
          }
        },
        error => {
          this.loadingIndex = -1;
          this.$message.error("同步失败 " + (error && error.toString()));
        }
      );
    },
    async showForm(item, index) {
      const isAdd = !item;
      item = { ...(item || { name: "", link: "", lastSyncTime: null }) };
      const res = await this.$prompt(
        isAdd ? "请输入订阅信息 (JSON格式)" : "编辑订阅信息",
        isAdd ? "新增订阅" : "编辑订阅",
        {
          inputType: "textarea",
          inputValue: JSON.stringify(
            { name: item.name, link: item.link },
            null,
            2
          ),
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          inputValidator: val => {
            try {
              const obj = JSON.parse(val);
              if (!obj.name) return "名称不能为空";
              if (!obj.link) return "链接不能为空";
              return true;
            } catch (e) {
              return "必须是JSON格式";
            }
          }
        }
      ).catch(() => false);
      if (!res) return;
      try {
        const data = JSON.parse(res.value);
        let list = [].concat(this.remoteBookSourceList);
        if (isAdd) {
          list = list.concat([
            { name: data.name, link: data.link, lastSyncTime: null }
          ]);
        } else {
          list[index] = {
            ...list[index],
            name: data.name,
            link: data.link
          };
        }
        this.saveData(list);
      } catch (e) {
        this.$message.error("格式错误");
      }
    },
    saveData(data, silent) {
      Axios.post(
        "/file/save",
        {
          path: this.filePath,
          content: JSON.stringify(data),
          home: "__HOME__"
        },
        { silent: true }
      )
        .then(res => {
          if (res.data.isSuccess) {
            this.localSelection = [];
            if (!silent) {
              this.$message.success("操作成功");
            }
            this.loadRemoteBookSourceList();
          }
        })
        .catch(() => null);
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
