<script>
  import { globalState } from '$lib/stores/appState.svelte.js';
  import AnnouncementBanner from './AnnouncementBanner.svelte';
  import { api } from '$lib/api/index.js';
  import { showToast } from './toast.js';

  let isResending = $state(false);

  async function handleResend() {
    if (isResending) return;
    const inputEmail =
      typeof window !== 'undefined'
        ? window.prompt(
            'Sıfır bilgi mimarisi gereği e-posta adresiniz sunucuda saklanmaz. Doğrulama bağlantısı için lütfen kayıtlı e-posta adresinizi girin:'
          )
        : null;
    if (!inputEmail || !inputEmail.trim()) return;

    isResending = true;
    try {
      await api.resendVerification({ email: inputEmail.trim() });
      showToast('Doğrulama bağlantısı e-posta adresinize gönderildi.', { type: 'success' });
    } catch (e) {
      if (e.status === 429) {
        showToast('Lütfen yeni bir e-posta istemeden önce 24 saat bekleyiniz.', { type: 'error' });
      } else {
        showToast(e.message || 'E-posta gönderilirken bir hata oluştu.', { type: 'error' });
      }
    } finally {
      isResending = false;
    }
  }
</script>

{#if globalState.user && !globalState.user.is_verified}
  <AnnouncementBanner
    id={`unverified-banner-${globalState.user.id}`}
    text="Hesabınızın özelliklerini tam olarak kullanabilmek için e-postanızı onaylamalısınız."
    ctaText={isResending ? 'Gönderiliyor...' : 'Tekrar Gönder'}
    theme="accent-disclaimer"
    onCtaClick={handleResend}
  />
{/if}
