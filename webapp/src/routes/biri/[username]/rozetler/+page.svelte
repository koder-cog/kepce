<script>
  import "@/styles/pages/_achievements.css";
  import { api } from "@/api/index.js";
  import { icon, icons } from "@/components/ui/icons.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import { page } from "$app/stores";
  import { getContext } from "svelte";

  const CATEGORY_TITLES = {
    sadakat: "Müdavimlik",
    sosyal: "Dedikodu",
    denetim: "Teftiş",
    veri: "Tedarik",
  };

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

  const HIDDEN_BADGE_SLUGS = new Set(["muzmin_muhalif", "linc_kurbani"]);

  function getBadgeTier(badge) {
    if (badge.tier) return badge.tier;
    if (badge.slug && BADGE_TIER_MAP[badge.slug])
      return BADGE_TIER_MAP[badge.slug];
    return "other";
  }

  let username = $derived($page.params.username);
  const getLayoutProfile = getContext("profileContext");

  let fallbackProfile = $state(null);
  let fallbackLoading = $state(false);
  let fallbackError = $state(null);

  let layoutProf = $derived(getLayoutProfile ? getLayoutProfile() : null);
  let isLayoutProfileMatch = $derived(
    Boolean(
      layoutProf &&
        layoutProf.username?.toLowerCase() === username?.toLowerCase(),
    ),
  );

  let profile = $derived(
    isLayoutProfileMatch ? layoutProf : fallbackProfile,
  );
  let loading = $derived(
    !profile && !fallbackError && (fallbackLoading || !isLayoutProfileMatch),
  );

  $effect(() => {
    if (username && !isLayoutProfileMatch && !fallbackLoading && !fallbackProfile) {
      loadFallbackProfile(username);
    }
  });

  async function loadFallbackProfile(uname) {
    fallbackLoading = true;
    fallbackError = null;
    try {
      fallbackProfile = await api.getPublicProfile(uname);
    } catch (err) {
      fallbackError = err;
    } finally {
      fallbackLoading = false;
    }
  }

  let badges = $derived(profile?.badges || []);

  let groupedBadges = $derived.by(() => {
    const grouped = {};
    for (const badge of badges) {
      const cat = badge.category || "diger";
      if (!grouped[cat]) grouped[cat] = [];
      grouped[cat].push(badge);
    }
    return grouped;
  });
</script>

{#if loading}
  <div class="loading-full">
    <div class="loading-spinner"></div>
    <p>Rozetler yükleniyor...</p>
  </div>
{:else if fallbackError}
  <div class="empty-state-container">
    <EmptyState
      iconName="warning"
      title="Rozetler Yüklenemedi"
      desc={fallbackError.message || "Bir hata oluştu"}
    >
      <button
        class="btn btn--primary btn--squish"
        onclick={() => loadFallbackProfile(username)}
      >
        Tekrar dene
      </button>
    </EmptyState>
  </div>
{:else if profile}
  {#if badges.length === 0}
    <div class="empty-state-container">
      <EmptyState
        iconName="trophy"
        title="Henüz Rozet Yok"
        desc="Bu kullanıcının henüz kazandığı bir rozet bulunmuyor veya rozet kataloğu henüz yüklenmemiş."
      />
    </div>
  {:else}
    <div class="achievements-page fade-in">
      {#each Object.entries(groupedBadges) as [catKey, catBadges]}
        {@const sectionTitle = CATEGORY_TITLES[catKey] || catKey}
        <section class="achievements-section">
          <div class="achievements-section__header">
            <h2 class="achievements-section__title">{sectionTitle}</h2>
          </div>
          <div class="badge-grid">
            {#each catBadges as badge}
              {@const tier = getBadgeTier(badge)}
              {@const isLocked = !badge.unlocked}
              {@const isHidden = badge.is_hidden || (badge.slug && (badge.slug.startsWith("hidden_") || HIDDEN_BADGE_SLUGS.has(badge.slug)))}
              {@const isHiddenAndLocked = isHidden && isLocked}
              <div
                class="badge-item {badge.unlocked
                  ? 'badge-item--unlocked'
                  : 'badge-item--locked'} badge--{tier}"
              >
                <div class="badge-circle">
                  {#if isHiddenAndLocked}
                    {@html icon("lock", 24)}
                  {:else}
                    {@html icon(
                      badge.icon && icons[badge.icon] ? badge.icon : "starFilled",
                      24,
                    )}
                  {/if}
                </div>
                <span class="badge-item__name">
                  {isHiddenAndLocked ? "Gizli Rozet" : badge.name}
                </span>
                <p class="badge-item__desc">
                  {isHiddenAndLocked
                    ? `@${profile?.username || username} kepçeyi doğru daldırıp denk getirebilirse tabağına düşer.`
                    : badge.description || ""}
                </p>
                <div class="badge-item__meta">
                  <span class="badge-item__karma">
                    {#if isHiddenAndLocked}
                      ? Puan
                    {:else}
                      +{badge.karma_reward} Puan{#if badge.is_repeatable && !badge.unlocked} <span class="badge-item__meta-sep" aria-hidden="true">•</span> Tekrar{:else if badge.is_repeatable && badge.count > 1} × {badge.count}{/if}
                    {/if}
                  </span>
                </div>
              </div>
            {/each}
          </div>
        </section>
      {/each}
    </div>
  {/if}
{/if}
