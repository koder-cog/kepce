<script>
  import "@/styles/pages/_content.css";
  import "@/styles/pages/_auth.css";
  import { globalState, authActions } from "@/state.svelte.js";

  import { API_BASE } from "@/api/client.js";
  import { icon } from "@/components/ui/icons.js";
  import { getCurrentCity } from "@/stores/city.svelte.js";
  import { showToast } from "@/components/ui/toast.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import { initCharCounter } from "@/utils/char-counter.js";
  import { onMount, tick } from "svelte";
  import { slide } from "svelte/transition";
  import { getDuration } from "@/lib/dom/motion.js";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import Modal from "@/components/ui/Modal.svelte";
  import { CITY_MAP } from "@/utils/turkish.js";
  import Seo from "@/components/ui/Seo.svelte";

  import TabBar from "@/components/ui/TabBar.svelte";

  const MONTHS = [
    "Ocak",
    "Şubat",
    "Mart",
    "Nisan",
    "Mayıs",
    "Haziran",
    "Temmuz",
    "Ağustos",
    "Eylül",
    "Ekim",
    "Kasım",
    "Aralık",
  ];

  const MAX_FILES = 5;

  const tabs = [
    { id: "menu", label: "Aylık Menü", icon: icon("calendar", 18) },
    { id: "al-gotur", label: "Al Götür", icon: icon("takeaway", 18) || icon("box", 18) },
    { id: "fiyat-listesi", label: "Fiyat Listesi", icon: icon("tag", 18) },
  ];

  let contributionType = $state("menu");

  function handleTypeChange(val) {
    contributionType = val;
    if (typeof window !== "undefined") {
      const url = new URL(window.location.href);
      if (val === "menu") {
        url.searchParams.delete("tur");
      } else {
        url.searchParams.set("tur", val);
      }
      window.history.replaceState({}, "", url.toString());
    }
  }

  let typeMeta = $derived.by(() => {
    if (contributionType === "al-gotur") {
      return {
        title: "Menü Gönder",
        subtitle:
          "Yurdunda verilen Al Götür kahvaltı/öğün paketlerinin içeriğini veya fotoğraflarını paylaş.",
        fileLabel: "Al Götür Belgesi veya Fotoğrafı",
        fileHint: ".png, .jpg, .pdf veya .xlsx (Dosya başı maks 10MB)",
      };
    }
    if (contributionType === "fiyat-listesi") {
      return {
        title: "Menü Gönder",
        subtitle:
          "Yurt kantininde asılı olan resmi tavan fiyat listesinin fotoğrafını veya tablosunu paylaş.",
        fileLabel: "Kantin Fiyat Panosu veya Belgesi",
        fileHint: ".png, .jpg, .pdf veya .xlsx (Dosya başı maks 10MB)",
      };
    }
    return {
      title: "Menü Gönder",
      subtitle:
        "Yurdunun yemek listesini paylaş, arkadaşlarının cebi rahat etsin.",
      fileLabel: "Menü Dosyası (Excel, PDF veya Resim)",
      fileHint: ".xlsx, .xls, .pdf, .png veya .jpg (Dosya başı maks 10MB)",
    };
  });

  let user = $derived(globalState?.user);
  let now = new Date();

  // ── Tarih mantığı: güncel ay + 1 ──
  const maxDate = new Date(now.getFullYear(), now.getMonth() + 1, 1);
  const maxYear = maxDate.getFullYear();
  const maxMonthCeiling = maxDate.getMonth() + 1; // 1-indexed

  let selectedCity = $state(getCurrentCity() || "istanbul");
  let selectedYear = $state(String(now.getFullYear()));
  let selectedMonth = $state(String(now.getMonth() + 1));
  let selectedFiles = $state([]);
  let notes = $state("");

  let isSubmitting = $state(false);
  let isSuccess = $state(false);
  let submitError = $state(null);
  let showEmailFallbackModal = $state(false);

  let hasCityError = $state(false);
  let hasFileError = $state(false);

  let isDragOver = $state(false);

  let fileInput = $state();
  let notesTextarea = $state();

  let cityOptions = Object.entries(CITY_MAP)
    .map(([slug, name]) => ({ value: slug, label: name }))
    .sort((a, b) => a.label.localeCompare(b.label, "tr"));

  let years = [];
  for (let y = 2026; y <= maxYear; y++) {
    years.push({ value: String(y), label: String(y) });
  }

  let monthOptions = $derived.by(() => {
    const selYear = parseInt(selectedYear);
    let ceiling;
    if (selYear === maxYear) {
      ceiling = maxMonthCeiling;
    } else if (selYear < maxYear) {
      ceiling = 12;
    } else {
      ceiling = 0; // Normalde buraya düşmemeli
    }
    return MONTHS.slice(0, Math.min(12, ceiling)).map((m, i) => ({
      value: String(i + 1),
      label: m,
    }));
  });

  $effect(() => {
    if (parseInt(selectedMonth) > monthOptions.length) {
      selectedMonth = String(monthOptions.length);
    }
  });

  onMount(() => {
    try {
      const params = new URLSearchParams(window.location.search);
      const turParam = params.get("tur");
      if (
        turParam &&
        ["menu", "al-gotur", "fiyat-listesi"].includes(turParam)
      ) {
        contributionType = turParam;
      }
    } catch {}

    if (notesTextarea) {
      initCharCounter(notesTextarea);
    }
  });

  function handleFileSelect(files) {
    const fileArray = Array.from(files);

    if (selectedFiles.length + fileArray.length > MAX_FILES) {
      showToast(`En fazla ${MAX_FILES} dosya gönderebilirsin.`, "error");
      return;
    }

    const validExts = ["xlsx", "xls", "pdf", "png", "jpg", "jpeg"];

    for (const file of fileArray) {
      const ext = file.name.split(".").pop().toLowerCase();
      if (!validExts.includes(ext)) {
        showToast(`${file.name}: Geçersiz format.`, "error");
        continue;
      }
      if (file.size > 10 * 1024 * 1024) {
        showToast(`${file.name}: Dosya çok büyük (Maks 10MB).`, "error");
        continue;
      }

      if (
        !selectedFiles.some((f) => f.name === file.name && f.size === file.size)
      ) {
        selectedFiles = [...selectedFiles, file];
      }
    }

    if (selectedFiles.length > 0) {
      hasFileError = false;
    }
  }

  function handleFileRemove(fileToRemove) {
    selectedFiles = selectedFiles.filter((f) => f !== fileToRemove);
  }

  function handleDragOver(e) {
    e.preventDefault();
    isDragOver = true;
  }

  function handleDragLeave() {
    isDragOver = false;
  }

  function handleDrop(e) {
    e.preventDefault();
    isDragOver = false;
    handleFileSelect(e.dataTransfer.files);
  }

  async function handleSubmit(e) {
    e.preventDefault();

    submitError = null;
    hasCityError = false;
    hasFileError = false;

    let error = false;
    if (!selectedCity) {
      hasCityError = true;
      error = true;
    }
    if (!selectedFiles.length) {
      hasFileError = true;
      error = true;
    }

    if (error) return;

    isSubmitting = true;

    try {
      const formData = new FormData();
      formData.append("city_slug", selectedCity);
      formData.append("year", selectedYear);
      formData.append("month", selectedMonth);
      formData.append("category", contributionType);

      const typeLabel =
        contributionType === "al-gotur"
          ? "Al Götür Menüsü"
          : contributionType === "fiyat-listesi"
            ? "Kantin Fiyat Listesi"
            : "Aylık Menü";
      const finalNotes = notes.trim()
        ? `[${typeLabel}] ${notes.trim()}`
        : `[${typeLabel}]`;
      formData.append("notes", finalNotes);

      for (const file of selectedFiles) {
        formData.append("files", file);
      }

      const res = await fetch(`${API_BASE}/ingestion/submit`, {
        method: "POST",
        credentials: "include",
        body: formData,
      });

      if (!res.ok) {
        const body = await res.text().catch(() => "");
        let detail;
        try {
          detail = JSON.parse(body)?.detail;
        } catch (_) {}
        throw new Error(detail || body || `Hata oluştu (Durum: ${res.status})`);
      }

      isSuccess = true;
    } catch (err) {
      submitError = err.message || "Bir hata oluştu.";
    } finally {
      isSubmitting = false;
    }
  }
