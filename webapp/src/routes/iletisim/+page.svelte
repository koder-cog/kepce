<script>
  import { api } from "@/api/index.js";
  import { showToast } from "@/components/ui/toast.js";
  import Dropdown from "@/components/features/Dropdown.svelte";

  const MAX_DESC = 2000;
  const MAX_SUBJ = 128;

  let email = $state("");
  let category = $state("error");
  let subject = $state("");
  let description = $state("");

  let isLoading = $state(false);
  let errorMsg = $state("");
  let errors = $state({});

  let descLength = $derived(description.length);

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
      });
      showToast("Teşekkürler! Mesajınız başarıyla iletildi.");

      // Reset form
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
  <title>İletişim - Kepçe</title>
</svelte:head>

<div class="content-page">
  <div class="content-page__header">
    <h1 class="content-page__title">İletişim</h1>
  </div>
  <div class="content-page__body">
    <section>
      <h2>Dikkat edilecek hususlar</h2>
      <ol>
        <li>
          İletişime geçmeden önce sorunuzun <strong
            ><a href="/sss" data-link>Sıkça Sorulabilecek Sorular</a></strong
          > sayfasında cevaplanıp cevaplanmadığını kontrol edin.
        </li>
        <li>
          Sistemle ilgili teknik hataları "Hata Bildir" kategorisini seçerek
          detaylıca iletebilirsiniz.
        </li>
        <li>
          Yasadışı veya hak ihlali içeren içerikler için "Uyar-Kaldır"
          bildirimlerinizi bu form üzerinden veya doğrudan e-posta ile
          yapabilirsiniz.
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
                { value: "legal", label: "Yasal / Uyar-Kaldır" },
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
            <label class="form-label" for="contact-description">Açıklama</label>
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
            class="btn btn--primary"
            id="contact-submit"
            disabled={isLoading}
          >
            {isLoading ? "Gönderiliyor…" : "Gönder"}
          </button>
        </form>
      </div>
    </section>

    <section class="contact-info-section">
      <h2>Şirket / proje bilgileri (künye)</h2>
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
  </div>
</div>
