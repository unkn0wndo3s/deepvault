import { createApp } from "vue";
import { createRouter, createWebHistory } from "vue-router";
import App from "./App.vue";
import DeviceList from "./components/DeviceList.vue";
import ConfigType from "./components/ConfigType.vue";
import SecurityConfig from "./components/SecurityConfig.vue";
import Progress from "./components/Progress.vue";
import Usage from "./components/Usage.vue";
import Logs from "./components/Logs.vue";

const routes = [
  { path: "/", component: DeviceList },
  { path: "/config-type", component: ConfigType },
  { path: "/security", component: SecurityConfig },
  { path: "/progress", component: Progress },
  { path: "/usage", component: Usage },
  { path: "/logs", component: Logs },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

const app = createApp(App);
app.use(router);
app.mount("#app");