</script>

<Seo
  title={`${typeMeta.title} | Kepçe`}
  description={typeMeta.subtitle}
  image="https://kepce.org/api/v1/public/og/page/menu-gonder"
/>

{#if isSuccess}
  <div class="content-page__body contribution-page-body">
    <EmptyState
      iconName={"check"}
      title={"Gönderim Başarılı"}
      desc={"Gönderdiğiniz dosya ekibimizce incelenip kısa sürede sisteme işlenecektir. Katkınız için teşekkürler!"}
    >
      <a href="/" data-link class="btn btn--primary">Ana sayfaya dön</a>
    </EmptyState>
  </div>
{:else}
  <div class="content-page__header contribution-header">
    <h1 class="content-page__title">Menü Gönder</h1>
    <div class="content-page__date">
      {#if user}
        {typeMeta.subtitle}
      {:else}
        <span class="u-color-disclaimer u-font-bold"
          >Giriş yapmadığın için bu katkı veri tabanına anonim olarak iletilecektir.</span
        >
      {/if}
    </div>

    <div class="form-footer-hint">
      <span class="form-required-mark">*</span>: Zorunlu
    </div>
  </div>

  <div class="content-page__body contribution-page-body">
    <!-- Katkı Türü Sekmeleri (Form Üstü) -->
    <div class="u-mb-lg u-w-full" style="max-width: 760px; margin-left: auto; margin-right: auto;">
      <TabBar
        {tabs}
        bind:activeId={contributionType}
        onChange={handleTypeChange}
      />
    </div>

    <form class="card contribution-form" onsubmit={handleSubmit}>
      <!-- Ana Form Alanları -->
      <div
        class="form-group {hasCityError ? 'form-group--error' : ''}"
        data-error={hasCityError ? "Lütfen bir şehir seçiniz." : ""}
      >
        <label class="form-label" for="city-select"
          >Şehir <span class="form-required-mark">*</span></label
        >
        <Dropdown
          options={cityOptions}
          bind:value={selectedCity}
          placeholder="Şehir seçiniz"
        />
      </div>

      <div class="form-row grid-cols-2">
        <div class="form-group">
          <label class="form-label" for="year-select"
            >Yıl <span class="form-required-mark">*</span></label
          >
          <Dropdown
            options={years}
            bind:value={selectedYear}
            placeholder="Yıl seçiniz"
          />
        </div>
        <div class="form-group">
          <label class="form-label" for="month-select"
            >Ay <span class="form-required-mark">*</span></label
          >
          <Dropdown
            options={monthOptions}
            bind:value={selectedMonth}
            placeholder="Ay seçiniz"
          />
        </div>
      </div>

      <div
        class="form-group {hasFileError ? 'form-group--error' : ''}"
        data-error={hasFileError
          ? "Lütfen en az bir dosya yükleyiniz."
          : ""}
      >
        <label class="form-label" for="file-input"
          >{typeMeta.fileLabel} <span class="form-required-mark"
            >*</span
          ></label
        >

        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="file-drop-zone {isDragOver
            ? 'u-border-primary'
            : selectedFiles.length > 0
              ? 'u-border-accent-primary'
              : ''}"
          role="button"
          tabindex="0"
          onclick={() => fileInput.click()}
          onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              fileInput.click();
            }
          }}
          ondragover={handleDragOver}
          ondragleave={handleDragLeave}
          ondrop={handleDrop}
        >
          <div class="file-drop-zone__icon u-mb-sm">
            {@html icon("upload", 32)}
          </div>
          <div class="u-text-sm u-font-bold">
            Dosyayı buraya fırlat veya seç
          </div>
          <div class="u-text-xs u-color-muted u-mt-xs">
            {typeMeta.fileHint}
          </div>
          <input
            type="file"
            id="file-input"
            aria-label="{typeMeta.fileLabel}"
            bind:this={fileInput}
            accept=".xlsx,.xls,.pdf,image/*"
            class="u-hidden"
            multiple
            onchange={(e) => handleFileSelect(e.target.files)}
          />
        </div>

        <div class="c-file-list u-mt-xs">
          {#each selectedFiles as file (file.name + "-" + file.size)}
            <div
              class="c-file-item"
              transition:slide={{ duration: getDuration(200) }}
            >
              <div class="c-file-item__info">
                <div class="c-file-item__icon">
                  {@html icon("attach", 16)}
                </div>
                <span class="c-file-item__name">{file.name}</span>
              </div>
              <button
                type="button"
                class="c-file-item__remove"
                title="Kaldır"
                onclick={(e) => {
                  e.stopPropagation();
                  handleFileRemove(file);
                }}
              >
                {@html icon("close", 14)}
              </button>
            </div>
          {/each}
        </div>
      </div>

      <div class="form-group">
        <label class="form-label" for="notes-textarea">Notlar / Açıklama</label>
        <textarea
          bind:this={notesTextarea}
          bind:value={notes}
          id="notes-textarea"
          name="notes"
          placeholder=""
          rows="4"
          maxlength="1024"
          class="contribution-notes-area"
        ></textarea>
      </div>

      {#if submitError}
        <div class="auth-error u-block">{submitError}</div>
      {/if}

      <button
        type="submit"
        class="btn btn--primary btn--large u-w-full btn--squish"
        disabled={isSubmitting}
      >
        {isSubmitting ? "Gönderiliyor..." : "Gönderimi Tamamla"}
      </button>
      <div class="form-footer__links">
        <button
          type="button"
          class="text-link"
          onclick={() => (showEmailFallbackModal = true)}
        >
          Menüyü gönderemiyor musunuz?
        </button>
      </div>
      <!-- E-posta Fallback -->
    </form>
  </div>
{/if}

{#if showEmailFallbackModal}
  <Modal
    options={{ title: "E-posta ile Gönder" }}
    onClose={() => (showEmailFallbackModal = false)}
  >
    {#snippet children()}
      <div class="u-text-sm u-color-muted">
        <p class="u-mb-sm">
          Eğer formu kullanamıyorsanız menüyü doğrudan e-posta adresimize
          iletebilirsiniz.
        </p>
        <p>
          Göndereceğiniz menüler manuel olarak incelenecektir. Lütfen menü
          dosyasını e-postaya eklemeyi unutmayın.
        </p>
      </div>
    {/snippet}

    {#snippet footer()}
      {@const subject = encodeURIComponent(
        `[Menü] ${selectedCity || "Belirtilmemiş"} - ${selectedMonth}/${selectedYear}`,
      )}
      {@const body = encodeURIComponent(
        `Merhaba,\n\nMenü dosyası ektedir.\n\nŞehir: ${selectedCity || ""}\nDönem: ${selectedMonth}/${selectedYear}\n${user?.username ? `Kullanıcı Adı (Opsiyonel): ${user.username}` : ""}\n\n(Varsa eklemek istediğiniz notlar...)`,
      )}
      <button
        type="button"
        class="btn btn--secondary"
        onclick={() => (showEmailFallbackModal = false)}>Vazgeç</button
      >
      <a
        class="btn btn--primary"
        href="mailto:menugonder@kepce.org?subject={subject}&body={body}"
      >
        E-posta Gönder
      </a>
    {/snippet}
  </Modal>
{/if}
