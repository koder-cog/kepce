import { browser } from '$app/environment';

/**
 * Build-time menü verisi yükleyici.
 *
 * Pre-build adımında `scripts/prefetch-menus.js` üretim API'sinden
 * bugünün menüsünü çekip `static/data/prerender-menus.json` dosyasına yazar.
 * Bu load() fonksiyonu o dosyayı okur ve SSR HTML'ine menü kartlarını gömer.
 *
 * Tarayıcıda (client-side navigation) veri çekmez;
 * timeline store + onMount zaten bunu halleder.
 */

export const prerender = true;

export async function load({ fetch }) {
    // SSR / Prerender ve Client Hydration: statik JSON dosyasından veri aktarımı
    try {
        const res = await fetch('/data/prerender-menus.json');
        if (!res.ok) {
            return { prerenderedMenus: null, prerenderedCity: null, prerenderedDate: null };
        }

        const payload = await res.json();
        const menus = payload?.menus;

        if (Array.isArray(menus) && menus.length > 0) {
            console.log(`[prerender] ${payload.city} ${payload.date}: ${menus.length} menü yüklendi.`);
            return {
                prerenderedMenus: menus,
                prerenderedCity: payload.city,
                prerenderedDate: payload.date
            };
        }

        return { prerenderedMenus: null, prerenderedCity: payload?.city, prerenderedDate: payload?.date };
    } catch (err) {
        // Dosya yoksa sessizce boş dön (pre-build adımı çalışmamış olabilir)
        console.warn(`[prerender] prerender-menus.json okunamadı: ${err.message}`);
        return { prerenderedMenus: null, prerenderedCity: null, prerenderedDate: null };
    }
}

