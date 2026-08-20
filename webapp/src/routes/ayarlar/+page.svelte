<script>
  import "@/styles/pages/_settings.css";
  import { globalState, authActions } from "@/state.svelte.js";
  import { goto } from "$app/navigation";

  import { onMount } from "svelte";
  import { icon } from "@/components/ui/icons.js";
  import { api } from "@/api/index.js";
  import { getCitiesData } from "@/stores/city.svelte.js";
  import { timelineState } from "@/stores/timeline.svelte.js";
  import { showToast } from "@/components/ui/toast.js";
  import Modal from "@/components/ui/Modal.svelte";
  import SegmentedControl from "@/components/ui/SegmentedControl.svelte";
  import * as ui from "@/components/ui/forms.js";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import SessionManagerModal from "@/components/features/SessionManagerModal.svelte";
  import Seo from "@/components/ui/Seo.svelte";
  import { subscribeToPush, unsubscribeFromPush, sendTestPush, isPushSupported } from "@/utils/push.js";

  let user = $derived(globalState?.user);
  let isResending = $state(false);

  async function handleResendVerification() {
    if (isResending) return;
    isResending = true;
    try {
      await api.resendVerification();
      showToast("Doğrulama bağlantısı e-posta adresinize gönderildi.", {
        type: "success",
      });
    } catch (e) {
      if (e.status === 429) {
        showToast(
          "Lütfen yeni bir e-posta istemeden önce 24 saat bekleyiniz.",
          { type: "error" },
        );
      } else {
        showToast(e.message || "E-posta gönderilirken bir hata oluştu.", {
          type: "error",
        });
      }
    } finally {
      isResending = false;
    }
  }
  let cities = $state([]);
  let cityOptions = $derived([
    { value: "", label: "Belirtmek istemiyorum" },
    ...cities
      .map((c) => ({ value: c.slug, label: c.name }))
      .sort((a, b) => a.label.localeCompare(b.label, "tr")),
  ]);

  const safeStorageGet = (key, fallback) => {
    if (typeof window === "undefined" || typeof localStorage === "undefined") {
      return fallback;
    }
    return localStorage.getItem(key) ?? fallback;
  };

  let currentTheme = $state(safeStorageGet("renkTercihi", "sistem"));

  let showBot = $state(safeStorageGet("kepce_show_bot", "true") !== "false");

  if (
    typeof window !== "undefined" &&
    typeof localStorage !== "undefined" &&
    localStorage.getItem("kepce_show_empty_cards") === null &&
    localStorage.getItem("kepce_hide_empty_cards") !== null
  ) {
    const wasHidden = localStorage.getItem("kepce_hide_empty_cards") === "true";
    localStorage.setItem(
      "kepce_show_empty_cards",
      !wasHidden ? "true" : "false",
    );
    localStorage.removeItem("kepce_hide_empty_cards");
  }

  let showEmptyCards = $state(
    safeStorageGet("kepce_show_empty_cards", "true") === "true",
  );
  let scrollbarPermanent = $state(
    safeStorageGet("kepce_scrollbar_permanent", "false") === "true",
  );
  let animationsEnabled = $state(
    safeStorageGet("kepce_animations", "true") !== "false",
  );
  let effectsEnabled = $state(
    safeStorageGet("kepce_effects", "true") !== "false",
  );
  let showIndicators = $state(
    safeStorageGet("kepce_show_indicators", "false") === "true",
  );
  import { setPaginationMode } from "@/state.svelte.js";
  let paginationMode = $state(safeStorageGet("sayfalamaModu", "sayfali"));

  function handlePaginationModeChange(val) {
    setPaginationMode(val);
    showToast(val === "sayfali" ? "Numaralı sayfalama seçildi." : "Akıcı liste akışı seçildi.", { type: "info" });
  }

  let dietMode = $state(safeStorageGet("kepce_diet_mode", "standard"));
  let externalLinkWarning = $state(
    safeStorageGet("kepce_external_link_warning", "true") !== "false",
  );

  onMount(async () => {
    try {
      cities = await getCitiesData();
    } catch (e) {
      console.error(e);
    }

    if (typeof window !== "undefined" && window.AndroidBridge && window.AndroidBridge.getNotificationSettings) {
      try {
        const notifJson = JSON.parse(window.AndroidBridge.getNotificationSettings());
        if (notifJson && !user) {
          anonBreakfastEnabled = Boolean(notifJson.breakfast_enabled);
          anonBreakfastTime = notifJson.breakfast_time || "07:30";
          anonDinnerEnabled = Boolean(notifJson.dinner_enabled);
          anonDinnerTime = notifJson.dinner_time || "16:30";
        }
      } catch (err) {
        console.warn("AndroidBridge ayar okuma hatası:", err);
      }
    }
  });

  $effect(() => {
    if (currentTheme) {
      localStorage.setItem("renkTercihi", currentTheme);
      window.applyTheme && window.applyTheme(currentTheme);
    }
  });

  function handleBotToggle() {
    localStorage.setItem("kepce_show_bot", showBot);
    document.documentElement.classList.toggle("hide-ai", !showBot);
  }

  function handleShowEmptyCardsToggle() {
    localStorage.setItem("kepce_show_empty_cards", showEmptyCards);
  }

  function handleDevModeToggle() {
    localStorage.setItem("kepce_dev_mode", globalState.devMode);
  }

  function handleScrollbarToggle() {
    localStorage.setItem("kepce_scrollbar_permanent", scrollbarPermanent);
    window.dispatchEvent(new CustomEvent("scrollbar-setting-changed"));
  }

  function handleAnimationsToggle() {
    localStorage.setItem("kepce_animations", animationsEnabled);
    document.documentElement.classList.toggle(
      "disable-animations",
      !animationsEnabled,
    );
  }

  function handleEffectsToggle() {
    localStorage.setItem("kepce_effects", effectsEnabled);
    document.documentElement.classList.toggle(
      "disable-effects",
      !effectsEnabled,
    );
  }

  function handleIndicatorsToggle() {
    localStorage.setItem("kepce_show_indicators", showIndicators);
    document.documentElement.classList.toggle(
      "show-indicators",
      showIndicators,
    );
    document.body.classList.toggle("show-indicators", showIndicators);
  }

  function handleDietToggle() {
    timelineState.setPermanentDietMode(dietMode);
  }

  function handleExternalLinkWarningToggle() {
    localStorage.setItem("kepce_external_link_warning", externalLinkWarning);
  }

  async function handleDefaultCityChange(val) {
    try {
      await api.updateProfile({ default_city_slug: val || null });
      globalState.user.default_city_slug = val || null;
      showToast("Varsayılan şehir tercihiniz güncellendi.", "success");
    } catch (err) {
      showToast(err.message, "error");
    }
  }

  async function handleOptOutChange(e) {
    try {
      await api.updateProfile({ opt_out_statistics: e.target.checked });
      globalState.user.opt_out_statistics = e.target.checked;
      showToast("Gizlilik tercihiniz güncellendi.", "success");
    } catch (err) {
      showToast(err.message, "error");
    }
  }

  async function handlePreferenceChange(key, value, label) {
    try {
      await api.updateProfile({ [key]: value });
      if (globalState.user) {
        globalState.user[key] = value;
      }
      showToast(`${label} tercihiniz güncellendi.`, { type: "success" });
    } catch (err) {
      showToast(err.message || "Ayar güncellenirken bir hata oluştu.", { type: "error" });
    }
  }

  let isTestingPush = $state(false);

  // Anonim öğün bildirim state'leri
  let anonBreakfastEnabled = $state(safeStorageGet("kepce_notif_breakfast_enabled", "false") === "true");
  let anonBreakfastTime = $state(safeStorageGet("kepce_notif_breakfast_time", "07:30"));
  let anonDinnerEnabled = $state(safeStorageGet("kepce_notif_dinner_enabled", "false") === "true");
  let anonDinnerTime = $state(safeStorageGet("kepce_notif_dinner_time", "17:00"));

  async function syncPushSubscription(breakfastEnabled, breakfastTime, dinnerEnabled, dinnerTime) {
    if (typeof window !== "undefined" && window.AndroidBridge && window.AndroidBridge.updateNotificationSettings) {
      try {
        window.AndroidBridge.updateNotificationSettings(
          Boolean(breakfastEnabled),
          breakfastTime || "07:30",
          Boolean(dinnerEnabled),
          dinnerTime || "16:30"
        );
      } catch (err) {
        console.warn("AndroidBridge bildirim senkronizasyon hatası:", err);
      }
    }

    if (!isPushSupported()) return;
    if (!breakfastEnabled && !dinnerEnabled) {
      await unsubscribeFromPush().catch(() => {});
      return;
    }

    try {
      const activeCitySlug = globalState.user?.default_city_slug || timelineState.selectedCitySlug || "ankara";
      const matchedCity = cities.find((c) => c.slug === activeCitySlug);
      const cityId = matchedCity ? matchedCity.id : null;

      await subscribeToPush({
        cityId,
        breakfastEnabled,
        breakfastTime,
        dinnerEnabled,
        dinnerTime,
      });
    } catch (err) {
      console.error("Push sync hatası:", err);
      showToast(err.message || "Bildirim izni alınamadı.", { type: "error" });
    }
  }

  async function handleMealNotifToggle(meal, checked) {
    if (user) {
      const key = meal === "breakfast" ? "notif_breakfast_enabled" : "notif_dinner_enabled";
      await handlePreferenceChange(key, checked, meal === "breakfast" ? "Kahvaltı bildirimi" : "Akşam yemeği bildirimi");
      const bEnabled = meal === "breakfast" ? checked : (user.notif_breakfast_enabled ?? false);
      const bTime = user.notif_breakfast_time || "07:30";
      const dEnabled = meal === "dinner" ? checked : (user.notif_dinner_enabled ?? false);
      const dTime = user.notif_dinner_time || "17:00";
      await syncPushSubscription(bEnabled, bTime, dEnabled, dTime);
    } else {
      if (meal === "breakfast") {
        anonBreakfastEnabled = checked;
        localStorage.setItem("kepce_notif_breakfast_enabled", String(checked));
      } else {
        anonDinnerEnabled = checked;
        localStorage.setItem("kepce_notif_dinner_enabled", String(checked));
      }
      showToast(`${meal === "breakfast" ? "Kahvaltı" : "Akşam yemeği"} bildirimi tercihiniz kaydedildi.`, { type: "success" });
      await syncPushSubscription(anonBreakfastEnabled, anonBreakfastTime, anonDinnerEnabled, anonDinnerTime);
    }
  }

  async function handleMealTimeChange(meal, time) {
    if (user) {
      const key = meal === "breakfast" ? "notif_breakfast_time" : "notif_dinner_time";
      await handlePreferenceChange(key, time, meal === "breakfast" ? "Kahvaltı saati" : "Akşam yemeği saati");
      const bEnabled = user.notif_breakfast_enabled ?? false;
      const bTime = meal === "breakfast" ? time : (user.notif_breakfast_time || "07:30");
      const dEnabled = user.notif_dinner_enabled ?? false;
      const dTime = meal === "dinner" ? time : (user.notif_dinner_time || "17:00");
      await syncPushSubscription(bEnabled, bTime, dEnabled, dTime);
    } else {
      if (meal === "breakfast") {
        anonBreakfastTime = time;
        localStorage.setItem("kepce_notif_breakfast_time", time);
      } else {
        anonDinnerTime = time;
        localStorage.setItem("kepce_notif_dinner_time", time);
      }
      showToast("Bildirim saati güncellendi.", { type: "success" });
      await syncPushSubscription(anonBreakfastEnabled, anonBreakfastTime, anonDinnerEnabled, anonDinnerTime);
    }
  }

  async function handleTestPushNotification() {
    if (isTestingPush) return;
    isTestingPush = true;
    try {
      await sendTestPush();
      showToast("Test bildirimi cihazınıza gönderildi!", { type: "success" });
    } catch (err) {
      showToast(err.message || "Test bildirimi gönderilemedi.", { type: "error" });
    } finally {
      isTestingPush = false;
    }
  }

  let isNicknameModalOpen = $state(false);
  let nicknameInput = $state("");

  let isEmailModalOpen = $state(false);
  let emailInput = $state("");
  let emailPasswordInput = $state("");

  let isPasswordModalOpen = $state(false);
  let currentPasswordInput = $state("");
  let newPasswordInput = $state("");
  let confirmPasswordInput = $state("");

  let isDeleteModalOpen = $state(false);
  let deletePasswordInput = $state("");
  let nicknamePasswordInput = $state("");

  let isSessionModalOpen = $state(false);

  function changeNickname() {
    nicknameInput = user.username || "";
    nicknamePasswordInput = "";
    isNicknameModalOpen = true;
  }

  async function saveNickname() {
    const nickname = nicknameInput.trim();
    if (!nickname || nickname === user.username) {
      isNicknameModalOpen = false;
      return;
    }
    if (!nicknamePasswordInput) {
      showToast("Güvenlik için mevcut şifreni girmelisin.", "error");
      return;
    }
    try {
      await api.updateProfile({
        username: nickname,
        current_password: nicknamePasswordInput,
      });
      showToast("Kullanıcı adın başarıyla güncellendi!", "success");
      await authActions.refreshUser();
      isNicknameModalOpen = false;
    } catch (err) {
      showToast(err.message, "error");
    }
  }

  function changeEmail() {
    emailInput = user.email || "";
    emailPasswordInput = "";
    isEmailModalOpen = true;
  }

  async function saveEmail() {
    const email = emailInput.trim();
    if (!email || email === user.email) {
      isEmailModalOpen = false;
      return;
    }
    if (!emailPasswordInput) {
      showToast("Lütfen mevcut şifrenizi giriniz.", "warning");
      return;
    }
    try {
      await api.updateProfile({
        email,
        current_password: emailPasswordInput,
      });
      showToast("E-posta adresin başarıyla güncellendi!", "success");
      await authActions.refreshUser();
      isEmailModalOpen = false;
    } catch (err) {
      showToast(err.message, "error");
    }
  }

  function changePassword() {
    currentPasswordInput = "";
    newPasswordInput = "";
    confirmPasswordInput = "";
    isPasswordModalOpen = true;
  }

  async function savePassword() {
    if (!currentPasswordInput || !newPasswordInput || !confirmPasswordInput) {
      showToast("Lütfen tüm alanları doldurunuz.", "warning");
      return;
    }
    if (newPasswordInput !== confirmPasswordInput) {
      showToast("Yeni şifreler eşleşmiyor.", "error");
      return;
    }
    if (newPasswordInput.length < 8) {
      showToast("Yeni şifre en az 8 karakter olmalıdır.", "warning");
      return;
    }
    try {
      await api.updateProfile({
        password: newPasswordInput,
        current_password: currentPasswordInput,
      });
      showToast("Şifren başarıyla güncellendi!", "success");
      isPasswordModalOpen = false;
    } catch (err) {
      showToast(err.message, "error");
    }
  }

  function deleteAccount() {
    deletePasswordInput = "";
    isDeleteModalOpen = true;
  }

  async function confirmDeleteAccount() {
    if (!deletePasswordInput) {
      showToast("Hesabını silmek için şifreni girmelisin.", "error");
      return;
    }
    try {
      await api.deleteAccount(deletePasswordInput);
      showToast("Hesabın ve tüm verilerin kalıcı olarak silindi.", "success");
      await authActions.logout();
    } catch (err) {
      showToast(err.message || "Hesap silinirken bir hata oluştu.", "error");
    }
  }
