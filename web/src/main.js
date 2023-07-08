import { createApp } from "vue";
import { createI18n } from "vue-i18n";
import languages from "./assets/i18n";
import router from "./router";
import auth from "./auth";
import App from "./App.vue";

const i18n = createI18n({
  locale: "en-US",
  fallbackLocale: "en-US",
  messages: { ...languages },
});

const app = createApp(App);

app
  .use(router)
  .use(i18n)
  .use(auth)
  .mount("#app");
