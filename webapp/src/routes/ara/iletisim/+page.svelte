<script>
  import { page } from "$app/stores";
  import { icon } from "@/components/ui/icons.js";
  import "@/styles/pages/_content.css";
  import "@/styles/pages/_auth.css";
  import { api } from "@/api/index.js";
  import { showToast } from "@/components/ui/toast.js";
  import Dropdown from "@/components/features/Dropdown.svelte";

  const MAX_DESC = 2000;
  const MAX_SUBJ = 150;

  let isSubdomain = $derived(
    $page.url.hostname.startsWith("ara.") ||
      $page.url.hostname === "ara.localhost",
  );
  let basePath = $derived(isSubdomain ? "" : "/ara");

  let email = $state("");
  let category = $state("error");
  let subject = $state("");
  let description = $state("");

  let isLoading = $state(false);
  let errorMsg = $state("");
  let errors = $state({});

  let descLength = $derived(description.length);
  let subjLength = $derived(subject.length);

  async function handleSubmit(e) {
    e.preventDefault();
    errors = {};
    errorMsg = "";

    if (!email.includes("@")) {
      errors.email = "Geçerli bir e-posta giriniz.";
      return;
    }
    if (!subject.trim()) {
      errors.subject = "Konu alanı zorunludur.";
      return;
    }
    if (subject.length > MAX_SUBJ) {
      errors.subject = `Konu en fazla ${MAX_SUBJ} karakter olabilir.`;
      return;
    }
    if (category === "other" && !description.trim()) {
      errors.description = "Lütfen diğer kategorisi için bir açıklama giriniz.";
      return;
    }
    if (description.length > MAX_DESC) {
      errors.description = `Açıklama çok uzun (maks. ${MAX_DESC} karakter).`;
      return;
    }

    isLoading = true;

    try {
      await api.submitContactForm({
        email,
        report_type: category,
        subject: subject.trim(),
        description: description.trim(),
        source: "ara",
        page_url: typeof window !== "undefined" ? window.location.href : null,
      });
      showToast("Teşekkürler! Mesajınız başarıyla iletildi.");

      subject = "";
      description = "";
      category = "error";
    } catch (err) {
      errorMsg = err.message || "Bir hata oluştu.";
    } finally {
      isLoading = false;
    }
  }
</script>

<svelte:head>
  <title>İletişim | Kepçe Ara</title>
  <meta
    name="description"
    content="Kepçe Ara iletişim kanalları, 5651 sayılı Kanun kapsamında Uyar-Kaldır başvuruları ve hata bildirim formu."
  />
</svelte:head>

