// import { clientsClaim } from "workbox-core";
// import { cleanupOutdatedCaches, precacheAndRoute } from "workbox-precaching";

self.skipWaiting();
// clientsClaim();

// cleanupOutdatedCaches();
// precacheAndRoute(self.__WB_MANIFEST);

self.addEventListener("push", async (event) => {
  const { title, options } = event.data.json();
  event.waitUntil(self.registration.showNotification(title, options))
})

console.log("sw activated")
