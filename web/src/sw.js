import { clientsClaim } from 'workbox-core';
import { registerRoute, Route } from 'workbox-routing';
import { CacheFirst, StaleWhileRevalidate } from 'workbox-strategies';
import { ExpirationPlugin } from 'workbox-expiration';
import { cleanupOutdatedCaches, precacheAndRoute } from "workbox-precaching";

self.skipWaiting();
clientsClaim();

cleanupOutdatedCaches();
precacheAndRoute(self.__WB_MANIFEST);

registerRoute(new Route(({ request }) => request.destination == 'image', new CacheFirst({
  cacheName: 'images',
  plugins: [
    new ExpirationPlugin({
      maxAgeSeconds: 60 * 60 * 24 * 30,
    })
  ]
})));

const isApiUrl = ({ origin, pathname }) => {
  let api = import.meta.env.VITE_API_SERVER;
  if (api.startsWith('/')) {
    api = self.location.origin + api;
  }
  const url = new URL(api);
  return origin == url.origin && pathname.startsWith(url.pathname);
}

registerRoute(new Route(({ request }) => isApiUrl(new URL(request.url)), new StaleWhileRevalidate({
  cacheName: 'api',
  plugins: [
    new ExpirationPlugin({
      maxAgeSeconds: 60 * 60 * 24 * 30,
    })
  ]
})));

self.addEventListener("message", async (event) => {
  const {type, params} = event.data;
  if (type == 'invalidateCache') {
    const {cacheName, url} = params;
    const cache = await self.caches.open(cacheName);
    cache.delete(new URL(url));
  }
});

self.addEventListener("push", async (event) => {
  const { title, options } = event.data.json();
  event.waitUntil(self.registration.showNotification(title, options))
});
