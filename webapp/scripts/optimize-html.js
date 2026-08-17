/**
 * optimize-html.js — Post-build Critical CSS İnlining
 *
 * Google web.dev reçetesine göre:
 * 1. critters ile HTML'deki elementlere karşılık gelen CSS'i bulur
 * 2. Critical CSS'i <style> bloğuna inline eder
 * 3. <link rel="stylesheet"> → media="print" onload="this.media='all'" dönüşümü
 * 4. <noscript> fallback ekler
 *
 * Kaynak: https://web.dev/articles/defer-non-critical-css
 *         https://developer.chrome.com/docs/aurora/aurora-resource-inlining
 */

import Critters from 'critters';
import { readdir, readFile, writeFile, stat } from 'node:fs/promises';
import { join } from 'node:path';

const DIST_DIR = join(import.meta.dirname, '..', 'dist');

/**
 * dist/ altındaki tüm .html dosyalarını recursive olarak bulur.
 */
async function findHtmlFiles(dir) {
	const entries = await readdir(dir, { withFileTypes: true });
	const files = [];

	for (const entry of entries) {
		const fullPath = join(dir, entry.name);
		if (entry.isDirectory()) {
			files.push(...(await findHtmlFiles(fullPath)));
		} else if (entry.name.endsWith('.html')) {
			files.push(fullPath);
		}
	}

	return files;
}

async function main() {
	const critters = new Critters({
		path: DIST_DIR,
		// HTML'deki seçicilere karşılık gelen CSS'i inline et
		// Geriye kalanı async yükle (media="print" onload pattern)
		preload: 'media',
		// Inline edilen CSS'i minify et
		compress: true,
		// Font-face kurallarını da inline et (FCP için kritik)
		inlineFonts: true,
		// Kullanılmayan CSS kurallarını çıkar
		pruneSource: false,
		// noscript fallback ekle
		noscriptFallback: true,
	});

	const htmlFiles = await findHtmlFiles(DIST_DIR);

	console.log(`🔍 ${htmlFiles.length} HTML dosyası bulundu.`);

	for (const file of htmlFiles) {
		const relPath = file.replace(DIST_DIR + '/', '');
		const original = await readFile(file, 'utf-8');

		// Stylesheet linki olmayan HTML'leri atla (oauth callback vb.)
		if (!original.includes('rel="stylesheet"')) {
			console.log(`  ⏭️  ${relPath}: stylesheet yok, atlandı.`);
			continue;
		}

		try {
			const optimized = await critters.process(original);
			await writeFile(file, optimized, 'utf-8');

			const originalSize = Buffer.byteLength(original, 'utf-8');
			const optimizedSize = Buffer.byteLength(optimized, 'utf-8');
			const diff = optimizedSize - originalSize;

			console.log(
				`  ✅ ${relPath}: ${(originalSize / 1024).toFixed(1)}KB → ${(optimizedSize / 1024).toFixed(1)}KB (critical CSS: +${(diff / 1024).toFixed(1)}KB)`
			);
		} catch (err) {
			console.warn(`  ⚠️  ${relPath}: critters hatası, orijinal korundu. (${err.message})`);
		}
	}

	console.log(`\n🎉 Critical CSS inlining tamamlandı.`);
}

main().catch((err) => {
	console.error('❌ Critical CSS inlining hatası:', err);
	process.exit(1);
});
