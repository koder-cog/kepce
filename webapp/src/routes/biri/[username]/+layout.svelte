<script>
  import "@/styles/pages/_profile.css";
  import { goto } from "$app/navigation";
  import { globalState, authActions } from "@/state.svelte.js";
  import { onMount, tick, setContext } from "svelte";
  import { api } from "@/api/index.js";
  import { icon } from "@/components/ui/icons.js";
  import { timeAgo } from "@/utils/date.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import { createModal } from "@/components/features/modal.js";
  import { openUserReportModal } from "@/components/features/report-modal.js";
  import { showToast } from "@/components/ui/toast.js";
  import Loader from "@/components/ui/Loader.svelte";
  import { sanitizeText } from "@/utils/sanitize.js";
  import ActionMenu from "@/components/features/ActionMenu.svelte";
  import TabBar from "@/components/ui/TabBar.svelte";
  import Seo from "@/components/ui/Seo.svelte";
  import { initCharCounter } from "@/utils/char-counter.js";
  import { page } from "$app/stores";

  let { children } = $props();

  let params = $derived($page.params);
  let username = $derived(params?.username);

  let loading = $state(true);
  let profile = $state(null);
  let error = $state(null);
  let avatarTimestamp = $state(Date.now());

  setContext("profileContext", () => profile);

  // Tabs state based on URL
  let currentPath = $derived($page.url.pathname);

  let tabStructure = $derived({
    yorumlar: {
      label: "Yorumlar",
      path: `/biri/${username}`,
      icon: icon("chat", 18),
    },
    rozetler: {
      label: "Rozetler",
      path: `/biri/${username}/rozetler`,
      icon: icon("trophy", 18),
    },
    sabitlenenler: {
      label: "Favoriler",
      path: `/biri/${username}/sabitlenenler`,
      icon: icon("starFilled", 18),
    },
    begendikleri: {
      label: "Beğeniler",
      path: `/biri/${username}/begendikleri`,
      icon: icon("voteUpFilled", 18),
    },
    yazarlar: {
      label: "Sevilenler",
      path: `/biri/${username}/yazarlar`,
      icon: icon("user", 18),
    },
  });

  function getActiveTab(path) {
    if (path.endsWith("/rozetler")) return "rozetler";
    if (path.endsWith("/sabitlenenler")) return "sabitlenenler";
    if (path.endsWith("/begendikleri")) return "begendikleri";
    if (path.endsWith("/yazarlar")) return "yazarlar";
    return "yorumlar";
  }

  let activeTab = $derived(getActiveTab(currentPath));

  let isOwner = $derived(globalState?.user?.id === profile?.id);
  let safeNickname = $derived(sanitizeText(profile?.username || "isimsiz"));
  let safeBio = $derived(sanitizeText(profile?.bio || ""));
  let createdDate = $derived(
    profile?.created_at
      ? new Date(profile.created_at).toLocaleDateString("tr-TR", {
          year: "numeric",
          month: "long",
        })
      : "",
  );

  $effect(() => {
    if (username && globalState.isReady) {
      loadProfile();
    } else if (!username) {
      loading = false;
      error = {
        status: 404,
        message: "Kullanıcı bulunamadı. Kimi aradığını belirtmedin.",
      };
    }
  });

  async function loadProfile() {
    loading = true;
    error = null;
    profile = null;
    try {
      profile = await api.getPublicProfile(username);
    } catch (err) {
      error = err;
    } finally {
      loading = false;
    }
  }

  async function handleBlock() {
    if (!globalState?.user) {
      authActions.triggerLogin();
      return;
    }
    try {
      await api.blockUser(profile.id);
      profile.is_blocked = true;
      showToast("Kullanıcı engellendi.", "success");
    } catch (err) {
      if (
        err.message &&
        err.message.toLowerCase().includes("already blocked")
      ) {
        profile.is_blocked = true;
        showToast("Kullanıcı zaten engellenmiş.", "info");
      } else {
        showToast(err.message || "Engellenemedi.", "error");
      }
    }
  }

  async function handleUnblock() {
    if (!globalState?.user) {
      authActions.triggerLogin();
      return;
    }
    try {
      await api.unblockUser(profile.id);
      profile.is_blocked = false;
      showToast("Engel kaldırıldı.", "success");
    } catch (err) {
      showToast(err.message || "Engel kaldırılamadı.", "error");
    }
  }

  // --- Profile Data Getters ---
  let progressPercent = $derived(
    profile?.level_progress?.progress_percent || 0,
  );

  function getFlairs(prof) {
    if (!prof) return [];
    const flairs = [];
    if (prof.id === 1) flairs.push({ text: "Yönetim", cls: "flair--founder" });
    if (prof.is_admin || prof.role === "admin")
      flairs.push({ text: "Moderatör", cls: "flair--moderator" });
    return flairs;
  }

  function getEarnedBadges(prof) {
    if (!prof?.badges) return [];
    return prof.badges.filter((b) => b.unlocked);
  }

  const BADGE_TIER_MAP = {
    hucre_hapsi: "gold",
    demirbas: "gold",
    vefakar: "gold",
    kanaat_onderi: "gold",
    bakanlik_ajani: "gold",
    kurumsal_caresizlik: "silver",
    stokholm_sendromu: "silver",
    halkin_adami: "silver",
    fahri_mufettis: "silver",
    bas_muhbir: "silver",
    derin_devlet: "silver",
    demir_mide: "bronze",
    klavyesor: "bronze",
    ilk_kepce: "other",
    muzmin_muhalif: "other",
    linc_kurbani: "other",
    caylak_gammaz: "other",
    kacak_asci: "other",
  };

  function getBadgeTier(badge) {
    if (badge.tier) return badge.tier;
    if (badge.slug && BADGE_TIER_MAP[badge.slug])
      return BADGE_TIER_MAP[badge.slug];
    return "other";
  }

  let showcasedBadges = $derived.by(() => {
    if (!profile?.badges) return [];
    const earned = profile.badges.filter((b) => b.unlocked);
    if (Array.isArray(profile.pinned_badges)) {
      return profile.pinned_badges
        .map((slug) => earned.find((b) => b.slug === slug))
        .filter(Boolean)
        .slice(0, 5);
    }
    return earned.slice(0, 5);
  });

  function openBadgeShowcaseModal() {
    const earned = getEarnedBadges(profile);
    if (earned.length === 0) {
      showToast("Henüz sergilenecek bir rozetin yok.", "info");
      return;
    }

    let pinnedSlugs = Array.isArray(profile.pinned_badges)
      ? profile.pinned_badges
          .filter((slug) => earned.some((b) => b.slug === slug))
          .slice(0, 5)
      : showcasedBadges.map((b) => b.slug);

    function renderBadgeCard(b, isPinned, index, totalPinned, isMax) {
      const tier = getBadgeTier(b);
      const isFirst = index === 0;
      const isLast = index === totalPinned - 1;
      const isDisabled = isMax && !isPinned;

      return `
        <div class="badge-picker-card ${isPinned ? "is-pinned" : ""} ${isDisabled ? "is-disabled" : ""}" data-slug="${sanitizeText(b.slug)}">
          <div class="badge-picker-card__top">
            <div class="badge-picker-card__identity">
              <div class="achievement-badge badge--${tier}">
                ${icon(b.icon || "starFilled", 20)}
              </div>
              <span class="badge-picker-card__name">${sanitizeText(b.name)}</span>
            </div>
          </div>
          <div class="badge-picker-card__bottom">
            <div class="badge-picker-card__reorder">
              ${
                isPinned
                  ? `
                <button type="button" class="badge-reorder-btn" data-action="up" data-index="${index}" title="Öne taşı" ${isFirst ? "disabled" : ""}>
                  ${icon("chevronUp", 16)}
                </button>
                <button type="button" class="badge-reorder-btn" data-action="down" data-index="${index}" title="Arkaya taşı" ${isLast ? "disabled" : ""}>
                  ${icon("chevronDown", 16)}
                </button>
              `
                  : ""
              }
            </div>
            <label class="form-switch-row" title="${isDisabled ? "En fazla 5 rozet seçilebilir" : isPinned ? "Vitrinden çıkar" : "Vitrine ekle"}">
              <input type="checkbox" class="c-input-hidden" data-action="toggle-pin" data-slug="${sanitizeText(b.slug)}" ${isPinned ? "checked" : ""} ${isDisabled ? "disabled" : ""} />
              <span class="c-switch"><span class="c-switch__handle"></span></span>
            </label>
          </div>
        </div>
      `;
    }

    function buildContentHtml() {
      const isMax = pinnedSlugs.length >= 5;
      const pinnedBadges = pinnedSlugs
        .map((slug) => earned.find((b) => b.slug === slug))
        .filter(Boolean);
      const unpinnedBadges = earned.filter(
        (b) => !pinnedSlugs.includes(b.slug),
      );

      const progressPercent = Math.min(
        100,
        Math.round((pinnedSlugs.length / 5) * 100),
      );

      const cardsHtml = [
        ...pinnedBadges.map((b, idx) =>
          renderBadgeCard(b, true, idx, pinnedBadges.length, isMax),
        ),
        ...unpinnedBadges.map((b) =>
          renderBadgeCard(b, false, -1, 0, isMax),
        ),
      ].join("");

      const descText =
        earned.length <= 5
          ? "Profilinde sergilenecek rozetleri seç:"
          : "Profilinde sergilenecek 5 rozeti seç:";

      return `
        <div class="c-modal__form-group">
          <div class="badge-picker-header">
            <p class="badge-picker-desc">
              ${descText}
            </p>
            <div class="badge-picker-status">
              <div class="badge-picker-bar" role="progressbar" aria-valuenow="${pinnedSlugs.length}" aria-valuemin="0" aria-valuemax="5" style="--badge-progress: ${progressPercent}%;">
                <div class="badge-picker-bar__fill"></div>
              </div>
              <span class="badge-picker-counter">${pinnedSlugs.length} / 5</span>
            </div>
          </div>
          <div class="badge-picker-container" id="badge-picker-container">
            ${cardsHtml}
          </div>
        </div>
      `;
    }

    const modalObj = createModal({
      title: "Rozet Vitrini",
      iconHtml: icon("trophy", 24),
      contentHtml: buildContentHtml(),
      buttons: [
        { label: "Vazgeç", variant: "secondary" },
        {
          label: "Kaydet",
          variant: "primary",
          onClick: async () => {
            if (pinnedSlugs.length > 5) {
              showToast("En fazla 5 rozet seçebilirsin.", "warning");
              return false;
            }
            try {
              await api.updatePinnedBadges(pinnedSlugs);
              profile.pinned_badges = [...pinnedSlugs];
              showToast("Rozet vitrini güncellendi.", "success");
              return true;
            } catch (err) {
              showToast(err.message || "Kaydedilemedi.", "error");
              return false;
            }
          },
        },
      ],
    });

    function refreshModal() {
      const container = modalObj.modal?.querySelector(
        "#badge-picker-container",
      );
      const scrollPos = container ? container.scrollTop : 0;
      modalObj.updateContent(buildContentHtml());
      const newContainer = modalObj.modal?.querySelector(
        "#badge-picker-container",
      );
      if (newContainer) newContainer.scrollTop = scrollPos;
    }

    if (modalObj.modal) {
      modalObj.modal.addEventListener("click", (e) => {
        const btn = e.target.closest(".badge-reorder-btn");
        if (btn && !btn.disabled) {
          const action = btn.dataset.action;
          const index = parseInt(btn.dataset.index, 10);
          if (action === "up" && index > 0) {
            const temp = pinnedSlugs[index];
            pinnedSlugs[index] = pinnedSlugs[index - 1];
            pinnedSlugs[index - 1] = temp;
            refreshModal();
          } else if (action === "down" && index < pinnedSlugs.length - 1) {
            const temp = pinnedSlugs[index];
            pinnedSlugs[index] = pinnedSlugs[index + 1];
            pinnedSlugs[index + 1] = temp;
            refreshModal();
          }
        }
      });

      modalObj.modal.addEventListener("change", (e) => {
        const input = e.target.closest('input[data-action="toggle-pin"]');
        if (input) {
          const slug = input.dataset.slug;
          if (input.checked) {
            if (pinnedSlugs.length < 5 && !pinnedSlugs.includes(slug)) {
              pinnedSlugs.push(slug);
            }
          } else {
            pinnedSlugs = pinnedSlugs.filter((s) => s !== slug);
          }
          refreshModal();
        }
      });
    }
  }

  // --- Setup / Owner actions ---
  function openBioEditModal() {
    const modalObj = createModal({
      title: "Biyografiyi düzenle",
      iconHtml: icon("edit", 24),
      contentHtml: `
        <div class="c-modal__form-group">
          <div class="form-group">
            <textarea id="edit-bio" rows="5"
              placeholder="Kendinden bahset..." 
              maxlength="256">${sanitizeText(profile.bio || "")}</textarea>
          </div>
        </div>
      `,
      buttons: [
        { label: "Vazgeç", variant: "secondary" },
        {
          label: "Güncelle",
          variant: "primary",
          onClick: async (modalEl) => {
            const bio = modalEl.querySelector("#edit-bio").value.trim();
            try {
              await api.updateProfile({ bio });
              if (globalState?.user && globalState.user.id === profile.id) {
                globalState.user.bio = bio;
              }
              profile.bio = bio;
              showToast("Biyografin güncellendi!", "success");
              return true;
            } catch (err) {
              showToast(err.message, "error");
              return false;
            }
          },
        },
      ],
    });
    const modalEl = modalObj.modal;
    const textarea = modalEl.querySelector("#edit-bio");
    const saveBtn = modalEl.querySelector(".btn--primary");
    initCharCounter(textarea, {
      onUpdate: (_count, limit, isOver) => {
        saveBtn.disabled = isOver;
      },
    });
    textarea.focus();
  }

  function openAvatarManageModal() {
    const hasPhoto = !!profile.avatar_url;
    const modalObj = createModal({
      title: "Profil fotoğrafı",
      contentHtml: `
        <div class="avatar-manage" id="avatar-manage-root">
          <div class="avatar-manage__preview">
            ${profile.avatar_url ? `<img src="${api.getAvatarUrl(profile.avatar_url)}?v=${avatarTimestamp}" alt="Önizleme">` : icon("avatarEmpty", 160)}
          </div>
          <input type="file" id="avatar-file-input" accept="image/*" hidden>
        </div>
      `,
      buttons: [
        {
          label: hasPhoto ? "Yeni Fotoğraf" : "Fotoğraf Yükle",
          variant: "primary",
          onClick: (modalEl) => {
            modalEl.querySelector("#avatar-file-input").click();
            return false;
          },
        },
        ...(hasPhoto
          ? [
              {
                label: "Fotoğrafı Sil",
                variant: "danger",
                onClick: () => {
                  createModal({
                    title: "Fotoğrafı Sil",
                    iconHtml: icon("warning", 32),
                    contentHtml:
                      '<p class="modal-confirm-text">Profil fotoğrafını silmek istediğine emin misin?</p>',
                    buttons: [
                      { label: "Vazgeç", variant: "secondary" },
                      {
                        label: "Evet, Sil",
                        variant: "danger",
                        onClick: async () => {
                          try {
                            await api.deleteAvatar();
                            if (
                              globalState?.user &&
                              globalState.user.id === profile.id
                            ) {
                              globalState.user.avatar_url = null;
                            }
                            profile.avatar_url = null;
                            avatarTimestamp = Date.now();
                            showToast("Profil fotoğrafın silindi.", "success");
                            modalObj.close();
                            return true;
                          } catch (err) {
                            showToast(err.message, "error");
                            return false;
                          }
                        },
                      },
                    ],
                  });
                  return false;
                },
              },
            ]
          : []),
        { label: "İptal", variant: "secondary" },
      ],
    });

    const fileInput = modalObj.modal.querySelector("#avatar-file-input");
    fileInput.onchange = (e) => {
      const file = e.target.files[0];
      if (file) openCropper(modalObj, file, profile);
    };
  }

  // --- Avatar Cropper Integration ---
  function openCropper(parentModal, file, profile) {
    const reader = new FileReader();
    reader.onload = (e) => {
      const img = new Image();
      img.onload = () => startCropping(parentModal, img, profile);
      img.src = e.target.result;
    };
    reader.readAsDataURL(file);
  }

  function startCropping(modalObj, img, profile) {
    const root = modalObj.modal.querySelector("#avatar-manage-root");
    modalObj.updateTitle("Fotoğrafı Hizala");

    root.innerHTML = `
      <div class="avatar-cropper">
        <div class="cropper-container" id="cropper-container">
          <canvas class="cropper-canvas" id="cropper-canvas"></canvas>
          <div class="cropper-overlay"></div>
        </div>
        <div class="cropper-controls">
          <button class="zoom-btn" id="zoom-out" title="Uzaklaştır">${icon("minus", 18)}</button>
          <div class="zoom-slider-wrapper">
            <input type="range" class="c-range cropper-zoom" id="cropper-zoom" step="0.01">
          </div>
          <button class="zoom-btn" id="zoom-in" title="Yakınlaştır">${icon("plus", 18)}</button>
        </div>
        <p class="cropper-help">Görseli sürükleyerek hizalayın, tekerlek ile yakınlaştırın.</p>
      </div>
    `;

    const footer = modalObj.modal.querySelector(".c-modal__footer");
    footer.innerHTML = "";

    const cancelBtn = document.createElement("button");
    cancelBtn.className = "btn btn--secondary btn--squish";
    cancelBtn.textContent = "İptal";
    cancelBtn.onclick = () => openAvatarManageModal();

    const saveBtn = document.createElement("button");
    saveBtn.className = "btn btn--primary btn--squish";
    saveBtn.textContent = "Kaydet";

    footer.appendChild(cancelBtn);
    footer.appendChild(saveBtn);

    const canvas = root.querySelector("#cropper-canvas");
    const cropper = new AvatarCropper(canvas, img);
    const zoomInput = root.querySelector("#cropper-zoom");
    const zoomIn = root.querySelector("#zoom-in");
    const zoomOut = root.querySelector("#zoom-out");

    const updateZoom = (val) => {
      zoomInput.value = val;
      cropper.setZoom(parseFloat(val));
    };

    zoomInput.oninput = () => cropper.setZoom(parseFloat(zoomInput.value));
    zoomIn.onclick = () =>
      updateZoom(Math.min(3, parseFloat(zoomInput.value) + 0.2));
    zoomOut.onclick = () =>
      updateZoom(
        Math.max(parseFloat(zoomInput.min), parseFloat(zoomInput.value) - 0.2),
      );

    const minZoom = 220 / Math.min(img.width, img.height);
    zoomInput.min = minZoom;
    zoomInput.max = minZoom * 4;
    cropper.minScale = minZoom;
    cropper.maxScale = minZoom * 4;
    zoomInput.value = Math.max(minZoom, minZoom * 1.2);
    cropper.setZoom(parseFloat(zoomInput.value));

    saveBtn.onclick = async () => {
      saveBtn.disabled = true;
      saveBtn.innerHTML =
        '<div class="loading-spinner loading-spinner--xs"></div>';
      try {
        const blob = await cropper.getCroppedBlob();
        const formData = new FormData();
        formData.append("file", blob, "avatar.jpg");
        const uploadRes = await api.uploadAvatar(formData);
        const rawUrl = uploadRes.avatar_url;
        const freshTimestamp = Date.now();
        const urlWithCacheBuster = `${rawUrl}?t=${freshTimestamp}`;
        if (globalState?.user && globalState.user.id === profile.id) {
          globalState.user.avatar_url = urlWithCacheBuster;
          try {
            localStorage.setItem(
              "kepce_user_cache",
              JSON.stringify(globalState.user),
            );
          } catch {}
        }
        profile.avatar_url = rawUrl;
        avatarTimestamp = freshTimestamp;
        showToast("Profil fotoğrafı güncellendi!", "success");
        window.dispatchEvent(
          new CustomEvent("avatar-updated", {
            detail: { avatar_url: urlWithCacheBuster },
          }),
        );
        modalObj.close();
      } catch (err) {
        showToast(err.message, "error");
        saveBtn.disabled = false;
        saveBtn.textContent = "Kaydet";
      }
    };
  }

  // --- Actions ---
  function handleShare() {
    navigator.clipboard.writeText(window.location.href);
    showToast("Profil bağlantısı kopyalandı.", "success");
  }

  class AvatarCropper {
    constructor(canvas, img) {
      this.canvas = canvas;
      this.ctx = canvas.getContext("2d");
      this.img = img;
      this.canvas.width = 300 * window.devicePixelRatio;
      this.canvas.height = 300 * window.devicePixelRatio;
      this.ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
      this.scale = 1;
      this.minScale = 0.1;
      this.maxScale = 5;
      this.x = 150;
      this.y = 150;
      this.isDragging = false;
      this.lastMouse = { x: 0, y: 0 };
      this.initEvents();
      this.draw();
    }
    initEvents() {
      const container = this.canvas.parentElement;
      this.start = (e) => {
        this.isDragging = true;
        this.lastMouse = this.getPos(e);
      };
      this.move = (e) => {
        if (!this.isDragging) return;
        const pos = this.getPos(e);
        this.x += pos.x - this.lastMouse.x;
        this.y += pos.y - this.lastMouse.y;
        this.constrain();
        this.lastMouse = pos;
        this.draw();
      };
      this.end = () => (this.isDragging = false);
      container.onmousedown = this.start;
      window.addEventListener("mousemove", this.move);
      window.addEventListener("mouseup", this.end);
      this.touchStart = (e) => this.start(e.touches[0]);
      this.touchMove = (e) => this.move(e.touches[0]);
      container.ontouchstart = this.touchStart;
      window.addEventListener("touchmove", this.touchMove, { passive: false });
      window.addEventListener("touchend", this.end);
      container.onwheel = (e) => {
        e.preventDefault();
        const delta = e.deltaY > 0 ? 0.95 : 1.05;
        this.setZoom(this.scale * delta);
        const slider = document.getElementById("cropper-zoom");
        if (slider) slider.value = this.scale;
      };
      this._observer = new MutationObserver(() => {
        if (!document.body.contains(this.canvas)) this.destroy();
      });
      this._observer.observe(document.body, { childList: true, subtree: true });
    }
    destroy() {
      window.removeEventListener("mousemove", this.move);
      window.removeEventListener("mouseup", this.end);
      window.removeEventListener("touchmove", this.touchMove);
      window.removeEventListener("touchend", this.end);
      if (this._observer) this._observer.disconnect();
    }
    getPos(e) {
      const rect = this.canvas.getBoundingClientRect();
      return { x: e.clientX - rect.left, y: e.clientY - rect.top };
    }
    setZoom(s) {
      const oldScale = this.scale;
      this.scale = Math.max(this.minScale, Math.min(this.maxScale, s));
      this.x = 150 - (150 - this.x) * (this.scale / oldScale);
      this.y = 150 - (150 - this.y) * (this.scale / oldScale);
      this.constrain();
      this.draw();
    }
    constrain() {
      const w = (this.img.width * this.scale) / 2;
      const h = (this.img.height * this.scale) / 2;
      this.x = Math.max(260 - w, Math.min(40 + w, this.x));
      this.y = Math.max(260 - h, Math.min(40 + h, this.y));
    }
    draw() {
      this.ctx.clearRect(0, 0, 300, 300);
      const w = this.img.width * this.scale;
      const h = this.img.height * this.scale;
      this.ctx.drawImage(this.img, this.x - w / 2, this.y - h / 2, w, h);
    }
    getCroppedBlob() {
      return new Promise((resolve) => {
        const output = document.createElement("canvas");
        output.width = 512;
        output.height = 512;
        const octx = output.getContext("2d");
        octx.drawImage(
          this.canvas,
          (150 - 110) * window.devicePixelRatio,
          (150 - 110) * window.devicePixelRatio,
          220 * window.devicePixelRatio,
          220 * window.devicePixelRatio,
          0,
          0,
          512,
          512,
        );
        output.toBlob((blob) => resolve(blob), "image/jpeg", 0.9);
      });
    }
  }
