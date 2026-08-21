<script>
  import "@/styles/pages/_auth.css";
  import { slide } from "svelte/transition";
  import { goto } from "$app/navigation";
  import { globalState, authActions } from "@/state.svelte.js";

  import { api } from "@/api/index.js";

  import { CITY_MAP } from "@/utils/turkish.js";
  import { icon } from "@/components/ui/icons.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import Seo from "@/components/ui/Seo.svelte";
  import { subscribeToPush, isPushSupported } from "@/utils/push.js";

  const cityOptions = Object.entries(CITY_MAP)
    .map(([slug, name]) => ({ value: slug, label: name }))
    .sort((a, b) => a.label.localeCompare(b.label, "tr"));
  cityOptions.unshift({ value: "", label: "Belirtmek istemiyorum" });

  let user = $derived(globalState?.user);

  let username = $state("");
  let email = $state("");
  let password = $state("");
  let repeatPassword = $state("");
  let selectedCity = $state("");

  let showPassword = $state(false);
  let showRepeatPassword = $state(false);
  let honeypot = $state("");

  let isCeliac = $state(false);

  // Güvenlik e-postası bildirimi opt-in (varsayılan kapalı).
  let emailSecurity = $state(false);
  let enablePushNotifications = $state(false);

  let errors = $state({});
  let errorMsg = $state("");
  let isLoading = $state(false);

  $effect(() => {
    if (username.length > 0) {
      if (username.length < 3) {
        errors.username = "En az 3 karakter olmalıdır.";
      } else if (username.length > 30) {
        errors.username = "En fazla 30 karakter olabilir.";
      } else if (!/^[a-z0-9_]+$/.test(username)) {
        errors.username = "Sadece İngilizce küçük harf, rakam ve _ içerebilir.";
      } else {
        errors.username = null;
      }
    } else {
      errors.username = null;
    }
  });

  function generateRandomUsername() {
    const adjectives = [
      "aci",
      "tuzlu",
      "tatli",
      "eksi",
      "gurme",
      "ac",
      "doymus",
      "obur",
      "hizli",
      "yavas",
      "komik",
      "sinirli",
      "mutlu",
      "uykulu",
      "bayat",
      "taze",
      "sicak",
      "soguk",
      "pismis",
      "cig",
      "kizarmis",
      "haslanmis",
      "soslu",
      "sade",
      "karisik",
      "kivrak",
      "saskin",
    ];
    const nouns = [
      "kepce",
      "tabldot",
      "tepsi",
      "kasik",
      "catal",
      "bicak",
      "tuzluk",
      "biber",
      "domates",
      "patates",
      "pilav",
      "makarna",
      "fasulye",
      "nohut",
      "corba",
      "ekmek",
      "ayran",
      "su",
      "elma",
      "armut",
      "karpuz",
      "kavun",
      "tatli",
      "kofte",
      "tavuk",
      "et",
    ];
    const rndAdj = adjectives[Math.floor(Math.random() * adjectives.length)];
    const rndNoun = nouns[Math.floor(Math.random() * nouns.length)];
    const rndNum = Math.floor(Math.random() * 10000)
      .toString()
      .padStart(4, "0");
    username = `${rndAdj}_${rndNoun}_${rndNum}`;
  }

  function scrollToFirstError() {
    setTimeout(() => {
      const firstErrorElement = document.querySelector(
        ".form-group--error, .auth-error",
      );
      if (firstErrorElement) {
        firstErrorElement.scrollIntoView({
          behavior: "smooth",
          block: "center",
        });
      }
    }, 50);
  }

  async function handleSubmit(e) {
    e.preventDefault();
    if (honeypot) return; // Spam protection

    let newErrors = {};
    let newErrorMsg = "";

    if (username.length > 0) {
      if (
        username.length < 3 ||
        username.length > 30 ||
        !/^[a-z0-9_]+$/.test(username)
      ) {
        newErrors.username = "Kullanıcı adı kurallara uymuyor.";
        errors = newErrors;
        scrollToFirstError();
        return;
      }
    }

    if (password !== repeatPassword) {
      newErrors.repeatPassword = "Şifreler eşleşmiyor.";
      errors = newErrors;
      scrollToFirstError();
      return;
    }

    if (password.length < 8) {
      newErrors.password = "Şifre en az 8 karakter olmalı.";
      errors = newErrors;
      scrollToFirstError();
      return;
    }

    errors = newErrors;
    errorMsg = newErrorMsg;
    isLoading = true;

    try {
      let diet_mode = isCeliac ? "celiac" : null;
      await api.register(
        email,
        password,
        username.trim() || null,
        selectedCity || null,
        diet_mode,
        emailSecurity,
      );
      if (enablePushNotifications && isPushSupported()) {
        try {
          await subscribeToPush({
            cityId: null,
            breakfastEnabled: true,
            breakfastTime: "07:30",
            dinnerEnabled: true,
            dinnerTime: "17:00",
          });
        } catch (pushErr) {
          console.warn("Kayıt anında bildirim izni alınamadı:", pushErr);
        }
      }

      const { showToast } = await import("@/components/ui/toast.js");
      showToast(
        "Kayıt başarılı! Doğrulama linki e-postana gönderildi (Lütfen gereksiz/spam klasörünü de kontrol et).",
        { type: "success" },
      );
      goto("/giris");
    } catch (err) {
      const msg = err.message.toLowerCase();
      let failedErrors = {};
      if (msg.includes("email") || msg.includes("e-posta")) {
        failedErrors.email = err.message;
      } else if (msg.includes("kullanıcı adı") || msg.includes("username")) {
        failedErrors.username = err.message;
      } else {
        errorMsg = err.message || "Bilinmeyen bir sorun oluştu.";
      }
      errors = failedErrors;
      scrollToFirstError();
    } finally {
      isLoading = false;
    }
  }

  function handleLogout() {
    authActions.logout();
  }
