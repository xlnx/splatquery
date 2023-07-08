// import { clientsClaim } from "workbox-core";
// import { cleanupOutdatedCaches, precacheAndRoute } from "workbox-precaching";

// self.skipWaiting();
// clientsClaim();

// cleanupOutdatedCaches();
// precacheAndRoute(self.__WB_MANIFEST);

self.addEventListener("push", async (event) => {
  event.waitUntil(self.registration.showNotification(
    "title",
    {
      body: "body"
    }
  ))
})

console.log("sw activated")
