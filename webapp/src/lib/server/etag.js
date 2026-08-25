import { createHash } from 'node:crypto';

/**
 * Sitemap (ve benzeri XML) yanıtları için ETag/304 desteği.
 *
 * Gövdeden SHA-256 üretip ETag header'ı set eder; isteğin If-None-Match'i
 * eşleşirse 304 Not Modified döner (gövdesiz) - botlar bayat parçaları
 * yeniden indirmek zorunda kalmaz, sunucu band genişliği tasarruf eder.
 *
 * @param {Request} request
 * @param {string} body  XML gövdesi
 * @param {Record<string, string>} headers  Content-Type / Cache-Control vb.
 * @returns {Response} 200 (gövdeli) veya 304 (gövdesiz)
 */
export function withEtag(request, body, headers) {
	const hash = createHash('sha256').update(body).digest('hex');
	const etag = `"${hash}"`;

	const allHeaders = { ...headers, ETag: etag };

	const ifNoneMatch = request.headers.get('if-none-match');
	if (ifNoneMatch) {
		const cleanMatch = ifNoneMatch
			.trim()
			.replace(/^W\//, '')
			.replace(/"/g, '')
			.replace(/-gzip$/, '');
		if (cleanMatch === hash || ifNoneMatch.trim() === '*') {
			return new Response(null, { status: 304, headers: allHeaders });
		}
	}

	return new Response(body, { status: 200, headers: allHeaders });
}
