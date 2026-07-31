<template>
  <el-dialog
    title="密钥管理"
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
        >密钥管理
        <span class="float-right span-btn" @click="showForm">生成密钥</span>
      </span>
    </div>
    <div class="source-container table-container">
      <el-table
        :data="list"
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
          property="host"
          min-width="150px"
          label="Host"
          :fixed="$store.state.miniInterface"
        >
        </el-table-column>
        <el-table-column
          property="code"
          label="Code"
          min-width="100px"
        >
        </el-table-column>
        <el-table-column
          property="type"
          label="类型"
          min-width="100px"
        >
        </el-table-column>
        <el-table-column
          property="userMaxLimit"
          label="用户上限"
          min-width="100px"
        >
        </el-table-column>
        <el-table-column
          property="expiredAt"
          label="过期时间"
          :formatter="formatTableField"
          min-width="100px"
        >
        </el-table-column>
        <el-table-column
          property="simpleWebExpiredAt"
          label="Kindle过期"
          :formatter="formatTableField"
          min-width="150px"
        >
        </el-table-column>
        <el-table-column
          property="activeTime"
          label="激活时间"
          :formatter="formatTableField"
          min-width="150px"
        >
        </el-table-column>
        <el-table-column
          property="activeIp"
          label="激活IP"
          min-width="100px"
        >
        </el-table-column>
        <el-table-column
          property="lastOnlineTime"
          label="上次在线时间"
          :formatter="formatTableField"
          min-width="150px"
        >
        </el-table-column>
        <el-table-column
          property="lastOnlineIp"
          label="上次在线IP"
          min-width="100px"
        >
        </el-table-column>
      </el-table>
    </div>
    <div slot="footer" class="dialog-footer">
      <div>
        <el-button
          type="primary"
          size="medium"
          class="float-left"
          @click="deleteItems"
          >批量删除<span v-if="localSelection.length"> ({{ localSelection.length }})</span></el-button
        >
        <el-button
          type="primary"
          size="medium"
          class="float-left"
          @click="showForm"
          >生成密钥</el-button
        >
      </div>
    </div>
  </el-dialog>
</template>

<script>
import { mapGetters } from "vuex";
import Axios from "../plugins/axios";
import eventBus from "../plugins/eventBus";
import { getCache, setCache } from "../plugins/cache";

export default {
  model: {
    prop: "show",
    event: "setShow"
  },
  name: "ActiveLicense",
  data() {
    return {
      localSelection: [],
      filePath: "data/activeLicense.json",
      list: [],
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
        this.loadList();
      }
    }
  },
  methods: {
    formatTableField(row, column, cellValue) {
      switch (column.property) {
        case "expiredAt":
        case "simpleWebExpiredAt":
        case "activeTime":
        case "lastOnlineTime":
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
    loadList() {
      Axios.get(this.api + "/file/get", {
        params: { path: this.filePath, home: "__STORAGE__" },
        silent: true
      })
        .then(res => {
          if (res.data.isSuccess && res.data.data) {
            try {
              const data = JSON.parse(res.data.data);
              if (Array.isArray(data)) {
                this.list = data;
              }
            } catch (e) {
              // ignore
            }
          }
        })
        .catch(() => null);
    },
    async deleteItems() {
      if (!this.localSelection.length) {
        this.$message.error("请选择需要删除的记录");
        return;
      }
      const res = await this.$confirm("确认要删除所选择的记录吗?", "提示", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning"
      }).catch(() => false);
      if (!res) return;
      const remaining = this.list.filter(
        t => !this.localSelection.includes(t)
      );
      this.saveData(remaining);
    },
    async showForm() {
      let config = getCache("lastGenerateLicenseConfig") || {
        host: "*",
        userMaxLimit: 15,
        expiredAt: 0,
        openApi: false,
        simpleWebExpiredAt: 1735660799000,
        key: "Pvkp7tMQJpi4kWBE",
        type: "basic",
        code: ""
      };
      const items = [
        { name: "host", label: "Host", type: "input" },
        { name: "userMaxLimit", label: "用户上限", type: "input" },
        { name: "expiredAt", label: "过期时间", type: "input" },
        { name: "simpleWebExpiredAt", label: "Kindle有效期", type: "input" },
        { name: "type", label: "类型", type: "input" },
        { name: "code", label: "Code", type: "input" }
      ];
      const res = await this.$msgbox({
        title: "生成密钥",
        message: this.renderForm(this.filePath, config, items, value => {
          config = value;
        }),
        showCancelButton: true,
        confirmButtonText: "确定",
        cancelButtonText: "取消"
      }).catch(error => (error === "close" ? "close" : error));
      if (res !== "confirm") return false;

      config.userMaxLimit = parseInt(config.userMaxLimit);
      config.expiredAt = "" + config.expiredAt;
      if (config.expiredAt && config.expiredAt.indexOf("-") > 0) {
        config.expiredAt = new Date(
          config.expiredAt.replace(/-/g, "/") + " 23:59:59"
        ).getTime();
      } else {
        config.expiredAt = parseInt(config.expiredAt);
      }
      config.simpleWebExpiredAt = "" + config.simpleWebExpiredAt;
      if (
        config.simpleWebExpiredAt &&
        config.simpleWebExpiredAt.indexOf("-") > 0
      ) {
        config.simpleWebExpiredAt = new Date(
          config.simpleWebExpiredAt.replace(/-/g, "/") + " 23:59:59"
        ).getTime();
      } else {
        config.simpleWebExpiredAt = parseInt(config.simpleWebExpiredAt);
      }
      setCache("lastGenerateLicenseConfig", config);
      Axios.post(this.api + "/generateLicense", config).then(
        res => {
          if (res.data.isSuccess) {
            eventBus.$emit(
              "showEditor",
              "生成授权",
              JSON.stringify(res.data.data, null, 4)
            );
          }
        },
        error => {
          this.$message.error("生成密钥失败 " + (error && error.toString()));
        }
      );
    },
    saveData(data) {
      Axios.post(
        this.api + "/file/save",
        {
          path: this.filePath,
          content: JSON.stringify(data),
          home: "__STORAGE__"
        },
        { silent: true }
      )
        .then(res => {
          if (res.data.isSuccess) {
            this.localSelection = [];
            this.$message.success("操作成功");
            this.loadList();
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
