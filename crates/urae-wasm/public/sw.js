// Service Worker for URAE — Atomic Cache-First with Background Network Revalidation
const CACHE_NAME = 'urae-algebra-engine-cache-v2';

// Core assets to pre-cache on install to guarantee complete atomic offline capability
const PRECACHE_ASSETS = [
  './',
  './index.html',
  './manifest.json',
  './favicon.ico',
  './pkg/urae_wasm.js',
  './pkg/urae_wasm_bg.wasm'
];

// 1. Pre-cache all matching assets on install and activate immediately
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      return cache.addAll(PRECACHE_ASSETS);
    }).then(() => self.skipWaiting())
  );
});

// 2. Purge all legacy caches on activation and claim clients
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(
        keys.filter((key) => key !== CACHE_NAME).map((key) => caches.delete(key))
      )
    ).then(() => self.clients.claim())
  );
});

// 3. Fetch Router: Atomic Cache-First Execution
self.addEventListener('fetch', (event) => {
  // Only handle local same-origin GET requests
  if (event.request.method !== 'GET' || !event.request.url.startsWith(self.location.origin)) {
    return;
  }

  const url = new URL(event.request.url);

  // Bypass third-party analytics or beacon endpoints
  if (url.hostname.includes('cloudflareinsights.com') || url.hostname.includes('google-analytics.com')) {
    return;
  }

  // Bypass Edge API endpoints (/api/*)
  if (url.pathname.startsWith('/api/')) {
    return;
  }

  // Never cache the service worker itself so browser can check for updates in the background
  if (url.pathname.endsWith('/sw.js')) {
    event.respondWith(fetch(event.request));
    return;
  }

  // Atomic Cache-First Strategy:
  // Instantly serve from the active cache bucket so all assets in a session (JS, WASM, HTML)
  // are guaranteed to share the exact same build version without ABI skew.
  event.respondWith(
    caches.open(CACHE_NAME).then((cache) => {
      return cache.match(event.request).then((cachedResponse) => {
        if (cachedResponse) {
          return cachedResponse;
        }

        // Cache miss: fetch from network and populate active cache
        return fetch(event.request).then((networkResponse) => {
          if (networkResponse && networkResponse.status === 200) {
            cache.put(event.request, networkResponse.clone());
          }
          return networkResponse;
        }).catch((err) => {
          if (event.request.mode === 'navigate') {
            return cache.match('./index.html') || cache.match('./');
          }
          throw err;
        });
      });
    })
  );
});
