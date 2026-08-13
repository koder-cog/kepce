import { error } from '@sveltejs/kit';
import { LEGAL_VERSIONS } from '$lib/data/legal/versions.js';

export async function load({ params }) {
  const { version } = params;
  
  const vData = LEGAL_VERSIONS["gizlilik-politikasi"].find(v => v.slug === version);
  
  if (!vData || vData.current) {
    // Mevcut sürüm arşivde gösterilmez veya bulunamadı
    throw error(404, 'Arşivlenmiş sürüm bulunamadı');
  }

  // Dinamik komponent yükleme (Svelte komponenti)
  try {
    const module = await import(`../../../../../lib/data/legal/gizlilik-politikasi/${version}.svelte`);
    return {
      versionData: vData,
      component: module.default
    };
  } catch (e) {
    throw error(404, 'Arşiv içeriği yüklenemedi');
  }
}
