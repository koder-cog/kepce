<script>
  import '@/styles/pages/_auth.css';
  import { slide } from 'svelte/transition';
  import { goto } from '$app/navigation';
  import { globalState, authActions } from '$lib/stores/appState.svelte.js';
  import { api } from '$lib/api/index.js';
  import { icon } from '$lib/components/ui/icons.js';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Seo from '$lib/components/ui/Seo.svelte';
  import { showToast } from '$lib/components/ui/toast.js';

  let user = $derived(globalState?.user);

  let username = $state('');
  let password = $state('');
  let repeatPassword = $state('');

  let showPassword = $state(false);
  let showRepeatPassword = $state(false);
  let honeypot = $state('');

  let consentTerms = $state(false);
  let consentPrivacy = $state(false);

  let errors = $state({});
  let errorMsg = $state('');
  let isLoading = $state(false);

  // Kurtarma Kodu Teyidi Durumu
  let showRecoveryModal = $state(false);
  let recoveryKey = $state('');
  let lastBlockInput = $state('');
  let recoveryAcknowledged = $state(false);
  let isConfirmingRecovery = $state(false);
  let recoveryError = $state('');

  $effect(() => {
    if (username.length > 0) {
      if (username.length < 3) {
        errors.username = 'En az 3 karakter olmalıdır.';
      } else if (username.length > 25) {
        errors.username = 'En fazla 25 karakter olabilir.';
      } else if (!/^[a-z0-9_]+$/.test(username)) {
        errors.username = 'Sadece İngilizce küçük harf, rakam ve _ içerebilir.';
      } else {
        errors.username = null;
      }
    } else {
      errors.username = null;
    }
  });

  function generateRandomUsername() {
    const adjectives = [
      'aci',
      'tuzlu',
      'tatli',
      'eksi',
      'gurme',
      'ac',
      'doymus',
      'obur',
      'hizli',
      'yavas',
      'komik',
      'sinirli',
      'mutlu',
      'uykulu',
      'bayat',
      'taze',
      'sicak',
      'soguk',
      'pismis',
      'cig',
      'kizarmis',
      'haslanmis',
      'soslu',
      'sade',
      'karisik',
      'kivrak',
      'saskin'
    ];
    const nouns = [
      'kepce',
      'tabldot',
      'tepsi',
      'kasik',
      'catal',
      'bicak',
      'tuzluk',
      'biber',
      'domates',
      'patates',
      'pilav',
      'makarna',
      'fasulye',
      'nohut',
      'corba',
      'ekmek',
      'ayran',
      'su',
      'elma',
      'armut',
      'karpuz',
      'kavun',
      'tatli',
      'kofte',
      'tavuk',
      'et'
    ];
    const rndAdj = adjectives[Math.floor(Math.random() * adjectives.length)];
    const rndNoun = nouns[Math.floor(Math.random() * nouns.length)];
    const rndNum = Math.floor(Math.random() * 10000)
      .toString()
      .padStart(4, '0');
    username = `${rndAdj}_${rndNoun}_${rndNum}`;
  }

  function scrollToFirstError() {
    setTimeout(() => {
      const firstErrorElement = document.querySelector('.form-group--error');
      if (firstErrorElement) {
        firstErrorElement.scrollIntoView({
          behavior: 'smooth',
          block: 'center'
        });
      }
    }, 50);
  }

  async function handleSubmit(e) {
    e.preventDefault();
    if (honeypot) return;

    let newErrors = {};

    if (!username || username.trim().length === 0) {
      newErrors.username = 'Kullanıcı adı alanı zorunludur.';
      errors = newErrors;
      scrollToFirstError();
      return;
    }

    if (username.length < 3 || username.length > 25 || !/^[a-z0-9_]+$/.test(username)) {
      newErrors.username = 'Kullanıcı adı kurallara uymuyor (3-25 karakter, küçük harf/rakam/_).';
      errors = newErrors;
      scrollToFirstError();
      return;
    }

    if (password !== repeatPassword) {
      newErrors.repeatPassword = 'Şifreler eşleşmiyor.';
      errors = newErrors;
      scrollToFirstError();
      return;
    }

    if (password.length < 8 || password.length > 72) {
      newErrors.password = 'Şifre 8-72 karakter arasında olmalı.';
      errors = newErrors;
      scrollToFirstError();
      return;
    }

    if (!consentTerms || !consentPrivacy) {
      return;
    }

    errors = newErrors;
    errorMsg = '';
    isLoading = true;

    try {
      const res = await api.register(username.trim(), password);

      // Kayıt başarılı: Kurtarma Kodunu Modal ile kullanıcıya sun
      recoveryKey = res.recovery_key || '';
      showRecoveryModal = true;
      await authActions.refreshUser();
    } catch (err) {
      const msg = (err.message || '').toLowerCase();
      let failedErrors = {};
      if (
        msg.includes('kullanıcı adı') ||
        msg.includes('nickname') ||
        msg.includes('rumuz') ||
        msg.includes('zaten')
      ) {
        failedErrors.username = err.message;
      } else {
        errorMsg = err.message || 'Bilinmeyen bir sorun oluştu.';
        showToast(errorMsg, { type: 'error' });
      }
      errors = failedErrors;
      scrollToFirstError();
    } finally {
      isLoading = false;
    }
  }

  async function copyRecoveryKey() {
    try {
      await navigator.clipboard.writeText(recoveryKey);
      showToast('Kurtarma kodu kopyalandı.', { type: 'success' });
    } catch {
      showToast('Kopyalama başarısız oldu.', { type: 'error' });
    }
  }

  function downloadRecoveryFile() {
    const text = `KEPÇE HESAP KURTARMA KODU\n---------------------------------------------\nKullanıcı Adı: ${username}\nKurtarma Kodu: ${recoveryKey}\nTarih: ${new Date().toLocaleString('tr-TR')}\n\nÖNEMLİ: Kepçe'de e-posta veya telefon tutulmaz.\nŞifrenizi unutursanız hesabınızı kurtarmanın tek yolu bu koddur.\nDosyayı güvenli bir yerde saklayınız.\n---------------------------------------------\n`;
    const blob = new Blob([text], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `kepce_kurtarma_kodu_${username}.txt`;
    a.click();
    URL.revokeObjectURL(url);
    showToast('Kurtarma kodu dosyası indirildi.', { type: 'success' });
  }

  async function handleConfirmRecovery(e) {
    e?.preventDefault();
    if (!recoveryAcknowledged || lastBlockInput.trim().length !== 5) {
      return;
    }

    isConfirmingRecovery = true;
    recoveryError = '';

    try {
      await api.confirmRecovery(lastBlockInput.trim().toLowerCase());
      showToast('Kurtarma kodu teyit edildi, hoş geldiniz!', { type: 'success' });
      showRecoveryModal = false;
      await authActions.refreshUser();
      goto('/');
    } catch (err) {
      recoveryError = err.message || 'Son blok eşleşmedi. Lütfen kodu tekrar kontrol ediniz.';
      showToast(recoveryError, { type: 'error' });
    } finally {
      isConfirmingRecovery = false;
    }
  }

  function handleLogout() {
    authActions.logout();
  }
</script>

<Seo
  title="Kayıt Ol | Kepçe"
  description="Ücretsiz Kepçe hesabı oluşturun, yurt yemeklerini değerlendirin, menü takibi yapın."
/>

{#if user && !showRecoveryModal}
  <div class="empty-state-container">
    <EmptyState
      statusCode={403}
      title={'Zaten Aramızdasın!'}
      desc={`@${user.username} olarak zaten giriş yapmışsın. Yeni bir hesap açmak için önce çıkış yapmalısın.`}
    >
      <a href="/" data-link class="btn btn--secondary">Ana sayfaya dön</a>
      <button type="button" class="btn btn--primary" onclick={handleLogout}>Çıkış yap</button>
    </EmptyState>
  </div>
{:else}
  <div class="auth-form-container auth-form-container--wide">
    <h1 class="auth-page__title u-mb-lg">Kayıt ol</h1>

    <div class="form-footer-hint u-mb-xl u-text-sm u-weight-semibold u-color-secondary">
      <span class="form-required-mark">*</span>: Zorunlu
    </div>

    <form class="auth-form" onsubmit={handleSubmit}>
      <!-- Honeypot field for spam protection -->
      <input
        type="text"
        id="register-website-honeypot"
        name="website"
        aria-label="Website"
        tabindex="-1"
        autocomplete="off"
        class="u-honeypot"
        bind:value={honeypot}
      />

      <div class="form-group">
        <div class="form-group--floating u-mb-0" class:form-group--error={errors.username}>
          <input
            type="text"
            id="username"
            bind:value={username}
            autocomplete="username"
            required
            placeholder=" "
          />
          <label class="form-label" for="username"
            >Kullanıcı Adı<span class="form-required-mark">*</span></label
          >
          <button
            type="button"
            class="password-toggle"
            aria-label="Rastgele Kullanıcı Adı Üret"
            onclick={generateRandomUsername}
            title="Rastgele üret"
          >
            {@html icon('dice', 20)}
          </button>
        </div>
        <div
          class="form-help u-mt-xs u-weight-semibold"
          class:u-color-negative={errors.username}
          class:u-color-secondary={!errors.username}
        >
          3-25 karakter arası sadece İngilizce küçük harf, rakam ve _ içerebilir.
        </div>
      </div>

      <div class="auth-form-row">
        <div class="form-group">
          <div
            class="form-group--floating u-mb-0"
            class:form-group--error={errors.password}
            data-error={errors.password}
          >
            <input
              type={showPassword ? 'text' : 'password'}
              id="password"
              bind:value={password}
              required
              autocomplete="new-password"
              placeholder=" "
            />
            <label class="form-label" for="password"
              >Şifre<span class="form-required-mark">*</span></label
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
          <div class="form-help u-mt-xs">En az 8 karakter, harf ve rakam içermelidir.</div>
        </div>

        <div
          class="form-group form-group--floating"
          class:form-group--error={errors.repeatPassword}
          data-error={errors.repeatPassword}
        >
          <input
            type={showRepeatPassword ? 'text' : 'password'}
            id="password-repeat"
            bind:value={repeatPassword}
            required
            autocomplete="new-password"
            placeholder=" "
          />
          <label class="form-label" for="password-repeat"
            >Şifre (tekrar)<span class="form-required-mark">*</span></label
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
      </div>

      <div class="register-submit-group">
        <div class="legal-consents">
          <label class="form-switch-row">
            <input type="checkbox" class="c-input-hidden" bind:checked={consentTerms} />
            <div class="c-switch">
              <div class="c-switch__handle"></div>
            </div>
            <span class="form-switch-row__text">
              <a href="/kullanim-kosullari" target="_blank" onclick={(e) => e.stopPropagation()}
                >Kullanım Koşulları</a
              > metnini okudum ve kabul ediyorum
            </span>
          </label>

          <label class="form-switch-row">
            <input type="checkbox" class="c-input-hidden" bind:checked={consentPrivacy} />
            <div class="c-switch">
              <div class="c-switch__handle"></div>
            </div>
            <span class="form-switch-row__text">
              <a href="/gizlilik-politikasi" target="_blank" onclick={(e) => e.stopPropagation()}
                >Gizlilik Politikası</a
              > ve aydınlatma bildirimini okudum
            </span>
          </label>
        </div>

        <button
          type="submit"
          class="btn btn--primary btn--large u-w-full btn--squish u-mt-xl"
          disabled={isLoading || !consentTerms || !consentPrivacy}
        >
          {isLoading ? 'Hesap oluşturuluyor...' : 'Hesap oluştur'}
        </button>
      </div>
    </form>

    <div class="auth-footer--inline">
      <span class="auth-footer__title">E ama benim hesabım var:</span>
      <a href="/giris" class="auth-footer__link" data-link>Giriş yapılası</a>
    </div>
  </div>
{/if}

{#if showRecoveryModal}
  <Modal
    options={{
      title: 'Kurtarma Kodu',
      iconHtml: icon('lock', 24)
    }}
    onClose={() => {
      showRecoveryModal = false;
      goto('/');
    }}
  >
    {#snippet children()}
      <div class="u-flex u-flex-column u-gap-md">
        <p class="u-text-sm u-color-muted u-mb-sm">
          Şifrenizi unutursanız hesabınıza erişmenin tek yolu bu koddur. E-posta veya telefon
          tutulmadığı için bu kod olmadan şifre sıfırlanamaz.
        </p>

        <div class="recovery-key-box">
          {recoveryKey}
        </div>

        <div class="u-flex u-gap-sm u-flex-justify-center u-mb-md">
          <button type="button" class="btn btn--secondary btn--squish" onclick={copyRecoveryKey}>
            {@html icon('copy', 16)} Kopyala
          </button>
          <button
            type="button"
            class="btn btn--secondary btn--squish"
            onclick={downloadRecoveryFile}
          >
            {@html icon('download', 16)} Dosya olarak indir
          </button>
        </div>

        <form
          id="recovery-confirm-form"
          class="recovery-recall-form u-mt-md"
          onsubmit={handleConfirmRecovery}
        >
          <div
            class="form-group form-group--floating"
            class:form-group--error={!!recoveryError}
            data-error={recoveryError || undefined}
          >
            <input
              type="text"
              id="recall-last-block"
              maxlength="5"
              bind:value={lastBlockInput}
              class="form-input"
              placeholder=" "
              required
              autocomplete="off"
              oninput={() => {
                lastBlockInput = lastBlockInput.toLowerCase();
              }}
            />
            <label class="form-label" for="recall-last-block">
              Son 5 Karakter<span class="form-required-mark">*</span>
            </label>
          </div>

          <label class="form-switch-row u-mt-lg">
            <input type="checkbox" class="c-input-hidden" bind:checked={recoveryAcknowledged} />
            <div class="c-switch">
              <div class="c-switch__handle"></div>
            </div>
            <span class="form-switch-row__text u-text-xs u-weight-bold">
              Kurtarma kodumu güvenli bir yere kaydettim.
            </span>
          </label>
        </form>
      </div>
    {/snippet}

    {#snippet footer()}
      <button
        type="submit"
        form="recovery-confirm-form"
        class="btn btn--primary u-w-full"
        disabled={isConfirmingRecovery ||
          !recoveryAcknowledged ||
          lastBlockInput.trim().length !== 5}
      >
        {isConfirmingRecovery ? 'Teyit ediliyor...' : 'Teyit Et ve Tamamla'}
      </button>
    {/snippet}
  </Modal>
{/if}
