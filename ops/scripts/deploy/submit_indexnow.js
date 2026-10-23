#!/usr/bin/env node
/**
 * Kepçe - IndexNow Toplu Gönderim Aracı
 * =====================================
 * Sitedeki tüm kanonik URL'leri (statik sayfalar, şehir iniş sayfaları ve
 * aylık gün sayfaları) toplayıp api.indexnow.org üzerinden Bing, Yandex ve
 * DuckDuckGo'ya tek komutla bildirir.
 *
 * Kullanım:
 *   node scripts/submit_indexnow.js [--dry-run] [--endpoint=URL]
 */

const INDEXNOW_KEY = process.env.INDEXNOW_KEY || '2a72faa3a595fc8f63e5754ec96aa8d0';
const HOST = process.env.INDEXNOW_HOST || 'kepce.org';
const ENDPOINT = process.env.INDEXNOW_ENDPOINT || 'https://api.indexnow.org/indexnow';
const BASE_URL = `https://${HOST}`;
const KEY_LOCATION = `${BASE_URL}/${INDEXNOW_KEY}.txt`;
const CHUNK_SIZE = 10000;

const isDryRun = process.argv.includes('--dry-run');

async function fetchText(url) {
	const res = await fetch(url, {
		headers: { 'User-Agent': 'Kepce-IndexNow-Tool/1.0' }
	});
	if (!res.ok) {
		throw new Error(`HTTP ${res.status} alindi: ${url}`);
	}
	return await res.text();
}

function extractLocs(xml) {
	const matches = xml.matchAll(/<loc>([^<]+)<\/loc>/g);
	const urls = [];
	for (const m of matches) {
		const u = m[1].trim();
		if (u) urls.push(u);
	}
	return urls;
}

async function collectAllUrls() {
	console.log(`[1/3] Sitemap'ten URL'ler toplaniyor: ${BASE_URL}/sitemap.xml`);
	const xml = await fetchText(`${BASE_URL}/sitemap.xml`);
	const allUrls = new Set();

	if (xml.includes('<sitemapindex')) {
		const sitemaps = extractLocs(xml);
		console.log(`      ${sitemaps.length} adet sitemap parcasi bulundu.`);
		for (const sm of sitemaps) {
			try {
				const subXml = await fetchText(sm);
				const locs = extractLocs(subXml);
				locs.forEach((u) => allUrls.add(u));
				console.log(`      + ${sm}: ${locs.length} URL`);
			} catch (err) {
				console.warn(`      ! Hata: ${sm} okunamadi: ${err.message}`);
			}
		}
	} else {
		const locs = extractLocs(xml);
		locs.forEach((u) => allUrls.add(u));
		console.log(`      ${locs.length} adet tekil sayfa URL'i bulundu.`);
	}

	const sorted = Array.from(allUrls).sort();
	return sorted;
}

async function submitChunk(urls, chunkIndex, totalChunks) {
	const payload = {
		host: HOST,
		key: INDEXNOW_KEY,
		keyLocation: KEY_LOCATION,
		urlList: urls
	};

	if (isDryRun) {
		console.log(`[DRY-RUN] Paket ${chunkIndex + 1}/${totalChunks} (${urls.length} URL) gonderilecekti.`);
		return true;
	}

	const res = await fetch(ENDPOINT, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json; charset=utf-8'
		},
		body: JSON.stringify(payload)
	});

	if (res.status === 200 || res.status === 202) {
		console.log(`[OK] Paket ${chunkIndex + 1}/${totalChunks} (${urls.length} URL) basariyla iletildi (HTTP ${res.status}).`);
		return true;
	} else {
		const text = await res.text().catch(() => '');
		console.error(`[FAIL] Paket ${chunkIndex + 1}/${totalChunks} HTTP ${res.status}: ${text}`);
		return false;
	}
}

async function main() {
	console.log('=== KEPCE INDEXNOW SUBMISSION ===');
	console.log(`Host: ${HOST}`);
	console.log(`Key Location: ${KEY_LOCATION}`);
	console.log(`Endpoint: ${ENDPOINT}`);
	if (isDryRun) console.log('Mod: DRY-RUN (Gercek istek yapilmaz)');
	console.log('');

	try {
		const urls = await collectAllUrls();
		console.log(`\n[2/3] Toplam ${urls.length} tekil kanonik URL hazirlandi.`);

		if (urls.length === 0) {
			console.log('Gonderilecek URL bulunamadi.');
			return;
		}

		console.log(`\n[3/3] IndexNow API'sine gonderiliyor (${ENDPOINT})...`);

		const chunks = [];
		for (let i = 0; i < urls.length; i += CHUNK_SIZE) {
			chunks.push(urls.slice(i, i + CHUNK_SIZE));
		}

		let successCount = 0;
		for (let i = 0; i < chunks.length; i++) {
			const ok = await submitChunk(chunks[i], i, chunks.length);
			if (ok) successCount++;
		}

		console.log(`\nIslem tamamlandi: ${successCount}/${chunks.length} paket basarili.`);
	} catch (err) {
		console.error(`\nKritik hata: ${err.message}`);
		process.exit(1);
	}
}

main();
