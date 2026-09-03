// Service Worker for URAE — Atomic Serverless Deployment Caching Strategy
const CACHE_NAME = 'urae-algebra-engine-cache-v2';

// Static assets to pre-cache on install
const PRECACHE_ASSETS = [
  './',
  './index.html',
  './manifest.json'
];

// Pre-cache on install and activate immediately
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(PRECACHE_ASSETS))
  );
  self.skipWaiting();
});

// Purge all legacy caches on activation
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(
        keys.filter((key) => key !== CACHE_NAME).map((key) => caches.delete(key))
      )
    ).then(() => self.clients.claim())
  );
});

// Fetch router tailored for atomic immutable serverless deployments
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

  const isNavigation = event.request.mode === 'navigate' || event.request.destination === 'document' || url.pathname.endsWith('.html') || url.pathname === '/';
  const isCodeAsset = url.pathname.includes('/pkg/') || url.pathname.endsWith('.wasm') || url.pathname.endsWith('.js');

  if (isNavigation || isCodeAsset) {
    // Network-First for HTML, WASM binaries, and JS wrappers:
    // Always fetch newest deployment assets from edge CDN when online so JS and WASM are always in lockstep;
    // fallback to cache when offline.
    event.respondWith(
      fetch(event.request)
        .then((networkResponse) => {
          if (networkResponse && networkResponse.status === 200) {
            const copy = networkResponse.clone();
            caches.open(CACHE_NAME).then((cache) => cache.put(event.request, copy));
          }
          return networkResponse;
        })
        .catch(() => caches.match(event.request).then((cached) => {
          if (cached) return cached;
          if (isNavigation) return caches.match('./index.html') || caches.match('./');
          return Promise.reject('Resource offline');
        }))
    );
  } else {
    // Cache-First for static media, icons, and fonts
    event.respondWith(
      caches.match(event.request).then((cachedResponse) => {
        if (cachedResponse) {
          return cachedResponse;
        }
        return fetch(event.request)
          .then((networkResponse) => {
            if (networkResponse && networkResponse.status === 200) {
              const copy = networkResponse.clone();
              caches.open(CACHE_NAME).then((cache) => cache.put(event.request, copy));
            }
            return networkResponse;
          })
          .catch((err) => {
            console.warn('SW fetch failed for asset:', event.request.url, err);
            throw err;
          });
      })
    );
  }
});
