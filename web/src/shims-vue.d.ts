// Vue 模块与静态资源声明（转录 .ts 编译所需）—— 纯 ambient 声明，勿加 import/export
declare module "*.vue" {
  import Vue from "vue";
  export default Vue;
}

declare module "*.png" {
  const src: string;
  export default src;
}

declare module "*.jpeg" {
  const src: string;
  export default src;
}

declare module "*.jpg" {
  const src: string;
  export default src;
}

declare module "*.gif" {
  const src: string;
  export default src;
}

declare module "*.svg" {
  const src: string;
  export default src;
}

declare module "*.css";
declare module "*.scss";
declare module "*.less";
declare module "*.ttf";
declare module "*.woff";
declare module "*.woff2";
declare module "*.styl";

declare module "vue-lazyload";
declare module "element-ui/packages/image/src/image-viewer.vue";

interface Window {
  reader?: any;
  customCSSLoad?: any;
  loadLink?: any;
  shadowIframe?: any;
  simplized?: any;
  traditionalized?: any;
  reader_notify?: any;
  getQueryString(queryName: string): string | null;
  [key: string]: any;
}
