import DefaultTheme from "vitepress/theme";
import "./custom.css";
import MagoPlayground from "./components/playground/MagoPlayground.vue";
import BenchmarkChart from "./components/BenchmarkChart.vue";

export default {
  ...DefaultTheme,
  enhanceApp({ app }) {
    app.component("MagoPlayground", MagoPlayground);
    app.component("BenchmarkChart", BenchmarkChart);
  },
};
