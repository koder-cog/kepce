import { build, files, version } from '$service-worker';

const CACHE = `kepce-cache-${version}`;
const ASSETS = [...build, ...files];

self.addEventListener('install', (event) => {
    async function addFilesToCache() {
        const cache = await caches.open(CACHE);
        await cache.addAll(ASSETS);
    }
    event.waitUntil(addFilesToCache());
    self.skipWaiting();
});

self.addEventListener('activate', (event) => {
    async function deleteOldCaches() {
        for (const key of await caches.keys()) {
            if (key !== CACHE) {
                await caches.delete(key);
            }
        }
        await self.clients.claim();
    }
    event.waitUntil(deleteOldCaches());
});

self.addEventListener('fetch', (event) => {
    if (event.request.method !== 'GET') return;

    const url = new URL(event.request.url);
    if (!url.protocol.startsWith('http')) return;

    // API isteklerini veya dinamik backend sorgularını SW önbelleğe almaz
    if (url.pathname.startsWith('/api/') || url.pathname.includes('/api/v1/')) {
        return;
    }

    // 1. Mevcut sürümün build & static dosyaları için: Cache-First (Hızlı & Çevrimdışı hazır)
    if (ASSETS.includes(url.pathname)) {
        event.respondWith(
            caches.open(CACHE).then(async (cache) => {
                const cached = await cache.match(event.request);
                if (cached) return cached;
                const response = await fetch(event.request);
                if (response.status === 200) {
                    cache.put(event.request, response.clone());
                }
                return response;
            })
        );
        return;
    }

    // 2. HTML ve Sayfa İstekleri için: Network-First (Her zaman en taze menü/HTML)
    async function networkFirst() {
        const cache = await caches.open(CACHE);
        try {
            const response = await fetch(event.request);
            if (response.status === 200) {
                cache.put(event.request, response.clone()).catch(() => {});
            }
            return response;
        } catch (err) {
            const cached = await cache.match(event.request);
            if (cached) return cached;
            throw err;
        }
    }

    event.respondWith(networkFirst());
});
