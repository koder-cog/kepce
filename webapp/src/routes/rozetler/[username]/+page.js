import { error } from '@sveltejs/kit';

export function load() {
    error(501, 'Rozet ve başarım sistemi henüz kullanıma açılmadı.');
}
