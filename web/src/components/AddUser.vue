<template>
  <el-dialog
    :title="isAdd ? '新增用户' : '修改用户'"
    :visible.sync="show"
    :width="dialogSmallWidth"
    :top="dialogTop"
    :fullscreen="$store.state.miniInterface"
    :class="
      isWebApp && !$store.getters.isNight ? 'status-bar-light-bg-dialog' : ''
    "
    v-if="$store.getters.isNormalPage"
    :before-close="cancel"
  >
    <el-form :model="userForm">
      <el-form-item label="用户名">
        <el-input
          v-model="userForm.username"
          autocomplete="off"
          :readonly="!isAdd"
        ></el-input>
      </el-form-item>
      <el-form-item v-if="isAdd" label="密码">
        <el-input
          type="password"
          v-model="userForm.password"
          autocomplete="off"
          show-password
        ></el-input>
      </el-form-item>
      <el-form-item label="书籍上限">
        <el-input-number
          v-model="userForm.bookLimit"
          :min="1"
        ></el-input-number>
      </el-form-item>
      <el-form-item label="书源上限">
        <el-input-number
          v-model="userForm.bookSourceLimit"
          :min="1"
        ></el-input-number>
      </el-form-item>
      <el-checkbox v-model="userForm.enableWebdav">启用webdav</el-checkbox>
      <el-checkbox v-model="userForm.enableLocalStore">启用书仓</el-checkbox>
      <el-checkbox v-model="userForm.enableBookSource">编辑书源</el-checkbox>
      <el-checkbox v-model="userForm.enableRssSource">编辑RSS源</el-checkbox>
    </el-form>
    <div slot="footer" class="dialog-footer">
      <el-button size="medium" @click="cancel">取 消</el-button>
      <el-button size="medium" type="primary" @click="save">确 定</el-button>
    </div>
  </el-dialog>
</template>

<script>
import { mapGetters } from "vuex";
import Axios from "../plugins/axios";

const defaultUserForm = {
  username: "",
  password: "",
  bookLimit: 200,
  bookSourceLimit: 100,
  enableWebdav: false,
  enableLocalStore: false,
  enableBookSource: true,
  enableRssSource: true
};

export default {
  model: {
    prop: "show",
    event: "setShow"
  },
  name: "UserForm",
  data() {
    return {
      isAdd: true,
      userForm: { ...defaultUserForm }
    };
  },
  props: ["show", "user"],
  computed: {
    ...mapGetters(["dialogSmallWidth", "dialogTop"])
  },
  watch: {
    show(isVisible) {
      if (isVisible) {
        if (this.user && this.user.username) {
          this.userForm = { ...defaultUserForm, ...this.user };
          this.isAdd = false;
        } else {
          this.userForm = { ...defaultUserForm };
          this.isAdd = true;
        }
      }
    }
  },
  methods: {
    cancel() {
      this.$emit("setShow", false);
    },
    save() {
      if (!this.userForm.username) {
        this.$message.success("用户名不能为空");
        return;
      }
      if (!this.isAdd || this.userForm.password) {
        Axios.post(
          this.api + (this.isAdd ? "/addUser" : "/updateUser"),
          this.userForm
        ).then(
          res => {
            if (res.data.isSuccess) {
              this.$message.success("操作成功");
              this.cancel();
              const userList = res.data.data.map(v => ({
                ...v,
                userNS: v.username
              }));
              this.$store.commit("setUserList", userList);
            }
          },
          error => {
            this.$message.error("操作失败 " + (error && error.toString()));
          }
        );
      } else {
        this.$message.success("密码不能为空");
      }
    }
  }
};
</script>
<style lang="stylus" scoped></style>
