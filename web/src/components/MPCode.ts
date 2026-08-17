import { mapGetters } from "vuex";

export const MPCode = {
  name: "ReplaceRuleForm",
  template: `
  <el-dialog
    title="关注公众号【假装大佬】"
    :visible.sync="show"
    :width="dialogSmallWidth"
    :top="dialogTop"
    :fullscreen="$store.state.miniInterface"
    :class="
      isWebApp && !$store.getters.isNight ? 'status-bar-light-bg-dialog' : ''
    "
    :before-close="cancel"
  >
    <el-image
      :src="require('../assets/imgs/mpcode.jpg')"
      class="qrcode-img"
      fit="cover"
      lazy
    />
  </el-dialog>
`,
  model: {
    prop: "show",
    event: "setShow"
  },
  data() {
    return {};
  },
  props: ["show"],
  computed: {
    ...mapGetters(["dialogSmallWidth", "dialogTop"])
  },
  methods: {
    cancel() {
      this.$emit("setShow", false);
    }
  }
};

export const style = `
.qrcode-img {
  display: block;
  margin: 0 auto;
}
`;
