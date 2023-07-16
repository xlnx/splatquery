self.skipWaiting();

self.addEventListener("push", async (event) => {
  const { title, options } = event.data.json();
  event.waitUntil(self.registration.showNotification(title, options))
})

console.log("sw activated")
