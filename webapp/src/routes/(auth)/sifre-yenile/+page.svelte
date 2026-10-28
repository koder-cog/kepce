<script>
  import '@/styles/pages/_auth.css';
  import { goto } from '$app/navigation';
  import { api } from '$lib/api/index.js';
  import { icon } from '$lib/components/ui/icons.js';
  import { showToast } from '$lib/components/ui/toast.js';
  import Seo from '$lib/components/ui/Seo.svelte';

  let nickname = $state('');
  let recoveryKey = $state('');
  let newPassword = $state('');
  let repeatPassword = $state('');
  let showPassword = $state(false);
  let showRepeatPassword = $state(false);

  let isLoading = $state(false);
  let errorMsg = $state('');
  let formError = $state(false);
  let isSuccess = $state(false);
  let newlyIssuedKey = $state('');

  async function handleSubmit(e) {
    e.preventDefault();
    errorMsg = '';
    formError = false;

    if (!nickname.trim()) {
      errorMsg = 'Lütfen kullanıcı adınızı giriniz.';
      formError = true;
      return;
    }

    if (!recoveryKey.trim()) {
      errorMsg = 'Lütfen kurtarma kodunuzu giriniz.';
      formError = true;
      return;
    }

    if (newPassword.length < 8 || newPassword.length > 72) {
      errorMsg = 'Yeni şifre 8-72 karakter arasında olmalıdır.';
      formError = true;
      return;
    }

    if (newPassword !== repeatPassword) {
      errorMsg = 'Şifreler uyuşmuyor.';
      formError = true;
      return;
    }

    isLoading = true;

    try {
      const res = await api.resetPasswordWithRecovery(
        nickname.trim(),
        recoveryKey.trim(),
        newPassword
      );

      newlyIssuedKey = res.new_recovery_key || '';
      isSuccess = true;
      showToast('Şifreniz başarıyla güncellendi!', { type: 'success' });
    } catch (err) {
      errorMsg = err.message || 'Kurtarma kodu veya kullanıcı adı geçersiz.';
      formError = true;
      showToast(errorMsg, { type: 'error' });
    } finally {
      isLoading = false;
    }
  }

  async function copyNewKey() {
    try {
      await navigator.clipboard.writeText(newlyIssuedKey);
      showToast('Yeni kurtarma kodu kopyalandı.', { type: 'success' });
    } catch {
      showToast('Kopyalama başarısız oldu.', { type: 'error' });
    }
  }

  function downloadNewKey() {
    const text = `KEPÇE YENİ HESAP KURTARMA KODU\n---------------------------------------------\nKullanıcı Adı: ${nickname}\nYeni Kurtarma Kodu: ${newlyIssuedKey}\nTarih: ${new Date().toLocaleString('tr-TR')}\n\nÖNEMLİ: Şifre sıfırlama işlemi sonrası eski kodunuz iptal edilmiş ve bu yeni kod üretilmiştir.\nBu kodu güvenli bir yerde saklayınız.\n---------------------------------------------\n`;
    const blob = new Blob([text], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `kepce_yeni_kurtarma_kodu_${nickname}.txt`;
    a.click();
    URL.revokeObjectURL(url);
    showToast('Kurtarma kodu dosyası indirildi.', { type: 'success' });
  }
</script>

<Seo
  title="Şifre Sıfırlama | Kepçe"
  description="Kepçe hesap kurtarma ve şifre sıfırlama."
  noindex={true}
/>

<h1 class="auth-page__title">Şifre Sıfırlama</h1>

<div class="auth-form-container">
  {#if isSuccess}
    <div class="u-text-center u-py-md u-mb-lg">
      <h2 class="u-text-xl u-weight-bold u-mb-xs u-color-text">Şifreniz Güncellendi</h2>
      <p class="u-text-sm u-color-muted">
        Eski kurtarma kodunuz iptal edildi. Bir sonraki şifre sıfırlama işlemi için aşağıdaki <strong
          >yeni kurtarma kodunu</strong
        > güvenli bir yere kaydedin.
      </p>

      <div class="recovery-key-box u-my-md">
        {newlyIssuedKey}
      </div>

      <div class="u-flex u-gap-sm u-flex-justify-center u-mb-lg">
        <button type="button" class="btn btn--secondary btn--squish" onclick={copyNewKey}>
          {@html icon('copy', 16)} Kopyala
        </button>
        <button type="button" class="btn btn--secondary btn--squish" onclick={downloadNewKey}>
          {@html icon('download', 16)} Dosya olarak indir
        </button>
      </div>

      <button type="button" class="btn btn--primary u-w-full" onclick={() => goto('/giris')}>
        Yeni Şifre ile Giriş Yap
      </button>
    </div>
  {:else}
    <form class="auth-form" onsubmit={handleSubmit}>
      <div
        class="form-group form-group--floating"
        class:form-group--error={formError && !nickname}
        data-error={formError && !nickname ? 'Kullanıcı adınızı giriniz' : undefined}
      >
        <input
          type="text"
          id="reset-nickname"
          bind:value={nickname}
          required
          autocomplete="username"
          placeholder=" "
        />
        <label class="form-label" for="reset-nickname"
          >Kullanıcı Adı<span class="form-required-mark">*</span></label
        >
      </div>

      <div
        class="form-group form-group--floating"
        class:form-group--error={formError && !recoveryKey}
        data-error={formError && !recoveryKey ? 'Kurtarma kodunuzu giriniz' : undefined}
      >
        <input
          type="text"
          id="reset-recovery-key"
          bind:value={recoveryKey}
          required
          autocomplete="off"
          placeholder=" "
          oninput={() => {
            recoveryKey = recoveryKey.toLowerCase();
          }}
        />
        <label class="form-label" for="reset-recovery-key"
          >Kurtarma Kodu (örn. kpc-...)<span class="form-required-mark">*</span></label
        >
      </div>

      <div
        class="form-group form-group--floating"
        class:form-group--error={formError && newPassword.length < 8}
        data-error={formError && newPassword.length < 8
          ? 'Yeni şifre en az 8 karakter olmalıdır'
          : undefined}
      >
        <input
          type={showPassword ? 'text' : 'password'}
          id="reset-new-password"
          bind:value={newPassword}
          required
          autocomplete="new-password"
          placeholder=" "
        />
        <label class="form-label" for="reset-new-password"
          >Yeni Şifre<span class="form-required-mark">*</span></label
        >
        <button
          type="button"
          class="password-toggle"
          aria-label={showPassword ? 'Şifreyi gizle' : 'Şifreyi göster'}
          onclick={() => (showPassword = !showPassword)}
        >
          {@html icon(showPassword ? 'eyeNotLooking' : 'eyeLooking', 20)}
        </button>
      </div>

      <div
        class="form-group form-group--floating"
        class:form-group--error={formError && newPassword !== repeatPassword}
        data-error={formError && newPassword !== repeatPassword ? 'Şifreler uyuşmuyor' : undefined}
      >
        <input
          type={showRepeatPassword ? 'text' : 'password'}
          id="reset-repeat-password"
          bind:value={repeatPassword}
          required
          autocomplete="new-password"
          placeholder=" "
        />
        <label class="form-label" for="reset-repeat-password"
          >Yeni Şifre (Tekrar)<span class="form-required-mark">*</span></label
        >
        <button
          type="button"
          class="password-toggle"
          aria-label={showRepeatPassword ? 'Şifreyi gizle' : 'Şifreyi göster'}
          onclick={() => (showRepeatPassword = !showRepeatPassword)}
        >
          {@html icon(showRepeatPassword ? 'eyeNotLooking' : 'eyeLooking', 20)}
        </button>
      </div>

      <button
        type="submit"
        class="btn btn--primary btn--large u-w-full btn--squish u-mt-xl"
        disabled={isLoading}
      >
        {isLoading ? 'Sıfırlanıyor...' : 'Şifremi Sıfırla ve Yeni Kod Al'}
      </button>
    </form>
  {/if}

  <div class="auth-footer">
    <h2 class="auth-footer__title">Hatırladın mı:</h2>
    <div class="auth-footer__links">
      <a href="/giris" class="auth-footer__link" data-link>Giriş yapılası</a>
      <a href="/kayit" class="auth-footer__link" data-link>Kayıtlı kullanıcı olunası</a>
    </div>
  </div>
</div>
