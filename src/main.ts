import { createApp } from "vue";
import App from "./App.vue";
import 'splitpanes/dist/splitpanes.css'

// 确保这行在创建 Vue 应用之前
import '@tauri-apps/api/tauri'

createApp(App).mount("#app");