</script>

<Seo
  title="Kayıt Ol - Kepçe"
  description="Ücretsiz Kepçe hesabı oluşturun; yurt yemeklerini değerlendirin, yorum yapın ve menü takibi yapın."
/>

{#if user}
  <div class="empty-state-container">
    <EmptyState
      statusCode={403}
      title={"Zaten Aramızdasın!"}
      desc={`@${user.username} olarak zaten giriş yapmışsın. Yeni bir hesap açmak için önce çıkış yapmalısın.`}
    >
      <a href="/" data-link class="btn btn--secondary">Ana sayfaya dön</a>
      <button type="button" class="btn btn--primary" onclick={handleLogout}
        >Çıkış yap</button
      >
    </EmptyState>
  </div>
{:else}
  <div class="auth-form-container auth-form-container--wide">
    <h1 class="auth-page__title u-mb-xs">Kayıt formu</h1>
    <div
      class="form-footer-hint u-mb-xl u-text-sm u-weight-semibold u-color-secondary"
    >
      <span class="form-required-mark">*</span>: Zorunlu
    </div>

    {#if errorMsg}
      <div id="register-error-container">
        <div class="auth-error" role="alert">{errorMsg}</div>
      </div>
    {/if}

    <form class="auth-form" onsubmit={handleSubmit}>
      <!-- Honeypot field for spam protection -->
      <input
        type="text"
        name="website"
        tabindex="-1"
        autocomplete="off"
        class="u-honeypot"
        bind:value={honeypot}
      />

      <div class="auth-form-row">
        <div class="form-group">
          <div
            class="form-group--floating u-mb-0"
            class:form-group--error={errors.username}
          >
            <input
              type="text"
              id="username"
              bind:value={username}
              autocomplete="username"
              placeholder=" "
            />
            <label class="form-label" for="username">Kullanıcı adı</label>
            <button
              type="button"
              class="password-toggle"
              aria-label="Rastgele Kullanıcı Adı Üret"
              onclick={generateRandomUsername}
              title="Rastgele üret"
            >
              {@html icon("dice", 20)}
            </button>
          </div>
          <div
            class="form-help u-mt-xs u-weight-semibold"
            class:u-color-negative={errors.username}
            class:u-color-secondary={!errors.username}
          >
            Boş bırakılırsa e-postanızdan otomatik üretilir. 3-30 karakter arası
            sadece İngilizce küçük harf, rakam ve _ içerebilir.
          </div>
        </div>

        <div
          class="form-group form-group--floating"
          class:form-group--error={errors.email}
          data-error={errors.email}
        >
          <input
            type="email"
            id="email"
            bind:value={email}
            required
            autocomplete="email"
            placeholder=" "
          />
          <label class="form-label" for="email"
            >E-posta<span class="form-required-mark">*</span></label
          >
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
              type={showPassword ? "text" : "password"}
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
              aria-label={showPassword ? "Şifreyi gizle" : "Şifreyi göster"}
              onclick={() => (showPassword = !showPassword)}
            >
              {@html icon(showPassword ? "eyeOff" : "eye", 20)}
            </button>
          </div>
          {#if password.length > 0}
            <div
              class="password-checklist u-mt-sm u-text-sm u-weight-semibold"
              transition:slide
            >
              <div
                class="u-flex u-flex-gap-sm u-flex-align-center"
                class:u-color-positive={password.length >= 8 &&
                  password.length <= 64}
                class:u-color-negative={password.length < 8 ||
                  password.length > 64}
              >
                {@html icon(
                  password.length >= 8 && password.length <= 64
                    ? "checkCircle"
                    : "circle",
                  16,
                )} 8-64 karakter uzunluğunda
              </div>
              <div
                class="u-flex u-flex-gap-sm u-flex-align-center u-mt-2xs"
                class:u-color-positive={/[a-zA-Z]/.test(password)}
                class:u-color-negative={!/[a-zA-Z]/.test(password)}
              >
                {@html icon(
                  /[a-zA-Z]/.test(password) ? "checkCircle" : "circle",
                  16,
                )} En az bir harf
              </div>
              <div
                class="u-flex u-flex-gap-sm u-flex-align-center u-mt-2xs"
                class:u-color-positive={/\d/.test(password)}
                class:u-color-negative={!/\d/.test(password)}
              >
                {@html icon(/\d/.test(password) ? "checkCircle" : "circle", 16)}
                En az bir rakam
              </div>
            </div>
          {/if}
        </div>

        <div
          class="form-group form-group--floating"
          class:form-group--error={errors.repeatPassword}
          data-error={errors.repeatPassword}
        >
          <input
            type={showRepeatPassword ? "text" : "password"}
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
            aria-label={showRepeatPassword ? "Şifreyi gizle" : "Şifreyi göster"}
            onclick={() => (showRepeatPassword = !showRepeatPassword)}
          >
            {@html icon(showRepeatPassword ? "eyeOff" : "eye", 20)}
          </button>
        </div>
      </div>

      <div class="form-group">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label class="form-label u-mb-xs">Bulunulan şehir</label>
        <div class="dropdown-form-control">
          <Dropdown options={cityOptions} bind:value={selectedCity} />
        </div>
        <div class="form-help">
          Eğer bulunulan şehir için menü yoksa varsayılan olarak İstanbul ya da
          en son seçilen şehir gösterilir.
        </div>
      </div>

      <div class="form-group">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label class="form-label">Diyet tercihleri</label>
        <div class="c-boxed-list">
          <label class="c-list-row c-list-row--clickable">
            <div class="c-list-row__content">
              <span class="c-list-row__title">Çölyak modu</span>
              <span class="c-list-row__desc"
                >Glutensiz menüleri ve uyarıları önceliklendir</span
              >
            </div>
            <div class="c-list-row__control">
              <input
                type="checkbox"
                class="c-input-hidden"
                bind:checked={isCeliac}
              />
              <span class="c-switch"
                ><span class="c-switch__handle"></span></span
              >
            </div>
          </label>
        </div>
      </div>

      <div class="form-group">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label class="form-label">İletişim tercihleri</label>
        <div class="c-boxed-list">
          <label class="c-list-row c-list-row--clickable">
            <div class="c-list-row__content">
              <span class="c-list-row__title">Güvenlik bildirimleri</span>
              <span class="c-list-row__desc"
                >Şifre değişiklikleri ve kritik güvenlik alarmlarında e-posta al</span
              >
            </div>
            <div class="c-list-row__control">
              <input
                type="checkbox"
                class="c-input-hidden"
                bind:checked={emailSecurity}
              />
              <span class="c-switch"
                ><span class="c-switch__handle"></span></span
              >
            </div>
          </label>

          <label class="c-list-row c-list-row--clickable">
            <div class="c-list-row__content">
              <span class="c-list-row__title">Öğün bildirimleri (Web Push)</span>
              <span class="c-list-row__desc"
                >Günün menüsü açıklandığında tarayıcınıza anlık bildirim gelsin</span
              >
            </div>
            <div class="c-list-row__control">
              <input
                type="checkbox"
                class="c-input-hidden"
                bind:checked={enablePushNotifications}
              />
              <span class="c-switch"
                ><span class="c-switch__handle"></span></span
              >
            </div>
          </label>
        </div>
      </div>

      <div class="register-submit-group">
        <div class="legal-consents">
          <p class="u-text-sm u-color-secondary u-mb-md">
            Hesap oluşturarak Kepçe'nin <a
              href="/kullanim-kosullari"
              target="_blank">Kullanım Koşulları</a
            >nı ve
            <a href="/gizlilik-politikasi" target="_blank"
              >Gizlilik Politikası</a
            >nı (KVKK Aydınlatma Metni) okuduğunuzu, yurt dışı aktarımı ve özel
            nitelikli veri işlenmesine açık rıza verdiğinizi kabul edersiniz.
          </p>
        </div>

        <button
          type="submit"
          class="btn btn--primary btn--large auth-submit u-w-full"
          disabled={isLoading}
        >
          {isLoading ? "Hesap oluşturuluyor..." : "Hesap oluştur"}
        </button>
      </div>
    </form>

    <div class="auth-footer--inline">
      <span class="auth-footer__title">E ama benim hesabım var:</span>
      <a href="/giris" class="auth-footer__link" data-link>Giriş yapılası</a>
    </div>
  </div>
{/if}
