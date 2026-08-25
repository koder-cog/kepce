<script>
  import "@/styles/pages/_developer.css";
  import { globalState, authActions } from "@/state.svelte.js";

  import { createModal } from "@/components/features/modal.js";
  import { showToast } from "@/components/ui/toast.js";
  import { api } from "@/api/index.js";
  import { icon } from "@/components/ui/icons.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import Modal from "@/components/ui/Modal.svelte";
  import { initCharCounter } from "@/utils/char-counter.js";
  import { sanitizeText } from "@/utils/sanitize.js";
  import Seo from "@/components/ui/Seo.svelte";
  import { onMount } from "svelte";

  let user = $derived(globalState?.user);

  let projects = $state([]);
  let keys = $state([]);
  let isLoading = $state(true);
  let errorMsg = $state(null);

  let usageProjectFilter = $state("all");
  let usageTimeFilter = $state("28");
  let isCommercialModalOpen = $state(false);

  let limitsProjectFilter = $state("all");
  let limitsTimeFilter = $state("28");

  let projectOptions = $derived([
    { value: "all", label: "Tüm projeler" },
    ...projects.map((p) => ({
      value: String(p.id),
      label: sanitizeText(p.name),
    })),
  ]);

  const timeOptions = [
    { value: "7", label: "Son 7 gün" },
    { value: "28", label: "Son 28 gün" },
    { value: "90", label: "Son 90 gün" },
  ];

  let totalRequests = $state(0);
  let totalErrors = $state(0);
  let peakMinMax = $state(0);
  let peakDayMax = $state(0);

  let sliceDays = $derived(parseInt(usageTimeFilter, 10));
  let limitsSliceDays = $derived(parseInt(limitsTimeFilter, 10));

  let rawUsageData = $state([]);
  let rawLimitsMinData = $state([]);
  let rawLimitsDayData = $state([]);
  let usageToken = 0;
  let limitsToken = 0;

  $effect(() => {
    if (user) {
      loadUsageData(usageProjectFilter);
      loadLimitsData(limitsProjectFilter);
    }
  });

  async function loadUsageData(pid) {
    const token = ++usageToken;
    try {
      const data = await api.getApiUsage(pid, 90);
      if (token !== usageToken) return;
      rawUsageData = data;
    } catch (err) {
      if (token !== usageToken) return;
      console.error(err);
    }
  }

  async function loadLimitsData(pid) {
    const token = ++limitsToken;
    try {
      const res = await api.getApiUsage(pid, 90);
      if (token !== limitsToken) return;
      rawLimitsMinData = res.map((d) => ({
        ...d,
        requests: Math.floor(d.requests / 5),
      }));
      rawLimitsDayData = res;
    } catch (err) {
      if (token !== limitsToken) return;
      console.error(err);
    }
  }

  let usageData = $derived(rawUsageData.slice(-sliceDays));
  let slicedDataMin = $derived(rawLimitsMinData.slice(-limitsSliceDays));
  let slicedDataDay = $derived(rawLimitsDayData.slice(-limitsSliceDays));

  let initialLoadDone = false;
  $effect(() => {
    if (user && !initialLoadDone) {
      initialLoadDone = true;
      loadData();
    }
  });

  async function loadData() {
    isLoading = true;
    errorMsg = null;
    try {
      const [p, k] = await Promise.all([api.getProjects(), api.getApiKeys()]);
      projects = p;
      keys = k;
    } catch (err) {
      errorMsg = err.message || "Veriler yüklenirken bir hata oluştu.";
    } finally {
      isLoading = false;
    }
  }

  function formatDate(dateStr) {
    if (!dateStr) return "";
    const date = new Date(dateStr);
    const yyyy = date.getFullYear();
    const mm = String(date.getMonth() + 1).padStart(2, "0");
    const dd = String(date.getDate()).padStart(2, "0");
    return `${yyyy}.${mm}.${dd}`;
  }

  function handleCreateProject() {
    if (!globalState?.user?.is_verified) {
      showToast(
        "Proje oluşturabilmek için e-postanızı onaylamalısınız.",
        "warning",
      );
      return;
    }
    const modalInstance = createModal({
      title: "Yeni bir proje oluştur",
      contentHtml: `
        <div class="form-group">
          <label class="form-label" for="project-name">Projenin ismi</label>
          <textarea id="project-name" class="form-textarea--resizable" placeholder="Proje ismini buraya yazınız..." maxlength="30"></textarea>
          <span class="form-help">En az 4, en fazla 30 karakter uzunluğunda; yalnızca harf, sayı, tire, tırnak işareti, boşluk ve ünlem işareti kullanılabilir.</span>
        </div>
      `,
      buttons: [
        { label: "Boşver", variant: "secondary" },
        {
          label: "Oluştur",
          variant: "primary",
          onClick: async (modalEl) => {
            const textarea = modalEl.querySelector("#project-name");
            const value = textarea.value.trim();

            if (!value) {
              showToast("Lütfen proje ismi girin.", "warning");
              textarea.focus();
              return false;
            }
            if (value.length < 4 || value.length > 30) {
              showToast(
                "Proje ismi en az 4, en fazla 30 karakter olmalıdır.",
                "warning",
              );
              textarea.focus();
              return false;
            }
            const isValid = /^[a-zA-Z0-9çÇğĞıİöÖşŞüÜ\s\-"'!]+$/.test(value);
            if (!isValid) {
              showToast("Proje ismi geçersiz karakterler içeriyor.", "warning");
              textarea.focus();
              return false;
            }

            try {
              await api.createProject(value);
              showToast(`"${value}" projesi başarıyla oluşturuldu.`, "success");
              await loadData();
              return true;
            } catch (err) {
              showToast(
                err.message || "Proje oluşturulurken bir hata oluştu.",
                "error",
              );
              return false;
            }
          },
        },
      ],
    });

    const modalEl = modalInstance.modal;
    const textarea = modalEl.querySelector("#project-name");
    const submitBtn = modalEl.querySelector(".btn--primary");

    if (submitBtn) submitBtn.disabled = true;

    if (textarea && submitBtn) {
      const VALID_RE = /^[a-zA-Z0-9çÇğĞıİöÖşŞüÜ\s\-"'!]+$/;
      initCharCounter(textarea, {
        onUpdate: (_count, _limit, isOver) => {
          const trimmed = textarea.value.trim();
          const isLengthValid = trimmed.length >= 4 && trimmed.length <= 30;
          const isCharValid = VALID_RE.test(trimmed);
          submitBtn.disabled = isOver || !isLengthValid || !isCharValid;
        },
      });
    }
  }

  let isCreateApiKeyModalOpen = $state(false);
  let newApiKeyState = $state({
    name: "",
    projectId: "",
  });

  function handleCreateApiKey() {
    if (!globalState?.user?.is_verified) {
      showToast(
        "API anahtarı oluşturabilmek için e-postanızı onaylamalısınız.",
        "warning",
      );
      return;
    }
    if (projects.length === 0) {
      showToast(
        "API anahtarı oluşturabilmek için en az bir proje olmalıdır.",
        "warning",
      );
      return;
    }
    newApiKeyState = {
      name: "",
      projectId: projects[0].id,
    };
    isCreateApiKeyModalOpen = true;
  }

  async function submitCreateApiKey() {
    const { name, projectId } = newApiKeyState;
    if (!name.trim()) {
      showToast("Lütfen anahtar için açıklayıcı bir isim girin.", "warning");
      return;
    }

    try {
      const newKeyData = await api.createApiKey(projectId, name.trim());
      createModal({
        title: "API anahtarınız oluşturuldu!",
        contentHtml: `
          <p class="u-text-sm u-opacity-subtle u-mb-md">
            Yeni API anahtarınız başarıyla üretilmiştir. Güvenliğiniz için bu anahtar size <strong>yalnızca bir kez</strong> gösterilecektir. Lütfen hemen kopyalayın!
          </p>
          <div class="card u-p-md u-text-center dev-sunken-card">
            <code class="u-font-mono u-text-base u-user-select-all dev-key-output">${sanitizeText(newKeyData.key || "Hata: Anahtar alınamadı")}</code>
          </div>
        `,
        buttons: [
          {
            label: "Kopyaladım ve anladım",
            variant: "primary",
            onClick: async () => {
              await loadData();
              return true;
            },
          },
        ],
      });
      showToast("API Anahtarı başarıyla oluşturuldu.", "success");
      isCreateApiKeyModalOpen = false;
    } catch (err) {
      showToast(err.message || "Anahtar oluşturulamadı.", "error");
    }
  }

  function openKeysManagementModal(projectId, projectName) {
    const projectKeys = keys.filter((k) => k.project_id === projectId);

    let keysListHtml = "";
    if (projectKeys.length === 0) {
      keysListHtml = `
        <div class="u-p-lg u-text-center u-opacity-dim u-text-sm">
          Bu projeye bağlı aktif bir API anahtarı bulunmamaktadır.
        </div>
      `;
    } else {
      keysListHtml = `
        <div class="u-flex u-flex-col u-gap-sm u-mb-md">
          ${projectKeys
            .map(
              (key) => `
            <div class="card u-p-md u-flex u-flex-justify-between u-flex-align-center dev-key-row">
              <div class="u-flex u-flex-col">
                <span class="u-text-sm u-font-bold">${sanitizeText(key.name || "İsimsiz anahtar")}</span>
                <span class="u-text-xs u-opacity-muted">
                  Prefix: <code>${sanitizeText(key.key_prefix)}</code> | Kademe: <code>${sanitizeText(key.tier === "commercial" ? "ticari" : "bireysel")}</code>
                </span>
              </div>
              <button class="btn btn--secondary btn--sm btn-revoke-modal-key" data-key-id="${key.id}" data-key-name="${sanitizeText(key.name)}">İptal et</button>
            </div>
          `,
            )
            .join("")}
        </div>
      `;
    }

    const modalInstance = createModal({
      title: `"${projectName}" API anahtarları`,
      contentHtml: `
        <div class="u-mb-md u-flex u-flex-justify-between u-flex-align-center">
          <span class="u-text-xs u-opacity-muted">Aktif anahtarlar</span>
          <button class="btn btn--primary btn--sm" id="btn-add-modal-key">Anahtar ekle</button>
        </div>
        <div id="modal-keys-list-container">
          ${keysListHtml}
        </div>
      `,
      buttons: [{ label: "Kapat", variant: "secondary" }],
    });

    const modalEl = modalInstance.modal;

    modalEl.querySelectorAll(".btn-revoke-modal-key").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        const keyId = parseInt(e.currentTarget.dataset.keyId, 10);
        const keyName = e.currentTarget.dataset.keyName;

        createModal({
          title: "Anahtarı iptal et",
          iconHtml: icon("warning", 24),
          contentHtml: `
            <p class="u-text-sm u-opacity-subtle">
              <strong>"${sanitizeText(keyName)}"</strong> API anahtarını kalıcı olarak iptal etmek istediğinizden emin misiniz? Bu anahtarı kullanan tüm entegrasyonlar anında erişim hatası almaya başlayacaktır.
            </p>
          `,
          buttons: [
            { label: "Vazgeç", variant: "secondary" },
            {
              label: "Evet, iptal et",
              variant: "primary",
              onClick: async () => {
                try {
                  await api.revokeApiKey(keyId);
                  showToast("API anahtarı iptal edildi.", "success");
                  modalInstance.close();
                  await loadData();
                  // Reopen modal to show updated list
                  setTimeout(
                    () => openKeysManagementModal(projectId, projectName),
                    100,
                  );
                  return true;
                } catch (err) {
                  showToast(err.message || "Anahtar iptal edilemedi.", "error");
                  return true;
                }
              },
            },
          ],
        });
      });
    });

    const btnAddModalKey = modalEl.querySelector("#btn-add-modal-key");
    if (btnAddModalKey) {
      btnAddModalKey.addEventListener("click", () => {
        createModal({
          title: "Yeni API anahtarı oluştur",
          contentHtml: `
            <div class="form-group form-group--floating">
              <input type="text" id="api-key-name" class="form-input" placeholder=" " maxlength="30" autocomplete="off">
              <label class="form-label" for="api-key-name">Anahtar ismi (Örn: Telegram botu)</label>
            </div>
            <span class="form-help">Bu anahtar ile Kepçe API servislerine projeniz üzerinden erişebilirsiniz.</span>
          `,
          buttons: [
            { label: "Vazgeç", variant: "secondary" },
            {
              label: "Oluştur",
              variant: "primary",
              onClick: async (createKeyModalEl) => {
                const input = createKeyModalEl.querySelector("#api-key-name");
                const name = input.value.trim();
                if (!name) {
                  showToast(
                    "Lütfen anahtar için açıklayıcı bir isim girin.",
                    "warning",
                  );
                  input.focus();
                  return false;
                }

                try {
                  const newKeyData = await api.createApiKey(projectId, name);
                  createModal({
                    title: "API anahtarınız oluşturuldu!",
                    contentHtml: `
                      <p class="u-text-sm u-opacity-subtle u-mb-md">
                        Yeni API anahtarınız başarıyla üretilmiştir. Güvenliğiniz için bu anahtar size <strong>yalnızca bir kez</strong> gösterilecektir. Lütfen hemen kopyalayın!
                      </p>
                      <div class="card u-p-md u-text-center dev-sunken-card">
                        <code class="u-font-mono u-text-base u-user-select-all dev-key-output">${sanitizeText(newKeyData.key || "Hata: Anahtar alınamadı")}</code>
                      </div>
                    `,
                    buttons: [
                      {
                        label: "Kopyaladım ve anladım",
                        variant: "primary",
                        onClick: async () => {
                          modalInstance.close();
                          await loadData();
                          setTimeout(
                            () =>
                              openKeysManagementModal(projectId, projectName),
                            100,
                          );
                          return true;
                        },
                      },
                    ],
                  });
                  showToast("API Anahtarı başarıyla oluşturuldu.", "success");
                  return true;
                } catch (err) {
                  showToast(err.message || "Anahtar oluşturulamadı.", "error");
                  return false;
                }
              },
            },
          ],
        });
      });
    }
  }

  function handleEditProject(projectId, projectName) {
    const modalInstance = createModal({
      title: "Projeyi düzenle",
      contentHtml: `
        <div class="form-group form-group--floating">
          <input type="text" id="edit-project-name" class="form-input" placeholder=" " maxlength="30" value="${sanitizeText(projectName)}">
          <label class="form-label" for="edit-project-name">Projenin yeni ismi</label>
        </div>
        <span class="form-help">En az 4, en fazla 30 karakter uzunluğunda; yalnızca harf, sayı, tire, tırnak işareti, boşluk ve ünlem işareti kullanılabilir.</span>
      `,
      buttons: [
        { label: "Vazgeç", variant: "secondary" },
        {
          label: "Güncelle",
          variant: "primary",
          onClick: async (modalEl) => {
            const textarea = modalEl.querySelector("#edit-project-name");
            const value = textarea.value.trim();

            if (!value) {
              showToast("Lütfen proje ismi girin.", "warning");
              textarea.focus();
              return false;
            }
            if (value.length < 4 || value.length > 30) {
              showToast(
                "Proje ismi en az 4, en fazla 30 karakter olmalıdır.",
                "warning",
              );
              textarea.focus();
              return false;
            }
            const isValid = /^[a-zA-Z0-9çÇğĞıİöÖşŞüÜ\s\-"'!]+$/.test(value);
            if (!isValid) {
              showToast("Proje ismi geçersiz karakterler içeriyor.", "warning");
              textarea.focus();
              return false;
            }
            try {
              await api.updateProject(projectId, value);
              showToast("Proje ismi başarıyla güncellendi.", "success");
              await loadData();
              return true;
            } catch (err) {
              showToast(err.message || "Proje güncellenemedi.", "error");
              return false;
            }
          },
        },
      ],
    });

    const modalEl = modalInstance.modal;
    const textarea = modalEl.querySelector("#edit-project-name");
    const submitBtn = modalEl.querySelector(".btn--primary");

    if (textarea && submitBtn) {
      textarea.focus();
      textarea.setSelectionRange(textarea.value.length, textarea.value.length);
      const VALID_RE = /^[a-zA-Z0-9çÇğĞıİöÖşŞüÜ\s\-"'!]+$/;
      initCharCounter(textarea, {
        onUpdate: (_count, _limit, isOver) => {
          const trimmed = textarea.value.trim();
          const isLengthValid = trimmed.length >= 4 && trimmed.length <= 30;
          const isCharValid = VALID_RE.test(trimmed);
          submitBtn.disabled = isOver || !isLengthValid || !isCharValid;
        },
      });
    }
  }

  function handleDeleteProject(projectId, projectName) {
    createModal({
      title: "Projeyi sil",
      iconHtml: icon("warning", 24),
      contentHtml: `
        <p class="u-text-sm u-opacity-subtle">
          <strong>"${sanitizeText(projectName)}"</strong> projesini kalıcı olarak silmek istediğinizden emin misiniz? Projeyle birlikte <strong>tüm bağlı API anahtarları da kalıcı olarak iptal edilecektir</strong> ve bu işlem geri alınamaz.
        </p>
      `,
      buttons: [
        { label: "Vazgeç", variant: "secondary" },
        {
          label: "Evet, Sil",
          variant: "primary",
          onClick: async () => {
            try {
              await api.deleteProject(projectId);
              showToast("Proje ve bağlı tüm anahtarlar silindi.", "success");
              await loadData();
              return true;
            } catch (err) {
              showToast(err.message || "Proje silinemedi.", "error");
              return true;
            }
          },
        },
      ],
    });
  }

  function handleRevokeApiKey(keyId, keyName) {
    createModal({
      title: "Anahtarı sil",
      iconHtml: icon("warning", 24),
      contentHtml: `
        <p class="u-text-sm u-opacity-subtle">
          <strong>"${sanitizeText(keyName)}"</strong> API anahtarını kalıcı olarak silmek istediğinizden emin misiniz? Bu anahtarı kullanan tüm entegrasyonlar anında erişim hatası almaya başlayacaktır.
        </p>
      `,
      buttons: [
        { label: "Vazgeç", variant: "secondary" },
        {
          label: "Evet, Sil",
          variant: "primary",
          onClick: async () => {
            try {
              await api.revokeApiKey(keyId);
              showToast("API anahtarı başarıyla silindi.", "success");
              await loadData();
              return true;
            } catch (err) {
              showToast(err.message || "Anahtar silinemedi.", "error");
              return true;
            }
          },
        },
      ],
    });
  }

  // Common charting functions are adapted to Svelte Actions (Google Developer Knowledge Canvas Guidelines)
  function actionDrawUsageComboChart(
    node,
    { dataPoints, seriesConfig, onTotalUpdate },
  ) {
    let resizeObserver;
    let overlayListeners;
    let hoverOverlay;
    let animationReq;

    // Accessibility attributes (Google Web / A11y Guidelines)
    node.setAttribute("role", "img");
    node.setAttribute(
      "aria-label",
      "Kullanım istatistikleri kombinasyon grafiği",
    );

    function render() {
      if (animationReq) cancelAnimationFrame(animationReq);
      node.innerHTML = "";
      node.style.position = "relative";

      const processedData = dataPoints.map((d) => ({
        ...d,
        success: Math.max(0, d.requests - (d.errors || 0)),
      }));

      const primaryValues = processedData.map((d) => d[seriesConfig[0].key]);
      const total = primaryValues.reduce((sum, v) => sum + v, 0);
      onTotalUpdate(total);

      let maxVal = 10;
      processedData.forEach((d) => {
        seriesConfig.forEach((s) => {
          if (d[s.key] > maxVal) maxVal = d[s.key];
        });
      });

      const len = processedData.length;
      if (len === 0) return;

      const rect = node.getBoundingClientRect();
      const width = Math.max(100, rect.width || node.clientWidth || 600);
      const height = Math.max(50, rect.height || node.clientHeight || 240);

      const canvas = document.createElement("canvas");
      canvas.style.width = "100%";
      canvas.style.height = "100%";
      canvas.style.display = "block";

      const dpr = window.devicePixelRatio || 1;
      canvas.width = Math.floor(width * dpr);
      canvas.height = Math.floor(height * dpr);

      const ctx = canvas.getContext("2d");

      const padX = 15;
      const padY = 20;
      const W = width - 2 * padX;
      const H = height - padY * 2;

      const gridY1 = padY;
      const gridY2 = padY + H * 0.5;
      const gridY3 = padY + H;

      node.appendChild(canvas);

      hoverOverlay = document.createElement("div");
      hoverOverlay.style.position = "absolute";
      hoverOverlay.style.top = "0";
      hoverOverlay.style.left = "0";
      hoverOverlay.style.width = "100%";
      hoverOverlay.style.height = "100%";
      hoverOverlay.style.cursor = "crosshair";
      hoverOverlay.style.zIndex = "10";

      const tooltip = document.createElement("div");
      tooltip.className = "dev-chart-tooltip-wrapper";

      node.appendChild(hoverOverlay);
      node.appendChild(tooltip);

      let animationStart = null;
      const duration = 800; // ms

      const drawFrame = (timestamp) => {
        if (!animationStart) animationStart = timestamp;
        const elapsed = timestamp - animationStart;
        const rawProgress = Math.min(1, elapsed / duration);
        const progress = 1 - Math.pow(1 - rawProgress, 4);

        ctx.clearRect(0, 0, canvas.width, canvas.height);
        ctx.save();
        ctx.scale(dpr, dpr);

        // Grid batching & pixel grid alignment (Google Canvas Best Practices)
        ctx.beginPath();
        ctx.setLineDash([2, 4]);
        ctx.strokeStyle =
          getComputedStyle(document.body)
            .getPropertyValue("--color-border-light")
            .trim() || "rgba(229,231,235,0.4)";
        ctx.lineWidth = 1;
        [gridY1, gridY2, gridY3].forEach((gy) => {
          const alignedY = Math.floor(gy) + 0.5;
          ctx.moveTo(padX, alignedY);
          ctx.lineTo(width - padX, alignedY);
        });
        ctx.stroke();
        ctx.setLineDash([]);

        seriesConfig.forEach((series) => {
          const colorStr = series.color.includes("var(")
            ? getComputedStyle(document.body)
                .getPropertyValue(
                  series.color.replace("var(", "").replace(")", ""),
                )
                .trim() || series.fallback
            : series.color;

          const points = processedData.map((d, idx) => ({
            x: Math.round(padX + (idx / (len - 1)) * W),
            y: Math.round(padY + H - (d[series.key] / maxVal) * H * progress),
            val: d[series.key],
            date: d.date,
          }));

          if (series.type === "bar") {
            const barWidth = Math.max(2, Math.floor((W / len) * 0.5));
            ctx.fillStyle = colorStr;
            points.forEach((p) => {
              const barHeight = padY + H - p.y;
              if (barHeight > 0) {
                if (ctx.roundRect) {
                  ctx.beginPath();
                  ctx.roundRect(
                    p.x - barWidth / 2,
                    p.y,
                    barWidth,
                    barHeight,
                    [4, 4, 0, 0],
                  );
                  ctx.fill();
                } else {
                  ctx.fillRect(p.x - barWidth / 2, p.y, barWidth, barHeight);
                }
              }
            });
          } else if (series.type === "line") {
            ctx.beginPath();
            ctx.moveTo(points[0].x, points[0].y);
            const tension = 0.15;
            for (let i = 0; i < len - 1; i++) {
              const p0 = points[i];
              const p1 = points[i + 1];
              const pPrev = i > 0 ? points[i - 1] : p0;
              const pNext = i < len - 2 ? points[i + 2] : p1;
              const cp1x = p0.x + (p1.x - pPrev.x) * tension;
              const cp1y = p0.y + (p1.y - pPrev.y) * tension;
              const cp2x = p1.x - (pNext.x - p0.x) * tension;
              const cp2y = p1.y - (pNext.y - p0.y) * tension;
              ctx.bezierCurveTo(
                Math.round(cp1x),
                Math.round(cp1y),
                Math.round(cp2x),
                Math.round(cp2y),
                p1.x,
                p1.y,
              );
            }
            ctx.strokeStyle = colorStr;
            ctx.lineWidth = 2.5;
            ctx.lineCap = "round";
            ctx.lineJoin = "round";
            ctx.stroke();
          }
        });

        ctx.restore();

        if (rawProgress < 1) {
          animationReq = requestAnimationFrame(drawFrame);
        }
      };

      animationReq = requestAnimationFrame(drawFrame);

      const onMove = (e) => {
        const overlayRect = hoverOverlay.getBoundingClientRect();
        const clientX = e.touches ? e.touches[0].clientX : e.clientX;
        const visualPadX = (padX / width) * overlayRect.width;
        const visualW = (W / width) * overlayRect.width;
        const relX = clientX - overlayRect.left - visualPadX;
        const percentX = Math.max(0, Math.min(1, relX / visualW));
        let idx = Math.round(percentX * (len - 1));
        if (idx < 0) idx = 0;
        if (idx >= len) idx = len - 1;

        const dataPoint = processedData[idx];
        if (dataPoint) {
          const displayX = ((padX + (idx / (len - 1)) * W) / width) * 100;
          tooltip.style.left = displayX + "%";
          tooltip.style.top = (padY / height) * 100 + "%";
          tooltip.style.display = "block";
          setTimeout(() => (tooltip.style.opacity = "1"), 10);

          const formattedDate = new Date(dataPoint.date).toLocaleDateString(
            "tr-TR",
            { day: "numeric", month: "long", year: "numeric" },
          );
          let rowsHtml = "";
          seriesConfig.forEach((series) => {
            const color = series.color.includes("var")
              ? `var(${series.color.replace("var(", "").replace(")", "")})`
              : series.fallback;
            rowsHtml += `
              <div class="dev-chart-tooltip">
                <div class="dev-chart-tooltip__label-group">
                  <span class="dev-chart-tooltip__color-indicator" style="--indicator-radius: ${series.type === "line" ? "50%" : "2px"}; --indicator-color: ${color};"></span>
                  <span class="dev-chart-tooltip__label">${series.label}</span>
                </div>
                <span class="dev-chart-tooltip__value">${dataPoint[series.key].toLocaleString("tr-TR")}</span>
              </div>
            `;
          });
          tooltip.innerHTML = `
            <div class="dev-chart-tooltip__header">${formattedDate}</div>
            ${rowsHtml}
          `;
        }
      };

      const onLeave = () => {
        tooltip.style.opacity = "0";
        setTimeout(() => {
          if (tooltip.style.opacity === "0") tooltip.style.display = "none";
        }, 150);
      };

      hoverOverlay.addEventListener("mousemove", onMove);
      hoverOverlay.addEventListener("mouseleave", onLeave);
      hoverOverlay.addEventListener("touchstart", onMove, { passive: true });
      hoverOverlay.addEventListener("touchmove", onMove, { passive: true });
      hoverOverlay.addEventListener("touchend", onLeave);

      overlayListeners = [
        ["mousemove", onMove, false],
        ["mouseleave", onLeave, false],
        ["touchstart", onMove, { passive: true }],
        ["touchmove", onMove, { passive: true }],
        ["touchend", onLeave, false],
      ];
    }

    render();

    // Responsive element resize observation (Google Canvas Performance Guidelines)
    if (typeof ResizeObserver !== "undefined") {
      resizeObserver = new ResizeObserver(() => render());
      resizeObserver.observe(node);
    }

    return {
      update(params) {
        dataPoints = params.dataPoints;
        seriesConfig = params.seriesConfig;
        onTotalUpdate = params.onTotalUpdate;
        render();
      },
      destroy() {
        if (animationReq) cancelAnimationFrame(animationReq);
        if (resizeObserver) resizeObserver.disconnect();
        if (overlayListeners && hoverOverlay) {
          overlayListeners.forEach(([type, fn, opts]) =>
            hoverOverlay.removeEventListener(type, fn, opts),
          );
        }
      },
    };
  }

  function actionDrawCanvasChart(
    node,
    {
      dataPoints,
      valueKey,
      strokeColor,
      fillColorHex,
      customTitle,
      onTotalUpdate,
    },
  ) {
    let resizeObserver;
    let overlayListeners;
    let hoverOverlay;
    let animationReq;

    // Accessibility attributes (Google Web / A11y Guidelines)
    node.setAttribute("role", "img");
    node.setAttribute(
      "aria-label",
      customTitle || "Kullanım istatistikleri alan grafiği",
    );

    function render() {
      if (animationReq) cancelAnimationFrame(animationReq);
      node.innerHTML = "";
      node.style.position = "relative";

      const values = dataPoints.map((d) => d[valueKey]);
      const total = values.reduce((sum, v) => sum + v, 0);
      const maxVal = Math.max(...values, 10);
      const len = dataPoints.length;
      if (onTotalUpdate) onTotalUpdate(maxVal);
      if (len === 0) return;

      const rect = node.getBoundingClientRect();
      const width = Math.max(100, rect.width || node.clientWidth || 600);
      const height = Math.max(50, rect.height || node.clientHeight || 240);

      const canvas = document.createElement("canvas");
      canvas.style.width = "100%";
      canvas.style.height = "100%";
      canvas.style.display = "block";

      const dpr = window.devicePixelRatio || 1;
      canvas.width = Math.floor(width * dpr);
      canvas.height = Math.floor(height * dpr);

      const ctx = canvas.getContext("2d");

      const padX = 15;
      const padY = 20;
      const W = width - 2 * padX;
      const H = height - padY - 30;

      const gridY1 = padY;
      const gridY2 = padY + H * 0.5;
      const gridY3 = padY + H;

      node.appendChild(canvas);

      hoverOverlay = document.createElement("div");
      hoverOverlay.style.position = "absolute";
      hoverOverlay.style.top = "0";
      hoverOverlay.style.left = "0";
      hoverOverlay.style.width = "100%";
      hoverOverlay.style.height = "100%";
      hoverOverlay.style.cursor = "crosshair";
      hoverOverlay.style.zIndex = "10";

      const tooltip = document.createElement("div");
      tooltip.className = "dev-chart-tooltip-wrapper";

      node.appendChild(hoverOverlay);
      node.appendChild(tooltip);

      let animationStart = null;
      const duration = 800;

      const drawFrame = (timestamp) => {
        if (!animationStart) animationStart = timestamp;
        const elapsed = timestamp - animationStart;
        const rawProgress = Math.min(1, elapsed / duration);
        const progress = 1 - Math.pow(1 - rawProgress, 4);

        ctx.clearRect(0, 0, canvas.width, canvas.height);
        ctx.save();
        ctx.scale(dpr, dpr);

        const points = values.map((val, idx) => {
          const x = Math.round(padX + (idx / (len - 1)) * W);
          const y = Math.round(padY + H - (val / maxVal) * H * progress);
          return { x, y, val, date: dataPoints[idx].date };
        });

        // Grid batching & pixel grid alignment (Google Canvas Best Practices)
        ctx.beginPath();
        ctx.setLineDash([2, 4]);
        ctx.strokeStyle =
          getComputedStyle(document.body)
            .getPropertyValue("--color-border-light")
            .trim() || "rgba(229,231,235,0.4)";
        ctx.lineWidth = 1;
        [gridY1, gridY2, gridY3].forEach((gy) => {
          const alignedY = Math.floor(gy) + 0.5;
          ctx.moveTo(padX, alignedY);
          ctx.lineTo(width - padX, alignedY);
        });
        ctx.stroke();
        ctx.setLineDash([]);

        const gradient = ctx.createLinearGradient(0, padY, 0, padY + H);
        gradient.addColorStop(0, fillColorHex + "66");
        gradient.addColorStop(1, fillColorHex + "00");

        const tension = 0.15;

        ctx.beginPath();
        ctx.moveTo(points[0].x, padY + H);
        ctx.lineTo(points[0].x, points[0].y);
        for (let i = 0; i < len - 1; i++) {
          const p0 = points[i],
            p1 = points[i + 1];
          const pPrev = i > 0 ? points[i - 1] : p0,
            pNext = i < len - 2 ? points[i + 2] : p1;
          const cp1x = p0.x + (p1.x - pPrev.x) * tension,
            cp1y = p0.y + (p1.y - pPrev.y) * tension;
          const cp2x = p1.x - (pNext.x - p0.x) * tension,
            cp2y = p1.y - (pNext.y - p0.y) * tension;
          ctx.bezierCurveTo(
            Math.round(cp1x),
            Math.round(cp1y),
            Math.round(cp2x),
            Math.round(cp2y),
            p1.x,
            p1.y,
          );
        }
        ctx.lineTo(points[len - 1].x, padY + H);
        ctx.closePath();
        ctx.fillStyle = gradient;
        ctx.fill();

        ctx.beginPath();
        ctx.moveTo(points[0].x, points[0].y);
        for (let i = 0; i < len - 1; i++) {
          const p0 = points[i],
            p1 = points[i + 1];
          const pPrev = i > 0 ? points[i - 1] : p0,
            pNext = i < len - 2 ? points[i + 2] : p1;
          const cp1x = p0.x + (p1.x - pPrev.x) * tension,
            cp1y = p0.y + (p1.y - pPrev.y) * tension;
          const cp2x = p1.x - (pNext.x - p0.x) * tension,
            cp2y = p1.y - (pNext.y - p0.y) * tension;
          ctx.bezierCurveTo(
            Math.round(cp1x),
            Math.round(cp1y),
            Math.round(cp2x),
            Math.round(cp2y),
            p1.x,
            p1.y,
          );
        }
        ctx.strokeStyle = strokeColor;
        ctx.lineWidth = 2.5;
        ctx.lineCap = "round";
        ctx.lineJoin = "round";
        ctx.stroke();

        ctx.fillStyle =
          getComputedStyle(document.body)
            .getPropertyValue("--color-text-secondary")
            .trim() || "#6b7280";
        ctx.font = "500 10px Inter, sans-serif";
        ctx.textBaseline = "middle";

        const formatDateLabel = (dStr) =>
          new Date(dStr).toLocaleDateString("tr-TR", {
            day: "numeric",
            month: "short",
          });
        const labelFirst = formatDateLabel(points[0].date);
        const labelLast = "Bugün";
        const midDateStr = points[Math.floor((len - 1) / 2)].date;
        const labelMiddle = formatDateLabel(midDateStr);
        const textY = height - 10;

        ctx.globalAlpha = progress;
        ctx.textAlign = "left";
        ctx.fillText(labelFirst, padX, textY);
        ctx.textAlign = "center";
        ctx.fillText(labelMiddle, width / 2, textY);
        ctx.textAlign = "right";
        ctx.fillText(labelLast, width - padX, textY);
        ctx.globalAlpha = 1.0;

        ctx.restore();

        if (rawProgress < 1) {
          animationReq = requestAnimationFrame(drawFrame);
        }
      };

      animationReq = requestAnimationFrame(drawFrame);

      const onMove = (e) => {
        const overlayRect = hoverOverlay.getBoundingClientRect();
        const clientX = e.touches ? e.touches[0].clientX : e.clientX;
        const visualPadX = (padX / width) * overlayRect.width;
        const visualW = (W / width) * overlayRect.width;
        const relX = clientX - overlayRect.left - visualPadX;
        const percentX = Math.max(0, Math.min(1, relX / visualW));
        let idx = Math.round(percentX * (len - 1));
        if (idx < 0) idx = 0;
        if (idx >= len) idx = len - 1;

        const dataPoint = dataPoints[idx];
        if (dataPoint) {
          const displayX = ((padX + (idx / (len - 1)) * W) / width) * 100;
          tooltip.style.left = displayX + "%";
          tooltip.style.top = (padY / height) * 100 + "%";
          tooltip.style.display = "block";
          setTimeout(() => (tooltip.style.opacity = "1"), 10);

          const formattedDate = new Date(dataPoint.date).toLocaleDateString(
            "tr-TR",
            { day: "numeric", month: "long", year: "numeric" },
          );
          const seriesLabel = customTitle || "Tepe istek";
          tooltip.innerHTML = `
            <div class="dev-chart-tooltip__header">${formattedDate}</div>
            <div class="dev-chart-tooltip">
              <div class="dev-chart-tooltip__label-group">
                <span class="dev-chart-tooltip__color-indicator" style="--indicator-radius: 50%; --indicator-color: ${strokeColor};"></span>
                <span class="dev-chart-tooltip__label">${seriesLabel}</span>
              </div>
              <span class="dev-chart-tooltip__value">${dataPoint[valueKey].toLocaleString("tr-TR")}</span>
            </div>
          `;
        }
      };

      const onLeave = () => {
        tooltip.style.opacity = "0";
        setTimeout(() => {
          if (tooltip.style.opacity === "0") tooltip.style.display = "none";
        }, 150);
      };

      hoverOverlay.addEventListener("mousemove", onMove);
      hoverOverlay.addEventListener("mouseleave", onLeave);
      hoverOverlay.addEventListener("touchstart", onMove, { passive: true });
      hoverOverlay.addEventListener("touchmove", onMove, { passive: true });
      hoverOverlay.addEventListener("touchend", onLeave);

      overlayListeners = [
        ["mousemove", onMove, false],
        ["mouseleave", onLeave, false],
        ["touchstart", onMove, { passive: true }],
        ["touchmove", onMove, { passive: true }],
        ["touchend", onLeave, false],
      ];
    }

    render();

    // Responsive element resize observation (Google Canvas Performance Guidelines)
    if (typeof ResizeObserver !== "undefined") {
      resizeObserver = new ResizeObserver(() => render());
      resizeObserver.observe(node);
    }

    return {
      update(params) {
        dataPoints = params.dataPoints;
        valueKey = params.valueKey;
        strokeColor = params.strokeColor;
        fillColorHex = params.fillColorHex;
        customTitle = params.customTitle;
        onTotalUpdate = params.onTotalUpdate;
        render();
      },
      destroy() {
        if (animationReq) cancelAnimationFrame(animationReq);
        if (resizeObserver) resizeObserver.disconnect();
        if (overlayListeners && hoverOverlay) {
          overlayListeners.forEach(([type, fn, opts]) =>
            hoverOverlay.removeEventListener(type, fn, opts),
          );
        }
      },
    };
  }
</script>

<Seo
  title="Geliştirici Portalı - Kepçe"
  description="Proje Yönetimi ve API Anahtarları."
  image="https://kepce.org/api/v1/public/og/page/gelistirici"
  noindex={true}
/>

{#if !user}
  <div class="content-page">
    <div class="content-page__header">
      <div>
        <h1 class="content-page__title">Geliştirici Panosu</h1>
        <button
          type="button"
          class="content-page__archive-link"
          onclick={() => (isCommercialModalOpen = true)}
        >
          Ticari API nedir?
        </button>
      </div>
    </div>
    <div class="content-page__body">
      <div class="empty-state-container">
        <EmptyState
          statusCode={401}
          desc={"API anahtarlarınızı ve projelerinizi yönetebilmek için giriş yapmanız gerekmektedir."}
        >
          <a
            href="/giris?redirect=%2Fgelistirici"
            class="btn btn--primary btn--squish"
            data-link>Giriş yap</a
          >
        </EmptyState>
      </div>
    </div>
  </div>
{:else}
  <div class="content-page">
    <div class="content-page__header">
      <div class="u-flex u-flex-justify-between u-flex-align-center u-w-full">
        <h1 class="content-page__title">Geliştirici Panosu</h1>
        <button
          type="button"
          class="content-page__archive-link btn--squish"
          onclick={() => (isCommercialModalOpen = true)}
        >
          Ticari API nedir?
        </button>
      </div>
    </div>

    <div class="content-page__body">
      <!-- PROJELER -->
      <section id="projeler" class="u-mb-xl">
        <div class="u-flex u-flex-justify-between u-flex-align-center u-mb-md">
          <h2>Projeler</h2>
          <button
            class="btn btn--primary btn--sm btn--squish"
            id="btn-create-project"
            onclick={handleCreateProject}>Proje oluştur</button
          >
        </div>
        <div id="projects-list-container">
          {#if isLoading}
            <div class="card u-p-lg u-text-center u-opacity-dim">
              Projeleriniz yükleniyor...
            </div>
          {:else if errorMsg}
            <div class="card u-p-lg u-text-center u-color-error">
              {errorMsg}
            </div>
          {:else if projects.length === 0}
            <div class="card u-p-lg u-text-center u-opacity-dim">
              Henüz projeniz yok
            </div>
          {:else}
            <div class="table-wrapper">
              <table class="table">
                <thead>
                  <tr>
                    <th class="u-text-center">Proje</th>
                    <th class="u-text-center">Anahtarlar</th>
                    <th class="u-text-center">Oluşturulma zamanı</th>
                    <th class="u-text-center">Kademe</th>
                    <th class="u-text-center">İşlemler</th>
                  </tr>
                </thead>
                <tbody>
                  {#each projects as project}
                    {@const projectKeys = keys.filter(
                      (k) => k.project_id === project.id,
                    )}
                    {@const isCommercial = projectKeys.some(
                      (k) => k.tier === "commercial",
                    )}
                    {@const tierName = isCommercial ? "ticari" : "bireysel"}
                    <tr>
                      <td class="u-text-center dev-table-project-name"
                        >{sanitizeText(project.name)}</td
                      >
                      <td class="u-text-center u-font-bold"
                        >{projectKeys.length} adet</td
                      >
                      <td class="u-text-center"
                        >{formatDate(project.created_at)}</td
                      >
                      <td class="u-text-center dev-table-tier-{tierName}"
                        >{tierName}</td
                      >
                      <td class="u-text-center">
                        <div class="dev-table-actions">
                          <button
                            class="dev-table-btn btn-manage-usage"
                            title="Kullanımı yönet"
                            onclick={() =>
                              showToast(
                                `"${sanitizeText(project.name)}" projesi için kullanım istatistikleri ve sınır yönetimi yakında aktif edilecektir.`,
                                "info",
                              )}
                          >
                            {@html icon("usage", 16)}
                          </button>
                          <button
                            class="dev-table-btn btn-manage-keys"
                            title="Anahtarları yönet"
                            onclick={() =>
                              openKeysManagementModal(project.id, project.name)}
                          >
                            {@html icon("key", 16)}
                          </button>
                          <button
                            class="dev-table-btn btn-edit-project"
                            title="Projeyi düzenle"
                            onclick={() =>
                              handleEditProject(project.id, project.name)}
                          >
                            {@html icon("edit", 16)}
                          </button>
                          <button
                            class="dev-table-btn dev-table-btn--danger btn-delete-project"
                            title="Projeyi sil"
                            onclick={() =>
                              handleDeleteProject(project.id, project.name)}
                          >
                            {@html icon("trash", 16)}
                          </button>
                        </div>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </div>
      </section>

      <!-- API ANAHTARLARI -->
      <section id="api-anahtarlari" class="u-mb-xl">
        <div class="u-flex u-flex-justify-between u-flex-align-center u-mb-md">
          <h2>API anahtarları</h2>
          <button
            class="btn btn--primary btn--sm"
            id="btn-create-apikey"
            disabled={projects.length === 0}
            onclick={handleCreateApiKey}>API anahtarı oluştur</button
          >
        </div>
        <div id="apikeys-list-container">
          {#if isLoading}
            <div class="card u-p-lg u-text-center u-opacity-dim">
              API anahtarlarınız yükleniyor...
            </div>
          {:else if errorMsg}
            <!-- Hidden intentionally if projects fails, it shares error -->
          {:else if keys.length === 0}
            <div class="card u-p-lg u-text-center u-opacity-dim">
              API anahtarı bulunmuyor
            </div>
          {:else}
            <div class="table-wrapper">
              <table class="table">
                <thead>
                  <tr>
                    <th class="u-text-center">Anahtar</th>
                    <th class="u-text-center">İsim</th>
                    <th class="u-text-center">Proje</th>
                    <th class="u-text-center">Oluşturulma zamanı</th>
                    <th class="u-text-center">Kademe</th>
                    <th class="u-text-center">İşlemler</th>
                  </tr>
                </thead>
                <tbody>
                  {#each keys as key}
                    {@const project = projects.find(
                      (p) => p.id === key.project_id,
                    )}
                    {@const projectName = project ? project.name : "Bağımsız"}
                    {@const tierName =
                      key.tier === "commercial" ? "ticari" : "bireysel"}
                    <tr>
                      <td class="u-text-center dev-table-project-name"
                        >...{sanitizeText(
                          key.key_prefix ? key.key_prefix.slice(-8) : "",
                        )}</td
                      >
                      <td class="u-text-center u-font-bold"
                        >{sanitizeText(key.name || "İsimsiz")}</td
                      >
                      <td class="u-text-center">{sanitizeText(projectName)}</td>
                      <td class="u-text-center">{formatDate(key.created_at)}</td
                      >
                      <td class="u-text-center dev-table-tier-{tierName}"
                        >{tierName}</td
                      >
                      <td class="u-text-center">
                        <div class="dev-table-actions">
                          <button
                            class="dev-table-btn btn-key-usage"
                            title="Kullanımı yönet"
                            onclick={() =>
                              showToast(
                                `"${sanitizeText(key.name)}" anahtarı için kullanım istatistikleri yakında aktif edilecektir.`,
                                "info",
                              )}
                          >
                            {@html icon("usage", 16)}
                          </button>
                          <button
                            class="dev-table-btn btn-edit-key-name"
                            title="Anahtarı düzenle"
                            onclick={() =>
                              showToast(
                                "API anahtarı ismini güncelleme desteği bir sonraki güncellemede aktif edilecektir.",
                                "info",
                              )}
                          >
                            {@html icon("edit", 16)}
                          </button>
                          <button
                            class="dev-table-btn dev-table-btn--danger btn-delete-key"
                            title="Anahtarı sil"
                            onclick={() => handleRevokeApiKey(key.id, key.name)}
                          >
                            {@html icon("trash", 16)}
                          </button>
                        </div>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </div>
      </section>

      <!-- KULLANIM -->
      <section id="kullanim" class="u-mb-xl">
        <h2 class="u-mb-md">Kullanım</h2>

        <div class="dev-filters-row u-mb-lg">
          <div class="dev-filter-group">
            <span class="dev-filter-label">Proje:</span>
            <div class="dev-filter-select-wrapper">
              <Dropdown
                options={projectOptions}
                bind:value={usageProjectFilter}
                disabled={projects.length === 0}
              />
            </div>
          </div>

          <div class="dev-filter-group">
            <span class="dev-filter-label">Zaman:</span>
            <div class="dev-filter-select-wrapper">
              <Dropdown
                options={timeOptions}
                bind:value={usageTimeFilter}
                disabled={projects.length === 0}
              />
            </div>
          </div>
        </div>
        <div class="dev-usage-grid">
          <div class="dev-usage-card">
            <div class="dev-usage-card__header">
              <h3 class="dev-usage-card__title">Toplam API istekleri</h3>
              <span class="dev-usage-card__value" id="total-requests-value"
                >{totalRequests.toLocaleString("tr-TR")}</span
              >
            </div>
            <div
              class="dev-usage-card__chart"
              id="requests-chart-container"
              use:actionDrawUsageComboChart={{
                dataPoints: usageData,
                seriesConfig: [
                  {
                    key: "requests",
                    label: "Tüm istekler",
                    type: "bar",
                    color: "var(--color-accent-primary)",
                    fallback: "#e38e69",
                  },
                  {
                    key: "success",
                    label: "Başarılı istekler",
                    type: "line",
                    color: "var(--color-accent-positive)",
                    fallback: "#add18a",
                  },
                ],
                onTotalUpdate: (v) => (totalRequests = v),
              }}
            ></div>
          </div>

          <div class="dev-usage-card">
            <div class="dev-usage-card__header">
              <h3 class="dev-usage-card__title">Toplam API hataları</h3>
              <span
                class="dev-usage-card__value dev-usage-card__value--danger"
                id="total-errors-value"
                >{totalErrors.toLocaleString("tr-TR")}</span
              >
            </div>
            <div
              class="dev-usage-card__chart"
              id="errors-chart-container"
              use:actionDrawUsageComboChart={{
                dataPoints: usageData,
                seriesConfig: [
                  {
                    key: "errors",
                    label: "Toplam hata",
                    type: "bar",
                    color: "var(--color-accent-negative)",
                    fallback: "#d2564a",
                  },
                ],
                onTotalUpdate: (v) => (totalErrors = v),
              }}
            ></div>
          </div>
        </div>
      </section>

      <!-- SINIRLAR -->
      <section id="sinirlar" class="u-mb-xl">
        <h2 class="u-mb-md">Sınırlar</h2>

        <div class="dev-filters-row u-mb-lg">
          <div class="dev-filter-group">
            <span class="dev-filter-label">Proje:</span>
            <div class="dev-filter-select-wrapper">
              <Dropdown
                options={projectOptions}
                bind:value={limitsProjectFilter}
                disabled={projects.length === 0}
              />
            </div>
          </div>

          <div class="dev-filter-group">
            <span class="dev-filter-label">Zaman:</span>
            <div class="dev-filter-select-wrapper">
              <Dropdown
                options={timeOptions}
                bind:value={limitsTimeFilter}
                disabled={projects.length === 0}
              />
            </div>
          </div>
        </div>

        <div class="dev-limits-grid">
          <div class="dev-limit-card">
            <h3 class="dev-limit-card__title">Dakikalık istek limiti</h3>
            <div class="dev-limit-card__content">
              <p>Bireysel: 240 istek</p>
              <p>Ticari: 240 istek</p>
            </div>
            <p class="dev-limit-card__footer">API limitleri değişebilir.</p>
          </div>

          <div class="dev-limit-card">
            <h3 class="dev-limit-card__title">Günlük istek limiti</h3>
            <div class="dev-limit-card__content">
              <p>Bireysel: 2.500 istek</p>
              <p>Ticari: 100.000 istek</p>
            </div>
            <p class="dev-limit-card__footer">API limitleri değişebilir.</p>
          </div>

          <div class="dev-usage-card">
            <div class="dev-usage-card__header">
              <h3 class="dev-usage-card__title">Dakikalık tepe istekler</h3>
              <span class="dev-usage-card__value" id="limits-peak-min-value"
                >{peakMinMax.toLocaleString("tr-TR")}</span
              >
            </div>
            <div
              class="dev-usage-card__chart"
              id="limits-peak-min-chart"
              use:actionDrawCanvasChart={{
                dataPoints: slicedDataMin,
                valueKey: "requests",
                strokeColor: "#e38e69",
                fillColorHex: "#e38e69",
                customTitle: "Dakika Başı Tepe İstekler",
                onTotalUpdate: (v) => (peakMinMax = v),
              }}
            ></div>
          </div>

          <div class="dev-usage-card">
            <div class="dev-usage-card__header">
              <h3 class="dev-usage-card__title">Günlük tepe istekler</h3>
              <span class="dev-usage-card__value" id="limits-peak-day-value"
                >{peakDayMax.toLocaleString("tr-TR")}</span
              >
            </div>
            <div
              class="dev-usage-card__chart"
              id="limits-peak-day-chart"
              use:actionDrawCanvasChart={{
                dataPoints: slicedDataDay,
                valueKey: "requests",
                strokeColor: "#e38e69",
                fillColorHex: "#e38e69",
                customTitle: "Gün Başı Tepe İstekler",
                onTotalUpdate: (v) => (peakDayMax = v),
              }}
            ></div>
          </div>
        </div>
      </section>
    </div>
  </div>
{/if}

{#if isCommercialModalOpen}
  <Modal
    options={{ title: "Ticari API Nedir?" }}
    onClose={() => (isCommercialModalOpen = false)}
  >
    {#snippet children()}
      <div class="u-flex u-flex-column u-gap-md">
        <p class="u-text-sm u-color-muted">
          Kepçe API; öğrenci projeleri, kulüpler ve açık kaynak geliştiriciler
          için tamamen ücretsizdir ancak yüksek hacimli veya ticari kullanım
          gerektiren durumlarda da nakit ücret yerine <strong
            >kazan-kazan veri ortaklığı</strong
          > modelini uyguluyoruz.
        </p>
        <div class="card u-p-md u-bg-surface-sunken">
          <h4 class="u-font-bold u-text-sm u-mb-xs">Ortaklık İlkeleri</h4>
          <ul
            class="u-text-xs u-color-muted"
            style="padding-left: 1.25rem; line-height: 1.6;"
          >
            <li>
              <strong>Marka Görünürlüğü:</strong> Platformunuzda "Veriler Kepçe (kepce.org)
              tarafından sağlanmaktadır" bağlantısı yer almalıdır.
            </li>
            <li>
              <strong>Veri Besleme:</strong> Toplanan menü veya fiyat fotoğraflarının
              doğrudan Kepçe veri tabanına iletilmesi.
            </li>
            <li>
              <strong>Önbellek & Node Koruma:</strong> Sunucularımızın korunması
              için backend seviyesinde önbellek tutulması.
            </li>
          </ul>
        </div>
      </div>
    {/snippet}
    {#snippet footer()}
      <a href="/menu-gonder" class="btn btn--secondary btn--squish" data-link>
        Menü Katkısı Sağla
      </a>
      <a
        href="mailto:iletisim@kepce.org?subject=Ticari%20API%20ve%20Veri%20Ortakligi%20Basvurusu"
        class="btn btn--primary btn--squish"
      >
        Ortaklık Başvurusu (E-posta)
      </a>
    {/snippet}
  </Modal>
{/if}

{#if isCreateApiKeyModalOpen}
  <Modal
    options={{ title: "Yeni API anahtarı oluştur" }}
    onClose={() => (isCreateApiKeyModalOpen = false)}
  >
    {#snippet children()}
      <div class="form-group form-group--floating u-mb-md">
        <input
          id="api-key-name"
          type="text"
          class="form-input"
          placeholder=" "
          maxlength="30"
          autocomplete="off"
          bind:value={newApiKeyState.name}
        />
        <label for="api-key-name" class="form-label"
          >Anahtar ismi (Örn: Telegram botu)</label
        >
      </div>
      <div class="form-group u-mb-md">
        <div class="u-display-block u-mb-xs u-text-sm u-color-muted">
          Proje seç
        </div>
        <Dropdown
          options={projects.map((p) => ({
            value: p.id,
            label: sanitizeText(p.name),
          }))}
          bind:value={newApiKeyState.projectId}
        />
      </div>
      <span class="form-help"
        >Bu anahtar ile Kepçe API servislerine projeniz üzerinden
        erişebilirsiniz.</span
      >
    {/snippet}
    {#snippet footer()}
      <button
        class="btn btn--secondary"
        onclick={() => (isCreateApiKeyModalOpen = false)}>Vazgeç</button
      >
      <button class="btn btn--primary" onclick={submitCreateApiKey}
        >Oluştur</button
      >
    {/snippet}
  </Modal>
{/if}
