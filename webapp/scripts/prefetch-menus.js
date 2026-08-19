/**
 * Pre-build: Üretim API'sinden bugünün menü verisini çeker
 * ve `static/data/prerender-menus.json` dosyasına yazar.
 *
 * Bu script `npm run build` öncesinde çalıştırılmalıdır.
 * Dockerfile'daki build adımına eklenir.
 *
 * Kullanım: node scripts/prefetch-menus.js
 */
import { writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));

const API_ORIGIN = process.env.PRERENDER_API_ORIGIN || 'https://kepce.org';
const DEFAULT_CITY = 'istanbul';
const OUTPUT_DIR = resolve(__dirname, '../static/data');
const OUTPUT_FILE = resolve(OUTPUT_DIR, 'prerender-menus.json');

async function main() {
    const today = new Date();
    const dateStr = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`;
    const url = `${API_ORIGIN}/api/v1/menus?city=${DEFAULT_CITY}&date=${dateStr}`;

    console.log(`[prefetch] ${url}`);

    try {
        const res = await fetch(url, {
            headers: { 'Accept': 'application/json' },
            signal: AbortSignal.timeout(10000)
        });

        if (!res.ok) {
            console.warn(`[prefetch] API ${res.status} döndü. Boş JSON yazılıyor.`);
            writeEmptyPayload(dateStr);
            return;
        }

        const data = await res.json();
        const menus = Array.isArray(data)
            ? data
            : (data?.menus ?? data?.results ?? data?.data ?? []);

        const payload = {
            city: DEFAULT_CITY,
            date: dateStr,
            fetchedAt: new Date().toISOString(),
            menus
        };

        if (!existsSync(OUTPUT_DIR)) {
            mkdirSync(OUTPUT_DIR, { recursive: true });
        }

        writeFileSync(OUTPUT_FILE, JSON.stringify(payload), 'utf-8');
        console.log(`[prefetch] [BAŞARI] ${menus.length} menü → ${OUTPUT_FILE}`);
    } catch (err) {
        console.warn(`[prefetch] API erişilemedi: ${err.message}. Boş JSON yazılıyor.`);
        writeEmptyPayload(dateStr);
    }
}

function writeEmptyPayload(dateStr) {
    const payload = {
        city: DEFAULT_CITY,
        date: dateStr,
        fetchedAt: new Date().toISOString(),
        menus: []
    };

    if (!existsSync(OUTPUT_DIR)) {
        mkdirSync(OUTPUT_DIR, { recursive: true });
    }

    writeFileSync(OUTPUT_FILE, JSON.stringify(payload), 'utf-8');
}

main();
