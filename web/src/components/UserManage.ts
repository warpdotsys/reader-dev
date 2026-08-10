import { mapGetters } from "vuex";
import Axios from "../plugins/axios";
import eventBus from "../plugins/eventBus";
import { formatSize } from "../plugins/helper";

export const UserManage = {
  model: {
    prop: "show",
    event: "setShow"
  },
  name: "UserManage",
  data() {
    return {
      manageUserSelection: [],
      search: "",
      pagination: {
        page: 1,
        size: 25
      },
      sortable: {
        prop: "",
        order: null
      }
    };
  },
  props: ["show"],
  computed: {
    ...mapGetters(["dialogWidth", "dialogTop", "dialogContentHeight"]),
    userList: {
      get() {
        return this.$store.state.userList;
      },
      set(val) {
        this.$store.commit("setUserList", val);
      }
    },
    filterList() {
      return this.userList.filter(
        v =>
          v.userNS !== "default" &&
          (!this.search ||
            v.username.toLowerCase().includes(this.search.toLowerCase()))
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
      const start = (this.pagination.page - 1) * this.pagination.size;
      return start > this.sortList.length
        ? []
        : this.sortList.slice(
            start,
            Math.min(start + this.pagination.size, this.sortList.length)
          );
    }
  },
  watch: {
    show(isVisible: boolean) {
      if (isVisible) {
        this.manageUserSelection = [];
      }
    }
  },
  methods: {
    cancel() {
      this.$emit("setShow", false);
    },
    sortChange({ prop, order }: any) {
      this.sortable = { prop, order };
    },
    showAddUserDialog() {
      eventBus.$emit("showUserFormDialog");
    },
    async clearInactiveUser() {
      const res = await this.$prompt(
        "请输入需要清理的未登录天数",
        "清理不活跃用户",
        {
          inputValue: 31,
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          inputValidator(v) {
            if (!v) {
              return "天数不能为空";
            }
            return !isNaN(parseInt(v)) || "天数必须是数字";
          }
        }
      ).catch(() => {
        return false;
      });
      if (!res || !res.value) {
        return;
      }
      Axios.post(
        this.api + "/clearInactiveUsers",
        {
          inactiveDay: parseInt(res.value)
        },
        {
          timeout: 0
        }
      ).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("清理不活跃用户成功");
            this.userList = res.data.data.map(v => ({
              ...v,
              userNS: v.username
            }));
          }
        },
        error => {
          this.$message.error(
            "清理不活跃用户失败 " + (error && error.toString())
          );
        }
      );
    },
    formatTableField(row: any, column: any, cellValue: any) {
      switch (column.property) {
        case "createdAt":
        case "lastLoginAt":
        case "lastModified":
          return cellValue ? new Date(cellValue).format("yy-MM-dd hh:mm") : "";
        case "size":
          return row.isDirectory ? "" : formatSize(cellValue);
        default:
          return cellValue;
      }
    },
    isUserSelectable(user: any) {
      return user.userNS !== "default";
    },
    async deleteUserList() {
      if (!this.manageUserSelection.length) {
        this.$message.error("请选择需要删除的用户");
        return;
      }
      const res = await this.$confirm("确认要删除所选择的用户吗?", "提示", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning"
      }).catch(() => {
        return false;
      });
      if (!res) {
        return;
      }
      Axios.post(
        this.api + "/deleteUsers",
        this.manageUserSelection.map(v => v.username),
        {
          timeout: 0
        }
      ).then(
        res => {
          if (res.data.isSuccess) {
            this.manageUserSelection = [];
            this.$message.success("删除用户成功");
            this.userList = res.data.data.map(v => ({
              ...v,
              userNS: v.username
            }));
          }
        },
        error => {
          this.$message.error("删除用户失败 " + (error && error.toString()));
        }
      );
    },
    async deleteUserBookSource() {
      if (!this.manageUserSelection.length) {
        this.$message.error("请选择需要删除书源的用户");
        return;
      }
      const res = await this.$confirm("确认要删除所选择的用户书源吗?", "提示", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning"
      }).catch(() => {
        return false;
      });
      if (!res) {
        return;
      }
      Axios.post(
        this.api + "/deleteUserBookSource",
        this.manageUserSelection.map(v => v.username)
      ).then(
        res => {
          if (res.data.isSuccess) {
            this.manageUserSelection = [];
            this.$message.success("操作成功");
          }
        },
        error => {
          this.$message.error("操作失败 " + (error && error.toString()));
        }
      );
    },
    toggleUserWebdav(user: any, enableWebdav: any) {
      Axios.post(this.api + "/updateUser", {
        username: user.username,
        enableWebdav
      }).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("修改成功");
            this.userList = res.data.data.map(v => ({
              ...v,
              userNS: v.username
            }));
          }
        },
        error => {
          this.$message.error("修改失败 " + (error && error.toString()));
        }
      );
    },
    toggleUserLocalStore(user: any, enableLocalStore: any) {
      Axios.post(this.api + "/updateUser", {
        username: user.username,
        enableLocalStore
      }).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("修改成功");
            this.userList = res.data.data.map(v => ({
              ...v,
              userNS: v.username
            }));
          }
        },
        error => {
          this.$message.error("修改失败 " + (error && error.toString()));
        }
      );
    },
    editUser(user: any) {
      eventBus.$emit("showUserFormDialog", user);
    },
    async setAsDefaultBookSources(user: any) {
      const res = await this.$confirm(
        `确认要将用户${user.username}的书源设为默认书源（新用户有效）吗?`,
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
      return Axios.post(this.api + "/setAsDefaultBookSources", {
        username: user.username
      }).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("设置成功");
          }
        },
        error => {
          this.$message.error("设置失败 " + (error && error.toString()));
        }
      );
    },
    async resetPassword(user: any) {
      const res = await this.$prompt("", "重置密码", {
        inputValue: "",
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        inputValidator(v) {
          return !!v || "密码不能为空";
        }
      }).catch(() => {
        return false;
      });
      if (!res) {
        return;
      }
      Axios.post(this.api + "/resetPassword", {
        username: user.username,
        password: res.value
      }).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("重置密码成功");
          }
        },
        error => {
          this.$message.error("重置密码失败 " + (error && error.toString()));
        }
      );
    },
    filterHandler(value: any, row: any, column: any) {
      return row[column.property] === value;
    }
  }
};

