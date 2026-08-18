<script>
  import "@/styles/pages/_auth.css";
  import { goto } from "$app/navigation";
  import { globalState, authActions } from "@/state.svelte.js";

  import { api } from "@/api/index.js";
  import { showToast } from "@/components/ui/toast.js";
  import { getDuration } from "@/lib/dom/motion.js";
  import * as ui from "@/components/ui/forms.js";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Seo from "@/components/ui/Seo.svelte";
  import { onMount } from "svelte";

  const user = $derived(globalState?.user);

  let isNewOAuth = $state(false);
  let cities = $state([]);
  let currentCitySlug = $state("");
  let currentDiet = $state(false);

  let selectedCity = $state("");
  let isCeliac = $state(false);
  let kvkk = $state(false);
  let terms = $state(false);
  let sensitive = $state(false);
  let abroad = $state(false);

  let errorMsg = $state("");
  let legalError = $state(false);
  let isLoading = $state(false);
  let isInitializing = $state(true);

  let cityOptions = $derived([
    { value: "", label: "Belirtmek istemiyorum" },
    ...cities
      .map((c) => ({ value: c.slug, label: c.name }))
      .sort((a, b) => a.label.localeCompare(b.label, "tr")),
  ]);

  onMount(async () => {
    if (!globalState?.user) {
      goto("/giris");
      return;
    }

    isNewOAuth = sessionStorage.getItem("kepce_is_new_oauth") === "true";

    if (isNewOAuth) {
      cities = await api.getCities();
      currentCitySlug = globalState.user.default_city_slug || "";
      currentDiet = localStorage.getItem("kepce_diet_mode") === "celiac";

      selectedCity = currentCitySlug;
      isCeliac = currentDiet;
    }

    isInitializing = false;
  });

  async function handleSubmit(e) {
    e.preventDefault();
    errorMsg = "";
    legalError = false;

    if (!kvkk || !terms || !sensitive || !abroad) {
      errorMsg = "Devam etmek için tüm onayları vermelisiniz.";
      legalError = true;
      return;
    }

    isLoading = true;

    try {
      if (selectedCity) {
        await api.updateProfile({ default_city_slug: selectedCity });
        if (globalState.user) {
          globalState.user.default_city_slug = selectedCity;
        }
      }

      const mode = isCeliac ? "celiac" : "standard";
      localStorage.setItem("kepce_diet_mode", mode);

      sessionStorage.removeItem("kepce_is_new_oauth");

      showToast("Profilin başarıyla güncellendi. Hoş geldin!", "success");

      setTimeout(() => {
        goto("/");
      }, getDuration(500));
    } catch (err) {
      console.error("Profile update error:", err);
      showToast(
        err.message || "Profil güncellenirken bir hata oluştu.",
        "error",
      );
    } finally {
      isLoading = false;
    }
  }
</script>

<Seo title="Profili Tamamla - Kepçe" noindex={true} />

{#if !isInitializing}
  {#if !isNewOAuth}
    <div class="empty-state-container">
      <EmptyState
        statusCode={403}
        title={"403: Elleşme Ayarlarla"}
        desc={"Bu hesabın profili daha önceden oluşturulmuş. İlla bir şeyleri kurcalayacaksan efendi gibi ayarlar sayfasına geç."}
      >
        <a href="/" data-link class="btn btn--secondary">Ana sayfaya dön</a>
        <a href="/ayarlar" data-link class="btn btn--primary">Ayarlar'a git</a>
      </EmptyState>
    </div>
  {:else}
    <h2 class="auth-page__title u-mb-xs">Profilini Tamamla</h2>
    <div
      class="form-footer-hint u-mb-xl u-text-sm u-weight-semibold u-color-secondary"
    >
      Kepçe'ye giriş yaptın ama halledilmesi gereken bağzı şeyler var.
    </div>

    <div class="auth-form-container">
      <form class="auth-form" onsubmit={handleSubmit}>
        <div class="form-group">
          <!-- svelte-ignore a11y_label_has_associated_control -->
          <label class="form-label u-mb-xs">Varsayılan Şehrin</label>
          <div class="dropdown-form-control">
            <Dropdown options={cityOptions} bind:value={selectedCity} />
          </div>
          <div class="form-help">
            Eğer bulunulan şehir için menü yoksa varsayılan olarak seçtiğin
            şehir gösterilir.
          </div>
        </div>

        <div class="form-group">
          <!-- svelte-ignore a11y_label_has_associated_control -->
          <label class="form-label">Diyet Tercihleri</label>
          <div class="c-boxed-list">
            <div class="c-list-row">
              <div class="c-list-row__content">
                <span class="c-list-row__title">Çölyak</span>
                <span class="c-list-row__desc"
                  >Glutensiz seçenekleri ve çapraz bulaşma uyarılarını göster.</span
                >
              </div>
              <div class="c-list-row__control">
                <div class="c-switch">
                  <input
                    type="checkbox"
                    bind:checked={isCeliac}
                    id="profile-diet"
                    class="c-input-hidden"
                  />
                  <div class="c-switch__handle"></div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="register-submit-group">
          <div
            class="legal-consents"
            class:form-group--error={legalError}
            data-error={errorMsg}
          >
            <label class="form-switch-row">
              <input
                type="checkbox"
                class="c-input-hidden"
                bind:checked={kvkk}
              />
              <div class="c-switch"><div class="c-switch__handle"></div></div>
              <span class="form-switch-row__text">
                <a href="/gizlilik-politikasi" target="_blank"
                  >Gizlilik Politikası</a
                >nı (KVKK Aydınlatma Metni) okudum.
              </span>
            </label>
            <label class="form-switch-row">
              <input
                type="checkbox"
                class="c-input-hidden"
                bind:checked={terms}
              />
              <div class="c-switch"><div class="c-switch__handle"></div></div>
              <span class="form-switch-row__text">
                <a href="/kullanim-kosullari" target="_blank"
                  >Kullanım Koşulları</a
                >nı kabul ediyorum.
              </span>
            </label>
            <label class="form-switch-row">
              <input
                type="checkbox"
                class="c-input-hidden"
                bind:checked={sensitive}
              />
              <div class="c-switch"><div class="c-switch__handle"></div></div>
              <span class="form-switch-row__text">
                Diyet tercihlerimin (özel nitelikli veri) işlenmesine açık rıza
                veriyorum.
              </span>
            </label>
            <label class="form-switch-row">
              <input
                type="checkbox"
                class="c-input-hidden"
                bind:checked={abroad}
              />
              <div class="c-switch"><div class="c-switch__handle"></div></div>
              <span class="form-switch-row__text">
                Kişisel verilerimin, KVKK Madde 9 uyarınca sunucuların bulunduğu
                yurt dışına (Marsilya/Fransa) aktarılmasına açık rıza veriyorum.
              </span>
            </label>
          </div>

          <button
            type="submit"
            class="btn btn--primary btn--large auth-submit u-mt-lg"
            disabled={isLoading}
            class:is-loading={isLoading}
          >
            Kayıt ol işte böyle
          </button>
        </div>
      </form>
    </div>
  {/if}
{/if}
