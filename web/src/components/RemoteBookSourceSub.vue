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
      <span class="el-dialog__title">书源订阅管理 </span>
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
          min-width="150px"
          label="名称"
          :fixed="$store.state.miniInterface"
        >
        </el-table-column>
        <el-table-column property="link" label="链接" min-width="150px">
        </el-table-column>
        <el-table-column
          property="lastSyncTime"
          label="上次同步"
          :formatter="formatTableField"
          min-width="150px"
        >
        </el-table-column>
        <el-table-column label="操作" width="120px">
          <template slot-scope="scope">
            <el-button
              type="text"
              @click="showForm(scope.row, scope.$index)"
              >修改</el-button
            >
            <el-button
              type="text"
              @click="sync(scope.row, scope.$index)"
              :disabled="loadingIndex >= 0"
              ><i
                v-if="loadingIndex === scope.$index"
                class="el-icon-loading"
              ></i
              >{{ loadingIndex === scope.$index ? "同步中" : "同步" }}</el-button
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
          @click="deleteRemoteBookSourceSub"
          >批量删除<span v-if="localSelection.length"> ({{ localSelection.length }})</span></el-button
        >
        <el-button
          type="primary"
          size="medium"
          class="float-left"
          @click="showForm(false)"
          >新增</el-button
        >
      </div>
    </div>
  </el-dialog>
</template>

<script>
import { mapGetters } from "vuex";
import Axios from "../plugins/axios";

const getFile = (path, home) =>
  Axios.get("/file/get", {
    params: { path, home: home || "__HOME__" },
    silent: true
  })
    .then(res => {
      return res.data.isSuccess ? res.data.data : null;
    })
    .catch(() => {
      return null;
    });

const saveFile = (path, content, home) =>
  Axios.post(
    "/file/save",
    {
      path,
      content,
      home: home || "__HOME__"
    },
    { silent: true }
  )
    .then(res => {
      return !!res.data.isSuccess;
    })
    .catch(() => {
      return null;
    });

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
    async loadRemoteBookSourceList() {
      const data = await getFile(this.filePath);
      if (data) {
        try {
          const list = JSON.parse(data);
          if (Array.isArray(list)) {
            this.remoteBookSourceList = list;
          }
        } catch (error) {
        /* ignore */
      }
      }
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
      ).catch(() => {
        return false;
      });
      if (!res) {
        return;
      }
      const remaining = this.remoteBookSourceList.filter(
        item => !this.localSelection.includes(item)
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
            this.remoteBookSourceList[index].lastSyncTime =
              new Date().getTime();
            this.saveData(this.remoteBookSourceList, true);
          }
        },
        error => {
          this.$message.error("同步失败 " + (error && error.toString()));
        }
      );
    },
    async showForm(item, index) {
      const isAdd = !item;
      item = { ...(item || { name: "", link: "", lastSyncTime: null }) };
      const items = [
        { name: "name", label: "名称", type: "input" },
        { name: "link", label: "链接", type: "input" }
      ];
      const res = await this.$msgbox({
        title: isAdd ? "新增订阅" : "编辑订阅",
        message: this.renderForm(this.filePath, item, items, value => {
          item = value;
        }),
        showCancelButton: true,
        confirmButtonText: "确定",
        cancelButtonText: "取消"
      }).catch(error => {
        return error === "close" ? "close" : error;
      });
      if (res !== "confirm") {
        return false;
      }
      const list = [].concat(this.remoteBookSourceList);
      if (isAdd) {
        list.push(item);
      } else {
        list[index] = item;
      }
      this.saveData(list);
    },
    async saveData(data, silent) {
      const res = await saveFile(this.filePath, JSON.stringify(data));
      if (res) {
        this.localSelection = [];
        if (!silent) {
          this.$message.success("操作成功");
        }
        this.loadRemoteBookSourceList();
      }
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
.dialog-footer {
  .float-left {
    margin-right: 5px;
    margin-bottom: 5px;
  }
}
</style>
