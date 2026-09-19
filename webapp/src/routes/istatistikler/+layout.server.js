import { error } from '@sveltejs/kit';

/**
 * İstatistikler sayfası ve alt rotaları lansman sonrasına ertelendiği için
 * kod tabanından hiçbir şey silinmeden geçici olarak 404 döndürülür.
 */
export function load() {
  error(404, 'Bu sayfa henüz kullanıma açılmadı.');
}
