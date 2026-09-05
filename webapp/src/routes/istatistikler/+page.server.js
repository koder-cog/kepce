import { redirect } from '@sveltejs/kit';

/**
 * İstatistikler ana sayfası doğrudan en çok ziyaret edilen yemekler sekmesine
 * 301 kalıcı yönlendirmeyle aktarılır (arama motorları ve kullanıcılar için).
 */
export function load() {
	redirect(301, '/istatistikler/yemekler');
}
