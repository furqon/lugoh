/// <reference lib="WebWorker" />
/* global self, caches, fetch, Response */

const CACHE_NAME = 'agos-v1'
const ASSETS_TO_CACHE = [
  '/',
  '/index.html',
  '/offline.html',
  '/manifest.json',
  '/favicon.svg',
]

// Install event: pre-cache the app shell (including offline page)
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      return cache.addAll(ASSETS_TO_CACHE)
    })
  )
  self.skipWaiting()
})

// Activate event: clean up old caches
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames
          .filter((name) => name !== CACHE_NAME)
          .map((name) => caches.delete(name))
      )
    })
  )
  self.clients.claim()
})

// Fetch event: route requests to the appropriate strategy
self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url)

  // For API calls, use network-first strategy
  if (url.pathname.startsWith('/api/')) {
    event.respondWith(networkFirst(event.request))
    return
  }

  // For navigation requests (HTML pages), use network-first with offline fallback.
  // This ensures users always see a fresh version of the app when online,
  // and see the offline page when the network is unavailable.
  if (event.request.mode === 'navigate') {
    event.respondWith(networkFirstWithOfflineFallback(event.request))
    return
  }

  // For all other requests (CSS, JS, fonts, images), use cache-first
  event.respondWith(cacheFirst(event.request))
})

/**
 * Cache-first strategy: serve from cache, fall back to network.
 * Used for static assets (CSS, JS, fonts, images) where freshness
 * is less critical and speed matters most.
 */
async function cacheFirst(request) {
  const cached = await caches.match(request)
  if (cached) {
    return cached
  }
  try {
    const response = await fetch(request)
    if (response.ok) {
      const cache = await caches.open(CACHE_NAME)
      cache.put(request, response.clone())
    }
    return response
  } catch (_error) {
    return new Response('Offline', { status: 503 })
  }
}

/**
 * Network-first strategy: try network first, fall back to cache.
 * Used for API calls where fresh data is preferred.
 */
async function networkFirst(request) {
  try {
    const response = await fetch(request)
    if (response.ok) {
      const cache = await caches.open(CACHE_NAME)
      cache.put(request, response.clone())
    }
    return response
  } catch (_error) {
    const cached = await caches.match(request)
    if (cached) {
      return cached
    }
    return new Response(
      JSON.stringify({ success: false, error: 'You are offline. Analysis is unavailable.' }),
      { status: 503, headers: { 'Content-Type': 'application/json' } }
    )
  }
}

/**
 * Network-first for navigation requests with offline fallback page.
 * Tries to fetch the latest HTML from the network first. If that fails
 * (offline), serves the pre-cached offline.html page.
 */
async function networkFirstWithOfflineFallback(request) {
  try {
    const response = await fetch(request)
    if (response.ok) {
      const cache = await caches.open(CACHE_NAME)
      cache.put(request, response.clone())
    }
    return response
  } catch (_error) {
    const cached = await caches.match('/offline.html')
    if (cached) {
      return cached
    }
    // If even the offline page isn't cached (fresh install), return a minimal response
    return new Response(
      `<!doctype html><html><body><h1>Offline</h1><p>AGOS is unavailable without a network connection.</p></body></html>`,
      { status: 503, headers: { 'Content-Type': 'text/html; charset=utf-8' } }
    )
  }
}