</script>

<Seo
  title={profile ? `@${safeNickname} - Kepçe` : "Kullanıcı Profili - Kepçe"}
  description={safeBio ||
    `${safeNickname} adlı kullanıcının Kepçe öğrenci profili ve yemek yorumları.`}
  image={`https://kepce.org/api/v1/public/og/user/${username}`}
  noindex={true}
/>

{#if loading}
  <div class="loading-full">
    <div class="loading-spinner"></div>
    <p>Profil yükleniyor...</p>
  </div>
{:else if error}
  <div class="empty-state-container">
    <EmptyState
      iconName={error.status === 404 ? "warning" : "info"}
      title={error.status === 404 ? "Kullanıcı bulunamadı" : "Hata Oluştu"}
      desc={error.message}
    />
  </div>
{:else}
  <div class="profile-page">
    <section class="profile-intro profile-card">
      <div class="profile-intro__header">
        <div class="profile-intro__avatar-group">
          {#if isOwner}
            <button
              class="profile-intro__avatar profile-intro__avatar--owner"
              id="avatar-trigger"
              onclick={openAvatarManageModal}
              aria-label="Profil fotoğrafını düzenle"
            >
              {#if profile.avatar_url}
                <img
                  src="{api.getAvatarUrl(
                    profile.avatar_url,
                  )}?v={avatarTimestamp}"
                  alt={safeNickname}
                  onerror={(e) => {
                    e.target.onerror = null;
                    e.target.outerHTML = icon("avatarEmpty", 160).replace(
                      /[\r\n]+/g,
                      "",
                    );
                  }}
                />
              {:else}
                {@html icon("avatarEmpty", 160)}
              {/if}
              <div class="profile-intro__avatar-overlay"></div>
            </button>
          {:else}
            <div class="profile-intro__avatar" id="avatar-display">
              {#if profile.avatar_url}
                <img
                  src="{api.getAvatarUrl(
                    profile.avatar_url,
                  )}?v={avatarTimestamp}"
                  alt={safeNickname}
                  onerror={(e) => {
                    e.target.onerror = null;
                    e.target.outerHTML = icon("avatarEmpty", 160).replace(
                      /[\r\n]+/g,
                      "",
                    );
                  }}
                />
              {:else}
                {@html icon("avatarEmpty", 160)}
              {/if}
            </div>
          {/if}
        </div>

        <div class="profile-intro__info-stack">
          <div class="profile-intro__name-section">
            <div class="profile-intro__name-row">
              <h1 class="profile-intro__name" data-full-name="@{safeNickname}">
                @{safeNickname}
              </h1>
              {#if profile.is_blocked}
                <div class="profile-intro__flairs">
                  <span class="profile-flair profile-intro__badge--blocked"
                    >Engellendi</span
                  >
                </div>
              {:else if getFlairs(profile).length > 0}
                <div class="profile-intro__flairs">
                  {#each getFlairs(profile) as f}
                    <span class="profile-flair {f.cls}">{f.text}</span>
                  {/each}
                </div>
              {/if}
            </div>
          </div>

          {#if profile.level_progress?.title || profile.karma_score !== undefined}
            <div class="profile-intro__flair-rank">
              {profile.level_progress?.title || "düz tabldotçu"} ({profile.karma_score ?? 0})
            </div>
          {/if}

          <div class="profile-intro__achievements-dock">
            {#if showcasedBadges.length > 0}
              <div class="profile-intro__achievements">
                {#each showcasedBadges as a}
                  <div
                    class="achievement-badge badge--{getBadgeTier(a)}"
                    title="{a.name}{a.description ? ': ' + a.description : ''}"
                  >
                    {@html icon(a.icon || "starFilled", 20)}
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      </div>

      <div class="profile-intro__content" id="bio-container">
        {#if profile.bio}
          <p class="profile-intro__bio">{safeBio}</p>
        {/if}
      </div>

      <div class="profile-intro__footer">
        <div class="profile-intro__joined">
          {@html icon("calendar", 14)}
          {createdDate}
        </div>
        <div class="profile-intro__actions">
          <button
            class="btn btn--secondary btn--squish btn--icon-only"
            onclick={handleShare}
            title="Paylaş"
          >
            {@html icon("share", 16)}
          </button>
          <ActionMenu
            triggerClass="btn btn--secondary btn--squish btn--icon-only"
            triggerTitle="Daha fazla"
            items={[
              ...(isOwner
                ? [
                    {
                      label: "Biyografiyi düzenle",
                      onClick: () => openBioEditModal(),
                    },
                    ...(getEarnedBadges(profile).length > 0
                      ? [
                          {
                            label: "Rozet vitrinini düzenle",
                            onClick: () => openBadgeShowcaseModal(),
                          },
                        ]
                      : []),
                  ]
                : [
                    ...(profile.is_blocked
                      ? [
                          {
                            label: "Engeli kaldır",
                            onClick: () => handleUnblock(),
                          },
                        ]
                      : [
                          {
                            label: "Kullanıcıyı engelle",
                            onClick: () => handleBlock(),
                          },
                        ]),
                    {
                      label: "Şikayet et",
                      variant: "danger",
                      onClick: () => openUserReportModal(profile.id),
                    },
                  ]),
            ]}
          />
        </div>
      </div>
    </section>

    {#if profile.is_blocked}
      <div class="profile-blocked-notice">
        <EmptyState
          iconName="lock"
          title="Bu Kullanıcıyı Engelledin"
          desc="Engellediğin kullanıcıların yorumları ve profil aktiviteleri gizlenir."
        >
          <button
            class="btn btn--secondary btn--squish"
            onclick={handleUnblock}
          >
            Engeli Kaldır
          </button>
        </EmptyState>
      </div>
    {:else if profile.is_blocked_by}
      <div class="profile-blocked-notice">
        <EmptyState
          iconName="lock"
          title="Bu Profile Erişim Kısıtlandı"
          desc="Bu kullanıcının paylaşımlarını görüntüleyemezsin."
        />
      </div>
    {:else}
      <!-- ── Tab Navigation ─────────────── -->
      <TabBar
        bind:activeId={activeTab}
        tabs={Object.entries(tabStructure).map(([id, tab]) => ({
          id,
          label: tab.label,
          href: tab.path,
          icon: tab.icon,
        }))}
      />

      <!-- ── Content Grid ───────────────── -->
      <div class="profile-grid">
        {@render children()}
      </div>
    {/if}
  </div>
{/if}
