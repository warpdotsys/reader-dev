import { mapGetters } from "vuex";
import Axios from "../plugins/axios";

export const BookConfig = {
  name: "BookConfig",
  template: `
  <el-dialog
    title="书籍设置"
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
    <el-form :model="bookConfig">
      <el-form-item label="PDF图片宽度">
        <el-select
          v-model="bookConfig.pdfImageWidth"
          size="mini"
          class="setting-select"
          filterable
          placeholder="请选择PDF图片宽度"
        >
          <el-option
            v-for="(item, index) in pdfImageWidthList"
            :key="'pdf-image-width-' + index"
            :label="item.name"
            :value="item.value"
          >
          </el-option>
        </el-select>
      </el-form-item>
    </el-form>
    <div slot="footer" class="dialog-footer">
      <el-button size="medium" type="primary" @click="save">保 存</el-button>
      <el-button size="medium" type="primary" @click="cancel">关 闭</el-button>
    </div>
  </el-dialog>
`,
  model: {
    prop: "show",
    event: "setShow"
  },
  data() {
    return {
      isAdd: true,
      bookConfig: {
        pdfImageWidth:
          (this.$store.state.showBookInfo.readConfig || {}).pdfImageWidth || 800
      },
      pdfImageWidthList: [
        { name: "750px", value: 750 },
        { name: "800px", value: 800 },
        { name: "850px", value: 850 },
        { name: "900px", value: 900 },
        { name: "950px", value: 950 },
        { name: "1000px", value: 1000 },
        { name: "1100px", value: 1100 },
        { name: "1200px", value: 1200 },
        { name: "1300px", value: 1300 },
        { name: "1400px", value: 1400 },
        { name: "1500px", value: 1500 },
        { name: "1600px", value: 1600 }
      ]
    };
  },
  props: ["show"],
  computed: {
    ...mapGetters(["dialogSmallWidth", "dialogTop"])
  },
  watch: {
    show(isVisible) {
      if (isVisible) {
        this.bookConfig = {
          pdfImageWidth:
            (this.$store.state.showBookInfo.readConfig || {}).pdfImageWidth ||
            800
        };
      }
    }
  },
  methods: {
    cancel() {
      this.$emit("setShow", false);
    },
    save() {
      Axios.post("/book/saveBookConfig", {
        bookUrl: this.$store.state.showBookInfo.bookUrl,
        ...this.bookConfig
      }).then(
        res => {
          if (res.data.isSuccess) {
            this.$message.success("更新成功");
            this.showBookInfo = res.data.data;
            this.$store.commit("updateShelfBook", res.data.data);
          }
        },
        error => {
          this.$message.error("更新失败" + (error && error.toString()));
        }
      );
    }
  }
};
