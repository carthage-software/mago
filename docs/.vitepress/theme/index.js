import DefaultTheme from "vitepress/theme";
import "./custom.css";
import MagoPlayground from "./components/playground/MagoPlayground.vue";

export default {
  ...DefaultTheme,
  enhanceApp({ app }) {
    app.component("MagoPlayground", MagoPlayground);
  },
};