export const UserManageTemplate = `<template>
  <el-dialog
    title="用户管理"
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
      <span class="el-dialog__title">用户管理 </span>
      <span class="title-center">
        <el-input
          class="search-input"
          size="mini"
          placeholder="输入关键字搜索"
          v-model="search"
        ></el-input>
      </span>
    </div>
    <div class="source-container table-container">
      <el-table
        :data="showList"
        :height="dialogContentHeight - 42"
        @selection-change="manageUserSelection = $event"
        @sort-change="sortChange"
      >
        <el-table-column
          type="selection"
          width="25"
          :selectable="isUserSelectable"
          :fixed="$store.state.miniInterface"
        >
        </el-table-column>
        <el-table-column
          property="username"
          label="用户名"
          min-width="100"
          sortable="custom"
          :fixed="$store.state.miniInterface"
        ></el-table-column>
        <el-table-column
          property="lastLoginAt"
          label="上次登录"
          sortable="custom"
          :formatter="formatTableField"
          min-width="120"
        ></el-table-column>
        <el-table-column
          property="createdAt"
          label="注册时间"
          sortable="custom"
          :formatter="formatTableField"
          min-width="120"
        ></el-table-column>
        <el-table-column
          property="enableWebdav"
          label="WebDAV"
          min-width="80"
          :filters="[
            { text: '开启', value: true },
            { text: '关闭', value: false }
          ]"
          :filter-method="filterHandler"
        >
          <template slot-scope="scope">
            <el-switch
              v-if="scope.row.userNS !== 'default'"
              v-model="scope.row.enableWebdav"
              active-color="#13ce66"
              inactive-color="#ff4949"
              :active-value="true"
              :inactive-value="false"
              @change="toggleUserWebdav(scope.row, $event)"
            >
            </el-switch>
          </template>
        </el-table-column>
        <el-table-column
          property="enableLocalStore"
          label="书仓"
          min-width="80"
          :filters="[
            { text: '开启', value: true },
            { text: '关闭', value: false }
          ]"
          :filter-method="filterHandler"
        >
          <template slot-scope="scope">
            <el-switch
              v-if="scope.row.userNS !== 'default'"
              v-model="scope.row.enableLocalStore"
              active-color="#13ce66"
              inactive-color="#ff4949"
              :active-value="true"
              :inactive-value="false"
              @change="toggleUserLocalStore(scope.row, $event)"
            >
            </el-switch>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="100px">
          <template slot-scope="scope">
            <el-button
              v-if="scope.row.userNS !== 'default'"
              class="text-button"
              type="text"
              @click="editUser(scope.row)"
              >修改</el-button
            >
            <el-button
              v-if="scope.row.userNS !== 'default'"
              class="text-button"
              type="text"
              @click="resetPassword(scope.row)"
              >重置密码</el-button
            >
            <el-button
              v-if="scope.row.userNS !== 'default'"
              class="text-button"
              type="text"
              @click="setAsDefaultBookSources(scope.row)"
              >设为默认书源</el-button
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
          @click="deleteUserList"
          >批量删除<span v-if="manageUserSelection.length">
            ({{ manageUserSelection.length }})</span
          ></el-button
        >
        <el-button
          type="primary"
          size="medium"
          class="float-left"
          @click="deleteUserBookSource"
          >使用默认书源</el-button
        >
        <el-button
          type="primary"
          size="medium"
          class="float-left"
          @click="clearInactiveUser()"
          >清理</el-button
        >
        <el-button
          type="primary"
          size="medium"
          class="float-left"
          @click="showAddUserDialog()"
          >新增</el-button
        >
      </div>
      <div class="source-pagination">
        <el-pagination
          :current-page="pagination.page"
          :page-sizes="[25, 50, 100, 200, 300, 400, filterList.length]"
          :page-size="pagination.size"
          layout="total, sizes, prev, pager, next"
          :total="filterList.length"
          :pager-count="$store.state.miniInterface ? 5 : 7"
          @update:currentPage="pagination.page = $event"
          @update:pageSize="pagination.size = $event"
        ></el-pagination>
      </div>
    </div>
  </el-dialog>
</template>`;

export const UserManageStyle = `<style lang="stylus" scoped>
.float-right {
  float: right;
}
.float-left {
  float: left;
}
.dialog-footer {
  .float-left {
    margin-right: 5px;
    margin-bottom: 5px;
  }
}
.text-button {
  padding: 3px 5px;
}
.source-pagination {
  margin-top: 5px;
  text-align: right;
}
</style>`;
