import { build, files, version } from '$service-worker';

const CACHE = `kepce-cache-${version}`;

self.addEventListener('install', (event) => {
    // Kurulum esnasında ağır indirme yapıp ilk sayfa açılışını engellemiyoruz (Network-First)
    self.skipWaiting();
});

self.addEventListener('activate', (event) => {
    async function deleteOldCaches() {
        for (const key of await caches.keys()) {
            if (key !== CACHE) await caches.delete(key);
        }
        await self.clients.claim();
    }
    event.waitUntil(deleteOldCaches());
});

self.addEventListener('fetch', (event) => {
    if (event.request.method !== 'GET') return;

    // Chrome extension veya non-http(s) isteklerini es geç
    const url = new URL(event.request.url);
    if (!url.protocol.startsWith('http')) return;

    // API isteklerini veya dinamik backend sorgularını önbelleğe alma
    if (url.pathname.startsWith('/api/') || url.pathname.includes('/api/v1/')) {
        return;
    }

    async function networkFirstWithBackgroundCache() {
        const cache = await caches.open(CACHE);

        try {
            // 1. Önce doğrudan ağdan (network) taze veriyi çek
            const response = await fetch(event.request);

            if (response.status === 200) {
                // 2. Yanıt başarılıysa arka planda önbelleği sessizce güncelle (non-blocking)
                const responseToCache = response.clone();
                cache.put(event.request, responseToCache).catch((err) => {
                    console.warn('Arka plan cache güncelleme uyarısı:', err);
                });
            }

            return response;
        } catch (err) {
            // 3. Ağ kopuksa veya istek başarısızsa önbellekteki son veriye düş (fallback)
            const cachedResponse = await cache.match(event.request);
            if (cachedResponse) {
                return cachedResponse;
            }
            throw err;
        }
    }

    event.respondWith(networkFirstWithBackgroundCache());
});