<div class="settings-page c-search-adv-page">
  <article class="content-page">
    <header class="content-page__header">
      <h1 class="content-page__title">İletişim</h1>
      <div class="content-page__meta">
        <time class="content-page__date">Kepçe Açık Kaynak Topluluğu</time>
      </div>
    </header>

    <div class="content-page__body">
      <section>
        <h2>Dikkat edilecek hususlar</h2>
        <ol>
          <li>
            Kepçe Ara bir meta arama aracıdır. Arama sonuçlarında listelenen
            sayfaların içeriği ilgili web sitelerine aittir.
          </li>
          <li>
            5651 sayılı Kanun m. 5/2 kapsamında hukuka aykırı veya kişilik
            haklarını ihlal eden bağlantılar için "Yasal / Uyar-Kaldır"
            kategorisini seçebilirsiniz. Bildiriminizde ihlale konu bağlantı
            adresini (URL) ve yasal gerekçenizi belirtiniz.
          </li>
          <li>
            Arama motoru arayüzü veya sonuç listelemeyle ilgili teknik
            aksaklıkları "Hata Bildirimi" kategorisi üzerinden iletebilirsiniz.
          </li>
        </ol>
      </section>

      <section class="contact-form-section">
        <div class="contact-form-card">
          {#if errorMsg}
            <div id="contact-error-container">
              <div class="auth-error" role="alert">{errorMsg}</div>
            </div>
          {/if}

          <form id="contact-form" onsubmit={handleSubmit}>
            <div
              class="form-group form-group--floating"
              class:form-group--error={errors.email}
              data-error={errors.email}
            >
              <input
                type="email"
                id="contact-email"
                bind:value={email}
                required
                placeholder=" "
              />
              <label class="form-label" for="contact-email"
                >E-posta Adresiniz <span class="form-required-mark">*</span
                ></label
              >
            </div>

            <div class="form-group">
              <div class="form-label">
                Kategori <span class="form-required-mark">*</span>
              </div>
              <Dropdown
                options={[
                  { value: "error", label: "Hata Bildirimi" },
                  { value: "legal", label: "Yasal / Uyar-Kaldır (5651)" },
                  { value: "suggest", label: "Öneri / Geri Bildirim" },
                  { value: "other", label: "Diğer" },
                ]}
                bind:value={category}
              />
            </div>

            <div
              class="form-group form-group--floating"
              class:form-group--error={errors.subject}
              data-error={errors.subject}
            >
              <input
                type="text"
                id="contact-subject"
                bind:value={subject}
                required
                maxlength={MAX_SUBJ}
                placeholder=" "
              />
              <label class="form-label" for="contact-subject"
                >Konu <span class="form-required-mark">*</span></label
              >
            </div>
            <span
              class="c-char-counter"
              class:c-char-counter--over={subjLength > MAX_SUBJ}
              id="contact-subject-char-counter"
            >
              {subjLength} / {MAX_SUBJ}
            </span>

            <div
              class="form-group form-group--floating"
              class:form-group--error={errors.description}
              data-error={errors.description}
            >
              <textarea
                id="contact-description"
                bind:value={description}
                placeholder=" "
                rows="6"
              ></textarea>
              <label class="form-label" for="contact-description"
                >Açıklama</label
              >
            </div>
            <span
              class="c-char-counter"
              class:c-char-counter--over={descLength > MAX_DESC}
              id="contact-char-counter"
            >
              {descLength} / {MAX_DESC}
            </span>

            <button
              type="submit"
              class="btn btn--primary btn--squish"
              id="contact-submit"
              disabled={isLoading}
            >
              {isLoading ? "Gönderiliyor…" : "Gönder"}
            </button>
          </form>
        </div>
      </section>

      <section class="contact-info-section">
        <h2>Proje bilgileri</h2>
        <p>
          5651 sayılı Kanun uyarınca yer sağlayıcıya ilişkin tanıtıcı bilgiler
          aşağıdadır:
          <br /><br />
          <strong>Yer Sağlayıcı:</strong> Kazım Geleş<br />
          <strong>E-posta:</strong>
          <a href="mailto:yasal@kepce.org">yasal@kepce.org</a><br />
          <strong>İletişim:</strong> Yukarıdaki form veya e-posta adresi üzerinden
          resmi başvurularınızı iletebilirsiniz.
        </p>
      </section>

      <section class="u-mt-xl">
        <h2>Açık kaynak depoları</h2>
        <p>
          Kepçe Ara yazılımı ve SearXNG altyapısıyla ilgili doğrudan kod katkısı
          veya geliştirici tartışmaları için aşağıdaki açık depoları da
          kullanabilirsiniz:
        </p>
        <ul>
          <li>
            <a
              href="https://github.com/koder-cog/kepce"
              target="_blank"
              rel="noopener noreferrer">Kepçe GitHub Deposu</a
            >
          </li>
          <li>
            <a
              href="https://github.com/searxng/searxng"
              target="_blank"
              rel="noopener noreferrer">SearXNG GitHub Deposu</a
            >
          </li>
        </ul>
      </section>
    </div>
  </article>

  <!-- Alt Eylem Butonu -->
  <div class="c-search-adv-footer-actions u-mt-xl">
    <a href={basePath || "/"} class="btn btn--secondary btn--squish">
      {@html icon("chevronLeft", 16)}
      <span>Aramaya Dön</span>
    </a>
  </div>
</div>
