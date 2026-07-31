<template>
  <el-dialog
    title="授权管理"
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
        >授权管理
        <span class="float-right span-btn" @click="importLicense">更新密钥</span>
      </span>
    </div>
    <div>
      <div class="license-item">
        <span class="license-name">授权类型</span>
        <span class="license-value">{{ license.type }}</span>
      </div>
      <div class="license-item">
        <span class="license-name">授权域名</span>
        <span class="license-value">{{ license.host }}</span>
      </div>
      <div class="license-item">
        <span class="license-name">过期时间</span>
        <span class="license-value">{{ formatTime(license.expiredAt) }}</span>
      </div>
      <div class="license-item">
        <span class="license-name">用户上限</span>
        <span class="license-value">{{ license.userMaxLimit }} 人</span>
      </div>
      <div class="license-item">
        <span class="license-name">kindle端</span>
        <span class="license-value"
          >{{ formatTime(license.simpleWebExpiredAt) }} 过期</span
        >
        <span class="span-btn" @click="supplyLicense">申请7天试用</span>
      </div>
    </div>
  </el-dialog>
</template>

<script>
import { mapGetters } from "vuex";
import Axios from "../plugins/axios";

const licenseApi = "https://r.htmake.com";

export default {
  model: {
    prop: "show",
    event: "setShow"
  },
  name: "License",
  data() {
    return {
      license: {
        host: "",
        userMaxLimit: 50,
        expiredAt: 0,
        openApi: false,
        simpleWebExpiredAt: new Date().getTime()
      }
    };
  },
  props: ["show"],
  computed: {
    ...mapGetters(["dialogWidth", "dialogTop", "dialogContentHeight"])
  },
  watch: {
    show(isVisible) {
      if (isVisible) {
        this.getLicense();
      }
    }
  },
  methods: {
    cancel() {
      this.$emit("setShow", false);
    },
    formatTime(val) {
      if (!val) return "永久有效";
      return new Date(val).format("yy-MM-dd hh:mm");
    },
    getLicense() {
      Axios.get(this.api + "/getLicense").then(
        res => {
          if (res.data.isSuccess) {
            this.license = res.data.data.license;
          }
        },
        error => {
          this.$message.error(
            "获取授权失败 " + (error && error.toString())
          );
        }
      );
    },
    async importLicense() {
      const res = await this.$prompt("请输入密钥", "更新密钥", {
        inputValue: "",
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        inputValidator(v) {
          if (!v) {
            return "密钥不能为空";
          }
          return true;
        }
      }).catch(() => {
        return false;
      });
      if (!res) {
        return;
      }
      return this.comfirmImport(res.value);
    },
    async comfirmImport(key) {
      const res = await this.$confirm("确认要更新密钥吗?", "提示", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning"
      }).catch(() => {
        return false;
      });
      if (!res) {
        return;
      }
      Axios.post(this.api + "/importLicense", {
        content: key
      }).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("更新密钥成功");
            this.license = res.data.data.license;
          }
        },
        error => {
          this.$message.error(
            "更新密钥失败 " + (error && error.toString())
          );
        }
      );
    },
    async supplyLicense() {
      const res = await this.$prompt(
        "请输入邮箱进行验证，每个邮箱仅限试用一次，有效期7天",
        "验证邮箱",
        {
          inputValue: "",
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          inputValidator(v) {
            if (!v) {
              return "邮箱不能为空";
            }
            return true;
          }
        }
      ).catch(() => {
        return false;
      });
      if (!res) {
        return;
      }
      const email = res.value;
      if (
        !email.match(
          /@(163|126|qq|yahoo|sina|sohu|yeah|139|189|21cn|outlook|gmail|icloud).com$/
        )
      ) {
        this.$message.error(
          "仅支持163|126|qq|yahoo|sina|sohu|yeah|139|189|21cn|outlook|gmail|icloud等邮箱"
        );
        return;
      }
      Axios.post(this.api + "/sendCodeToEmail", {
        email
      }).then(
        res => {
          if (res.data.isSuccess) {
            this.showVerifyCodePrompt(email);
          }
        },
        error => {
          this.$message.error(
            "发送验证码失败 " + (error && error.toString())
          );
        }
      );
    },
    async showVerifyCodePrompt(email) {
      const res = await this.$prompt("请输入验证码", "验证邮箱", {
        inputValue: "",
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        inputValidator(v) {
          if (!v) {
            return "验证码不能为空";
          }
          return true;
        }
      }).catch(() => {
        return false;
      });
      if (!res) {
        return;
      }
      Axios.post(this.api + "/supplyLicense", {
        email,
        code: res.value
      }).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("申请试用密钥成功，有效期7天，请谨慎更新");
            this.license = res.data.data.license;
          }
        },
        error => {
          this.$message.error(
            "申请试用失败 " + (error && error.toString())
          );
        }
      );
    },
    async supplyLicense() {
      const emailRes = await this.$prompt(
        "请输入邮箱进行验证，每个邮箱仅限试用一次，有效期7天",
        "验证邮箱",
        {
          inputValue: "",
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          inputValidator: val => !!val || "邮箱不能为空"
        }
      ).catch(() => false);
      if (!emailRes) return;
      const email = emailRes.value;
      if (
        !email.match(
          /@(163|126|qq|yahoo|sina|sohu|yeah|139|189|21cn|outlook|gmail|icloud).com$/
        )
      ) {
        this.$message.error(
          "仅支持163|126|qq|yahoo|sina|sohu|yeah|139|189|21cn|outlook|gmail|icloud等邮箱"
        );
        return;
      }
      Axios.post(licenseApi + "/reader3/sendCodeToEmail", { email }).then(
        res => {
          if (res.data.isSuccess) {
            this.showVerifyCodePrompt(email);
          }
        },
        error => {
          this.$message.error(
            "发送验证码失败 " + (error && error.toString())
          );
        }
      );
    },
    async showVerifyCodePrompt(email) {
      const codeRes = await this.$prompt("请输入验证码", "验证邮箱", {
        inputValue: "",
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        inputValidator: val => !!val || "验证码不能为空"
      }).catch(() => false);
      if (!codeRes) return;
      Axios.post(
        licenseApi + "/reader3/supplyLicense",
        { email, code: codeRes.value },
        { alert: false }
      ).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("申请试用密钥成功，有效期7天，请谨慎更新");
            this.comfirmImport(res.data.data.key);
          }
        },
        error => {
          this.$message.error(
            "申请试用失败 " + (error && error.toString())
          );
        }
      );
    },
    async importLicense() {
      const res = await this.$prompt("请输入密钥", "更新密钥", {
        inputValue: "",
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        inputValidator: val => !!val || "密钥不能为空"
      }).catch(() => false);
      if (!res) return;
      return this.comfirmImport(res.value);
    },
    async comfirmImport(content) {
      const res = await this.$confirm("确认要更新密钥吗?", "提示", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning"
      }).catch(() => false);
      if (!res) return;
      Axios.post(this.api + "/importLicense", { content }).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("更新密钥成功");
            this.license = res.data.data.license;
          }
        },
        error => {
          this.$message.error(
            "更新密钥失败 " + (error && error.toString())
          );
        }
      );
    }
  }
};
</script>
<style lang="stylus" scoped>
.float-right {
  float: right;
}
</style>
