import { createPinia } from "pinia";
import { createApp } from "vue";
import App from "./App.vue";
import { isTauri } from "./api/client";
import { i18n } from "./i18n";
import "./style.css";

const root = document.documentElement;
root.dataset.shell = isTauri() ? "native" : "web";

// ?theme=dark|light pins the theme (screenshots); otherwise it follows the system, live
const pinned = new URLSearchParams(location.search).get("theme");
const scheme = matchMedia("(prefers-color-scheme: dark)");
const applyTheme = () => (root.dataset.theme = pinned === "dark" || pinned === "light" ? pinned : scheme.matches ? "dark" : "light");
applyTheme();
scheme.addEventListener("change", applyTheme);

createApp(App).use(createPinia()).use(i18n).mount("#app");