</script>

<Seo
  title="Ayarlar - Kepçe"
  description="Kepçe hesap, bildirim ve görünüm ayarları."
  noindex={true}
/>

<!-- Modallar tamamen aynı kalıyor -->
{#if isNicknameModalOpen}
  <Modal
    options={{ title: "Kullanıcı adını değiştir", iconHtml: icon("edit", 24) }}
    onClose={() => (isNicknameModalOpen = false)}
  >
    {#snippet children()}
      <div class="form-group form-group--floating">
        <input
          type="text"
          id="new-nickname"
          class="form-input"
          placeholder=" "
          bind:value={nicknameInput}
          maxlength="20"
        />
        <label class="form-label" for="new-nickname">Yeni kullanıcı adı</label>
      </div>
      <div class="form-group form-group--floating">
        <input
          type="password"
          id="nickname-password"
          class="form-input"
          placeholder=" "
          bind:value={nicknamePasswordInput}
          maxlength="128"
        />
        <label class="form-label" for="nickname-password">Mevcut şifre</label>
      </div>
      <p class="u-mt-md u-text-sm u-color-muted">
        Kullanıcı adını değiştirdiğinde profil adresin de değişecektir.
      </p>
    {/snippet}
    {#snippet footer()}
      <button
        class="btn btn--secondary"
        onclick={() => (isNicknameModalOpen = false)}>İptal</button
      >
      <button class="btn btn--primary" onclick={saveNickname}>Güncelle</button>
    {/snippet}
  </Modal>
{/if}

{#if isEmailModalOpen}
  <Modal
    options={{ title: "E-posta adresini değiştir", iconHtml: icon("mail", 24) }}
    onClose={() => (isEmailModalOpen = false)}
  >
    {#snippet children()}
      <div class="form-group form-group--floating">
        <input
          type="email"
          id="new-email"
          class="form-input"
          placeholder=" "
          bind:value={emailInput}
        />
        <label class="form-label" for="new-email">Yeni e-posta adresi</label>
      </div>
      <div class="form-group form-group--floating">
        <input
          type="password"
          id="current-password-email"
          class="form-input"
          placeholder=" "
          bind:value={emailPasswordInput}
        />
        <label class="form-label" for="current-password-email"
          >Mevcut şifre</label
        >
      </div>
    {/snippet}
    {#snippet footer()}
      <button
        class="btn btn--secondary"
        onclick={() => (isEmailModalOpen = false)}>İptal</button
      >
      <button class="btn btn--primary" onclick={saveEmail}>Güncelle</button>
    {/snippet}
  </Modal>
{/if}

{#if isPasswordModalOpen}
  <Modal
    options={{ title: "Şifreni değiştir", iconHtml: icon("lock", 24) }}
    onClose={() => (isPasswordModalOpen = false)}
  >
    {#snippet children()}
      <div class="form-group form-group--floating">
        <input
          type="password"
          id="current-password"
          class="form-input"
          placeholder=" "
          bind:value={currentPasswordInput}
        />
        <label class="form-label" for="current-password">Mevcut şifre</label>
      </div>
      <div class="form-group form-group--floating">
        <input
          type="password"
          id="new-password"
          class="form-input"
          placeholder=" "
          bind:value={newPasswordInput}
        />
        <label class="form-label" for="new-password">Yeni şifre</label>
      </div>
      <div class="form-group form-group--floating">
        <input
          type="password"
          id="confirm-password"
          class="form-input"
          placeholder=" "
          bind:value={confirmPasswordInput}
        />
        <label class="form-label" for="confirm-password"
          >Yeni şifre (Tekrar)</label
        >
      </div>
    {/snippet}
    {#snippet footer()}
      <button
        class="btn btn--secondary"
        onclick={() => (isPasswordModalOpen = false)}>İptal</button
      >
      <button class="btn btn--primary" onclick={savePassword}
        >Şifreyi güncelle</button
      >
    {/snippet}
  </Modal>
{/if}

{#if isDeleteModalOpen}
  <Modal
    options={{
      title: "Hesabını sil",
      iconHtml: icon("warning", 24),
      iconColor: "danger",
    }}
    onClose={() => (isDeleteModalOpen = false)}
  >
    {#snippet children()}
      <p>Hesabını silmek istediğinden emin misin? Bu işlem geri alınamaz.</p>
      <div class="form-group form-group--floating u-mt-md">
        <input
          type="password"
          id="delete-password"
          class="form-input"
          placeholder=" "
          bind:value={deletePasswordInput}
          maxlength="128"
        />
        <label class="form-label" for="delete-password">Mevcut şifre</label>
      </div>
    {/snippet}
    {#snippet footer()}
      <button
        class="btn btn--secondary"
        onclick={() => (isDeleteModalOpen = false)}>İptal</button
      >
      <button class="btn btn--danger" onclick={confirmDeleteAccount}
        >Evet, Sil</button
      >
    {/snippet}
  </Modal>
{/if}

{#if isSessionModalOpen}
  <SessionManagerModal onClose={() => (isSessionModalOpen = false)} />
{/if}

<div class="settings-page" id="settings-page">
  <h1 class="settings-page__title">Ayarlar</h1>

  {#if globalState.user}
    <section class="settings-section" id="settings-account">
      <h2 class="settings-section__heading">Hesap</h2>
      <div class="c-boxed-list">
        <!-- Kullanıcı adı -->
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Kullanıcı adı</div>
            <div class="c-list-row__desc">
              {user.username || "Belirlenmedi"}
            </div>
          </div>
          <div class="c-list-row__control">
            <button
              class="btn btn--secondary btn--squish"
              onclick={changeNickname}>Değiştir</button
            >
          </div>
        </label>

        <!-- E-posta adresi -->
        <div class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">
              E-posta adresi
              {#if !user.is_verified}
                <span class="u-color-negative u-text-xs u-ml-xs"
                  >(Doğrulanmadı)</span
                >
              {:else}
                <span class="u-color-positive u-text-xs u-ml-xs"
                  >(Doğrulandı)</span
                >
              {/if}
            </div>
            <div class="c-list-row__desc">
              {user.email}
              {#if !user.is_verified}
                &middot;
                <button
                  type="button"
                  class="c-link u-text-xs"
                  onclick={handleResendVerification}
                  disabled={isResending}
                >
                  {isResending ? "Gönderiliyor..." : "Doğrulama Gönder"}
                </button>
              {/if}
            </div>
          </div>
          <div class="c-list-row__control">
            <button class="btn btn--secondary btn--squish" onclick={changeEmail}
              >Değiştir</button
            >
          </div>
        </div>

        <!-- Şifre -->
        <label class="c-list-row c-list-row--clickable c-list-row--regular">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Şifre</div>
          </div>
          <div class="c-list-row__control">
            <button
              class="btn btn--secondary btn--squish"
              onclick={changePassword}>Değiştir</button
            >
          </div>
        </label>

        <!-- Varsayılan şehir -->
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Varsayılan şehir</div>
            <div class="c-list-row__desc">
              Siteye girdiğinde otomatik seçilecek şehir
            </div>
          </div>
          <div class="c-list-row__control">
            <div class="city-dropdown-wrapper">
              <Dropdown
                options={cityOptions}
                bind:value={globalState.user.default_city_slug}
                variant="primary"
                onChange={handleDefaultCityChange}
              />
            </div>
          </div>
        </label>

        <!-- Kayıtlı cihazlar -->
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Kayıtlı cihazlar</div>
            <div class="c-list-row__desc">Oturum açtığın cihazları yönet</div>
          </div>
          <div class="c-list-row__control">
            <button
              class="btn btn--secondary btn--squish"
              onclick={() => (isSessionModalOpen = true)}>Yönet</button
            >
          </div>
        </label>
      </div>
    </section>
  {/if}

  <section class="settings-section" id="settings-personalization">
    <h2 class="settings-section__heading">Kişiselleştirme</h2>
    <div class="c-boxed-list">
      <!-- ==========================================
           1. GÖRÜNÜM VE EFEKTLER
      =========================================== -->

      <!-- Tema seçimi -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="c-list-row c-list-row--clickable c-list-row--tall"
        onclick={(e) => {
          if (!e.target.closest(".c-segmented-control__btn")) {
            const themes = ["sistem", "acik", "koyu"];
            const idx = themes.indexOf(currentTheme);
            currentTheme = themes[(idx + 1) % themes.length];
          }
        }}
      >
        <div class="c-list-row__info">
          <div class="c-list-row__title">Tema</div>
        </div>
        <div class="c-list-row__control c-list-row__control--flexible">
          <SegmentedControl
            bind:value={currentTheme}
            variant="responsive"
            options={[
              { value: "sistem", icon: icon("system", 18), label: "Sistem" },
              { value: "acik", icon: icon("sun", 18), label: "Açık" },
              { value: "koyu", icon: icon("moon", 18), label: "Koyu" },
            ]}
          />
        </div>
      </div>

      <!-- Sayfalama Modu -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="c-list-row c-list-row--clickable c-list-row--tall"
        onclick={(e) => {
          if (!e.target.closest(".c-segmented-control__btn")) {
            paginationMode = paginationMode === "sayfali" ? "akici" : "sayfali";
            handlePaginationModeChange(paginationMode);
          }
        }}
      >
        <div class="c-list-row__info">
          <div class="c-list-row__title">Sayfalama modu</div>
          <div class="c-list-row__desc">
            {paginationMode === "sayfali"
              ? "Numaralı (1, 2, 3...) butonlarla gezinme"
              : "Aşağı kaydırdıkça otomatik yüklenen sonsuz akış"}
          </div>
        </div>
        <div class="c-list-row__control c-list-row__control--flexible">
          <SegmentedControl
            bind:value={paginationMode}
            variant="text"
            options={[
              { value: "sayfali", label: "Sayfalı" },
              { value: "akici", label: "Sonsuz Akış" },
            ]}
            onChange={handlePaginationModeChange}
          />
        </div>
      </div>

      <!-- Görsel efektler -->
      <label class="c-list-row c-list-row--clickable c-list-row--tall">
        <div class="c-list-row__info">
          <div class="c-list-row__title">Görsel efektler</div>
          <div class="c-list-row__desc">Gölge ve bulanıklık gibi efektler</div>
        </div>
        <div class="c-list-row__control">
          <input
            type="checkbox"
            id="settings-effects-toggle"
            class="c-input-hidden"
            bind:checked={effectsEnabled}
            onchange={handleEffectsToggle}
          />
          <span class="c-switch"><span class="c-switch__handle"></span></span>
        </div>
      </label>

      <!-- Animasyon efektleri -->
      <label class="c-list-row c-list-row--clickable c-list-row--tall">
        <div class="c-list-row__info">
          <div class="c-list-row__title">Animasyon efektleri</div>
        </div>
        <div class="c-list-row__control u-flex u-align-center u-gap-sm">
          <div
            class="c-list-row__info-icon"
            data-tooltip="Bu anahtar kapatılırsa hareketler tamamen kapatılmak yerine azaltılır."
          >
            {@html icon("info", 20)}
          </div>
          <input
            type="checkbox"
            id="settings-animations-toggle"
            class="c-input-hidden"
            bind:checked={animationsEnabled}
            onchange={handleAnimationsToggle}
          />
          <span class="c-switch"><span class="c-switch__handle"></span></span>
        </div>
      </label>

      <!-- ==========================================
           2. ERİŞİLEBİLİRLİK
      =========================================== -->

      <!-- Açık/kapalı indikatörleri -->
      <label class="c-list-row c-list-row--clickable c-list-row--tall">
        <div class="c-list-row__info">
          <div class="c-list-row__title">Açık/kapalı indikatörleri</div>
          <div class="c-list-row__desc">
            Anahtar durumu için semboller kullan
          </div>
        </div>
        <div class="c-list-row__control">
          <input
            type="checkbox"
            id="settings-indicators-toggle"
            class="c-input-hidden"
            bind:checked={showIndicators}
            onchange={handleIndicatorsToggle}
          />
          <span class="c-switch"><span class="c-switch__handle"></span></span>
        </div>
      </label>

      <!-- Scrollbar'ı her zaman göster (Only if NOT isApp) -->
      {#if !globalState.isApp}
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Scrollbar'ı sürekli göster</div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="settings-scrollbar-toggle"
              class="c-input-hidden"
              bind:checked={scrollbarPermanent}
              onchange={handleScrollbarToggle}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>
      {/if}

      <!-- ==========================================
           3. İÇERİK VE ÖZELLİKLER
      =========================================== -->

      <!-- Çölyak modu -->
      <label class="c-list-row c-list-row--clickable c-list-row--tall">
        <div class="c-list-row__info">
          <div class="c-list-row__title">Çölyak modu</div>
          <div class="c-list-row__desc">
            Glutensiz menüleri ve uyarıları önceliklendir
          </div>
        </div>
        <div class="c-list-row__control">
          <input
            type="checkbox"
            id="settings-celiac-toggle"
            class="c-input-hidden"
            checked={dietMode === "celiac"}
            onchange={(e) => {
              dietMode = e.target.checked ? "celiac" : "standard";
              handleDietToggle();
            }}
          />
          <span class="c-switch"><span class="c-switch__handle"></span></span>
        </div>
      </label>

      <!-- Kepçe Bot'u göster -->
      <label class="c-list-row c-list-row--clickable c-list-row--tall">
        <div class="c-list-row__info">
          <div class="c-list-row__title">Kepçe Bot</div>
          <div class="c-list-row__desc">
            Günlük menülerdeki YZ yorumlarını göster
          </div>
        </div>
        <div class="c-list-row__control">
          <input
            type="checkbox"
            id="settings-bot-toggle"
            class="c-input-hidden"
            bind:checked={showBot}
            onchange={handleBotToggle}
          />
          <span class="c-switch"><span class="c-switch__handle"></span></span>
        </div>
      </label>

      <!-- Boş içerik kartlarını göster -->
      <label class="c-list-row c-list-row--clickable c-list-row--tall">
        <div class="c-list-row__info">
          <div class="c-list-row__title">Boş içerik kartları</div>
          <div class="c-list-row__desc">
            İçinde bilgi bulunmayan kartları göster
          </div>
        </div>
        <div class="c-list-row__control">
          <input
            type="checkbox"
            id="settings-show-empty-toggle"
            class="c-input-hidden"
            bind:checked={showEmptyCards}
            onchange={handleShowEmptyCardsToggle}
          />
          <span class="c-switch"><span class="c-switch__handle"></span></span>
        </div>
      </label>
      {#if globalState.user}
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Geliştirici modu</div>
            <div class="c-list-row__desc">Geliştirici panelini göster</div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="settings-devmode-toggle"
              class="c-input-hidden"
              bind:checked={globalState.devMode}
              onchange={handleDevModeToggle}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>
      {/if}
      <!-- ==========================================
           4. VERİ VE PERFORMANS
      =========================================== -->

      <!-- Veri tasarrufu (Altyapısı eklenecek) -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- <div
        class="c-list-row c-list-row--clickable c-list-row--tall"
        onclick={(e) => {
          if (!e.target.closest(".c-segmented-control__btn")) {
            const modes = ["off", "on", "auto"];
            const idx = modes.indexOf(dataSaverMode);
            dataSaverMode = modes[(idx + 1) % modes.length];
          }
        }}
      >
        <div class="c-list-row__info">
          <div class="c-list-row__title">Veri tasarrufu</div>
          <div class="c-list-row__desc">
            Görselleri ve ekstra medyaları yüklemez.
          </div>
        </div>
        <div class="c-list-row__control c-list-row__control--flexible">
          <SegmentedControl
            bind:value={dataSaverMode}
            options={[
              { value: "off", label: "Kapalı" },
              { value: "on", label: "Açık" },
              { value: "auto", label: "Mobil veri" },
            ]}
          />
        </div> 
      </div>
    </div> -->
    </div>
  </section>

  {#if globalState.user}
    <section class="settings-section" id="settings-communication">
      <h2 class="settings-section__heading">Haberleşme</h2>
      <h3>Bildirim</h3>
      <div class="c-boxed-list">
        <!-- Yorum yanıtları -->
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Yanıtlar</div>
            <div class="c-list-row__desc">Yorumlarına gelen yanıtlar</div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="settings-notif-replies"
              class="c-input-hidden"
              checked={globalState.user?.notif_replies ?? false}
              onchange={(e) =>
                handlePreferenceChange(
                  "notif_replies",
                  e.target.checked,
                  "Yanıt bildirimleri",
                )}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <!-- Etkileşimler -->
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Etkileşim</div>
            <div class="c-list-row__desc">Yorumlarının aldığı beğeniler</div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="settings-notif-interactions"
              class="c-input-hidden"
              checked={globalState.user?.notif_interactions ?? false}
              onchange={(e) =>
                handlePreferenceChange(
                  "notif_interactions",
                  e.target.checked,
                  "Etkileşim bildirimleri",
                )}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <!-- Sistem duyuruları -->
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Sistem</div>
            <div class="c-list-row__desc">
              Önemli güncellemeler ve moderasyon duyuruları
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="settings-notif-system"
              class="c-input-hidden"
              checked={globalState.user?.notif_system ?? false}
              onchange={(e) =>
                handlePreferenceChange(
                  "notif_system",
                  e.target.checked,
                  "Sistem bildirimleri",
                )}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>
      </div>

      <h3>Öğün Hatırlatıcıları</h3>
      <div class="c-boxed-list">
        <!-- Kahvaltı bildirimi -->
        <div class="c-list-row c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Kahvaltı</div>
            <div class="c-list-row__desc">Sabah menüsü ve eğlenceli hatırlatma</div>
          </div>
          <div class="c-list-row__actions">
            {#if (user ? user.notif_breakfast_enabled : anonBreakfastEnabled)}
              <input
                type="time"
                id="settings-notif-breakfast-time"
                class="c-time-input"
                value={user ? (user.notif_breakfast_time || "07:30") : anonBreakfastTime}
                onchange={(e) => handleMealTimeChange("breakfast", e.target.value)}
                title="Kahvaltı bildirim saati"
              />
            {/if}
            <label class="c-list-row__control">
              <input
                type="checkbox"
                id="settings-notif-breakfast"
                class="c-input-hidden"
                checked={user ? (user.notif_breakfast_enabled ?? false) : anonBreakfastEnabled}
                onchange={(e) => handleMealNotifToggle("breakfast", e.target.checked)}
              />
              <span class="c-switch"><span class="c-switch__handle"></span></span>
            </label>
          </div>
        </div>

        <!-- Akşam yemeği bildirimi -->
        <div class="c-list-row c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Akşam yemeği</div>
            <div class="c-list-row__desc">Akşam menüsü ve anlık tabldot haberi</div>
          </div>
          <div class="c-list-row__actions">
            {#if (user ? user.notif_dinner_enabled : anonDinnerEnabled)}
              <input
                type="time"
                id="settings-notif-dinner-time"
                class="c-time-input"
                value={user ? (user.notif_dinner_time || "17:00") : anonDinnerTime}
                onchange={(e) => handleMealTimeChange("dinner", e.target.value)}
                title="Akşam yemeği bildirim saati"
              />
            {/if}
            <label class="c-list-row__control">
              <input
                type="checkbox"
                id="settings-notif-dinner"
                class="c-input-hidden"
                checked={user ? (user.notif_dinner_enabled ?? false) : anonDinnerEnabled}
                onchange={(e) => handleMealNotifToggle("dinner", e.target.checked)}
              />
              <span class="c-switch"><span class="c-switch__handle"></span></span>
            </label>
          </div>
        </div>

        <!-- Test Bildirimi Gönderme Satırı -->
        {#if (user ? (user.notif_breakfast_enabled || user.notif_dinner_enabled) : (anonBreakfastEnabled || anonDinnerEnabled))}
          <div class="c-list-row c-list-row--tall">
            <div class="c-list-row__info">
              <div class="c-list-row__title">Bildirimleri Test Et</div>
              <div class="c-list-row__desc">Cihazınıza anlık bir deneme bildirimi gönderin</div>
            </div>
            <div class="c-list-row__actions">
              <button
                type="button"
                class="c-button c-button--secondary c-button--compact"
                onclick={handleTestPushNotification}
                disabled={isTestingPush}
              >
                {isTestingPush ? "Gönderiliyor..." : "Test Bildirimi Gönder"}
              </button>
            </div>
          </div>
        {/if}
      </div>
      <h3>E-posta</h3>
      <div class="c-boxed-list">
        <!-- Güvenlik bildirimleri -->
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Güvenlik</div>
            <div class="c-list-row__desc">
              Şifre değişiklikleri ve kritik güvenlik alarmları
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="settings-email-security"
              class="c-input-hidden"
              checked={globalState.user?.email_security ?? false}
              onchange={(e) =>
                handlePreferenceChange(
                  "email_security",
                  e.target.checked,
                  "Güvenlik e-postaları",
                )}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>
      </div>
    </section>

    <section class="settings-section" id="settings-privacy">
      <h2 class="settings-section__heading">Gizlilik</h2>
      <div class="c-boxed-list">
        <!-- Dış bağlantı uyarısı -->
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">Dış bağlantı uyarısı</div>
            <div class="c-list-row__desc">
              Kepçe dışındaki sitelere giderken uyarı göster
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="settings-external-link-toggle"
              class="c-input-hidden"
              bind:checked={externalLinkWarning}
              onchange={handleExternalLinkWarningToggle}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>

        <!-- İstatistik tablosunda gizlen -->
        <label class="c-list-row c-list-row--clickable c-list-row--tall">
          <div class="c-list-row__info">
            <div class="c-list-row__title">İstatistikte gizlen</div>
            <div class="c-list-row__desc">
              Kullanıcı adı yerine "Anonim" yaz
            </div>
          </div>
          <div class="c-list-row__control">
            <input
              type="checkbox"
              id="settings-opt-out-toggle"
              class="c-input-hidden"
              bind:checked={globalState.user.opt_out_statistics}
              onchange={handleOptOutChange}
            />
            <span class="c-switch"><span class="c-switch__handle"></span></span>
          </div>
        </label>
      </div>
    </section>
  {/if}

  {#if globalState.isApp}
    <section class="settings-section" id="settings-links">
      <h2 class="settings-section__heading">Bağlantılar</h2>

      <h3>Keşfet</h3>
      <div class="c-boxed-list">
        <a
          href="/menu-gonder"
          class="c-list-row c-list-row--clickable c-list-row--regular"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Menü gönder</div>
          </div>
          <div class="c-list-row__control u-color-muted">
            {@html icon("arrow-right", 20)}
          </div>
        </a>
        <a
          href="/sss"
          class="c-list-row c-list-row--clickable c-list-row--regular"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Sıkça sorulan sorular</div>
          </div>
          <div class="c-list-row__control u-color-muted">
            {@html icon("arrow-right", 20)}
          </div>
        </a>
        <a
          href="/istatistikler"
          class="c-list-row c-list-row--clickable c-list-row--regular"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">İstatistikler</div>
          </div>
          <div class="c-list-row__control u-color-muted">
            {@html icon("arrow-right", 20)}
          </div>
        </a>
        <a
          href="/moderasyon/altyapi/sistem-sagligi"
          class="c-list-row c-list-row--clickable c-list-row--regular"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Sistem durumu</div>
          </div>
          <div class="c-list-row__control u-color-muted">
            {@html icon("arrow-right", 20)}
          </div>
        </a>
        <a
          href="/rss.xml"
          class="c-list-row c-list-row--clickable c-list-row--regular"
          data-sveltekit-reload
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">RSS akışı</div>
          </div>
          <div class="c-list-row__control u-color-muted">
            {@html icon("arrow-right", 20)}
          </div>
        </a>
      </div>

      <h3>Topluluk</h3>
      <div class="c-boxed-list">
        <a
          href="https://github.com/koder-cog/kepce"
          target="_blank"
          rel="noopener noreferrer"
          class="c-list-row c-list-row--clickable c-list-row--regular"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Kaynak kodu</div>
          </div>
          <div class="c-list-row__control u-color-muted">
            {@html icon("external-link", 20)}
          </div>
        </a>
        <a
          href="https://instagram.com/kepceorg"
          target="_blank"
          rel="noopener noreferrer"
          class="c-list-row c-list-row--clickable c-list-row--regular"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Instagram</div>
          </div>
          <div class="c-list-row__control u-color-muted">
            {@html icon("external-link", 20)}
          </div>
        </a>
        <a
          href="https://x.com/kepceorg"
          target="_blank"
          rel="noopener noreferrer"
          class="c-list-row c-list-row--clickable c-list-row--regular"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Twitter</div>
          </div>
          <div class="c-list-row__control u-color-muted">
            {@html icon("external-link", 20)}
          </div>
        </a>
        <a
          href="https://reddit.com/r/kepce"
          target="_blank"
          rel="noopener noreferrer"
          class="c-list-row c-list-row--clickable c-list-row--regular"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Subreddit</div>
          </div>
          <div class="c-list-row__control">
            {@html icon("external-link", 20)}
          </div>
        </a>
      </div>

      <h3>Yasal</h3>
      <div class="c-boxed-list">
        <a
          href="/iletisim"
          class="c-list-row c-list-row--clickable c-list-row--regular"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Künye</div>
          </div>
          <div class="c-list-row__control u-color-muted">
            {@html icon("arrow-right", 20)}
          </div>
        </a>
        <a
          href="/yasal/gizlilik"
          class="c-list-row c-list-row--clickable c-list-row--regular"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Gizlilik politikası</div>
          </div>
          <div class="c-list-row__control u-color-muted">
            {@html icon("arrow-right", 20)}
          </div>
        </a>
        <a
          href="/yasal/kullanim"
          class="c-list-row c-list-row--clickable c-list-row--regular"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Kullanım koşulları</div>
          </div>
          <div class="c-list-row__control u-color-muted">
            {@html icon("arrow-right", 20)}
          </div>
        </a>
      </div>
    </section>
  {/if}
  {#if globalState.user}
    <section class="settings-section" id="settings-danger">
      <h2 class="settings-section__heading u-color-accent-negative">Tehlike</h2>
      <div class="c-boxed-list">
        {#if globalState.isApp}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="c-list-row c-list-row--clickable c-list-row--regular"
            onclick={() => authActions.logout()}
          >
            <div class="c-list-row__info">
              <div class="c-list-row__title u-color-accent-negative">
                Çıkış yap
              </div>
            </div>
            <div class="c-list-row__control u-color-muted">
              {@html icon("log-out", 20)}
            </div>
          </div>
        {/if}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="c-list-row c-list-row--clickable c-list-row--regular"
          onclick={deleteAccount}
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title u-color-accent-negative">
              Hesabını sil
            </div>
          </div>
          <div class="c-list-row__control">
            <button class="btn btn--danger btn--squish" onclick={deleteAccount}
              >Hesabı sil</button
            >
          </div>
        </div>
      </div>
    </section>
  {/if}
</div>
