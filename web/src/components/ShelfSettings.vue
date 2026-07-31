<template>
  <el-dialog
    title="书架布局"
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
    <el-form :model="shelfConfig">
      <el-form-item label="显示分组">
        <el-select
          v-model="shelfConfig.showBookGroup"
          size="mini"
          class="setting-select"
          filterable
          placeholder="请选择默认显示分组"
        >
          <el-option
            v-for="(item, index) in bookGroupDisplayList"
            :key="'book-group-' + index"
            :label="item.groupName"
            :value="item.groupId"
          />
        </el-select>
      </el-form-item>
      <el-form-item label="显示视图">
        <el-select v-model="shelfConfig.viewCate" size="mini" class="setting-select" filterable>
          <el-option v-for="item in viewCateList" :key="item.value" :label="item.name" :value="item.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="书籍排序">
        <el-select v-model="shelfConfig.bookOrder" size="mini" class="setting-select" filterable>
          <el-option v-for="item in bookOrderList" :key="item.value" :label="item.name" :value="item.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="图片代理">
        <el-select v-model="shelfConfig.imageProxy" size="mini" class="setting-select" filterable>
          <el-option v-for="item in imageProxyList" :key="item.value" :label="item.name" :value="item.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="列表优化">
        <el-select v-model="shelfConfig.virtualOptimize" size="mini" class="setting-select" filterable>
          <el-option v-for="item in virtualOptimizeList" :key="item.value" :label="item.name" :value="item.value" />
        </el-select>
      </el-form-item>
    </el-form>
    <div slot="footer" class="dialog-footer">
      <el-button size="medium" type="primary" @click="save">保 存</el-button>
      <el-button size="medium" @click="cancel">关 闭</el-button>
    </div>
  </el-dialog>
</template>

<script>
import { mapGetters } from "vuex";

export default {
  model: {
    prop: "show",
    event: "setShow"
  },
  name: "ShelfSettings",
  props: ["show"],
  data() {
    return {
      shelfConfig: { ...this.$store.state.shelfConfig },
      viewCateList: [
        { name: "列表", value: "list" },
        { name: "网格3列", value: "column-3" },
        { name: "网格4列", value: "column-4" },
        { name: "网格5列", value: "column-5" },
        { name: "网格6列", value: "column-6" },
        { name: "网格(小)", value: "column-100" },
        { name: "网格(中)", value: "column-150" },
        { name: "网格(大)", value: "column-200" }
      ],
      bookOrderList: [
        { name: "按阅读时间", value: "durChapterTime" },
        { name: "按更新时间", value: "lastCheckTime" },
        { name: "按书名", value: "name" }
      ],
      imageProxyList: [
        { name: "不使用代理", value: "noProxy" },
        { name: "服务器代理", value: "serverProxy" }
      ],
      virtualOptimizeList: [
        { name: "启用虚拟列表优化", value: "yes" },
        { name: "不启用优化", value: "no" }
      ]
    };
  },
  computed: {
    ...mapGetters(["dialogSmallWidth", "dialogTop"]),
    bookGroupDisplayList() {
      return this.$store.state.bookGroupList.filter(item => item.show);
    }
  },
  watch: {
    show(value) {
      if (value) {
        this.shelfConfig = { ...this.$store.state.shelfConfig };
      }
    }
  },
  methods: {
    cancel() {
      this.$emit("setShow", false);
    },
    save() {
      this.$store.commit("setShelfConfig", { ...this.shelfConfig });
      this.cancel();
    }
  }
};
</script>
