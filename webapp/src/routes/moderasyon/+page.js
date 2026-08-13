import { redirect } from '@sveltejs/kit';

export function load() {
  throw redirect(302, '/moderasyon/mutfak/yemekler');
}
