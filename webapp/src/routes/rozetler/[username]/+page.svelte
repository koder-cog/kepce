<script>
  import { api } from '@/api/index.js';
  import { icon, icons } from '@/components/ui/icons.js';
  import EmptyState from '@/components/ui/EmptyState.svelte';
  import { sanitizeText } from '@/utils/sanitize.js';
  import Seo from '@/components/ui/Seo.svelte';
  import { onMount } from 'svelte';

  const CATEGORY_META = {
    sadakat: { title: 'Sadakat ve İstikrar', subtitle: 'Çaresizlik Sınavı', icon: 'calendar' },
    sosyal: { title: 'Sosyal Etkileşim', subtitle: 'Yemekhane Dedikodusu', icon: 'chat' },
    denetim: { title: 'Denetim ve Kalite', subtitle: 'Sistemin Bedava Bekçileri', icon: 'search' },
    veri: { title: 'Veri Katkısı', subtitle: 'Tedarikçi Manyaklar', icon: 'plus' },
  };

  import { page } from '$app/stores';
  let params = $derived($page.params);
  let username = $derived(params?.username);

  let loading = $state(true);
  let profile = $state(null);
  let error = $state(null);

  $effect(() => {
    if (username) {
      loadProfile(username);
    } else {
      loading = false;
      error = { status: 404, message: 'Kullanıcı Bulunamadı. Hangi şefin rozetlerine bakmak istiyorsun?' };
    }
  });

  async function loadProfile(uname) {
    loading = true;
    error = null;
    profile = null;
    try {
      profile = await api.getPublicProfile(uname);
    } catch (err) {
      error = err;
    } finally {
      loading = false;
    }
  }

  let safeNickname = $derived(sanitizeText(profile?.username || 'isimsiz'));
  let badges = $derived(profile?.badges || []);
  let levelProgress = $derived(profile?.level_progress || {});
  let level = $derived(levelProgress.level || 0);
  let title = $derived(levelProgress.title || 'yeni kayıt');
  let karma = $derived(profile?.karma_score || 0);
  let totalUnlocked = $derived(profile?.badge_count || 0);
  let totalBadges = $derived(profile?.total_badges || badges.length);
  let progressPercent = $derived(levelProgress.progress_percent || 0);

  let groupedBadges = $derived.by(() => {
    const grouped = {};
    for (const badge of badges) {
      const cat = badge.category || 'diger';
      if (!grouped[cat]) grouped[cat] = [];
      grouped[cat].push(badge);
    }
    return grouped;
  });

  let strokeDashOffset = $derived(414.69 * (1 - totalUnlocked / Math.max(totalBadges, 1)));
</script>

<Seo
  title={profile ? `@${safeNickname} Rozetleri - Kepçe` : "Kullanıcı Rozetleri - Kepçe"}
  description={profile ? `@${safeNickname} kullanıcısının kazandığı rozetler, başarımlar ve karma seviyesi.` : "Kepçe rozetler ve başarımlar."}
/>

{#if loading}
  <div class="loading-full">
    <div class="loading-spinner"></div>
    <p>Başarımlar yükleniyor...</p>
  </div>
{:else if error}
  <div class="empty-state-container">
    <EmptyState iconName={error.status === 404 ? 'warning' : 'info'} title={error.status === 404 ? 'Kullanıcı Bulunamadı' : 'Bir Hata Oluştu'} desc={error.message}>
      {#if error.status !== 404}
        <button class="btn btn--primary btn--squish" onclick={() => loadProfile(username)}>Tekrar dene</button>
      {/if}
    </EmptyState>
  </div>
{:else if profile}
  <div class="achievements-page fade-in">
    <!-- ── Header Section ──────────────── -->
    <header class="achievements-header">
      <div class="achievements-header__avatar">
        <svg class="achievements-header__ring" viewBox="0 0 140 140">
          <circle cx="70" cy="70" r="66" fill="none" stroke="rgba(255,255,255,0.1)" stroke-width="4" />
          <circle cx="70" cy="70" r="66" fill="none" stroke="var(--color-accent-primary)" stroke-width="4" 
                  stroke-dasharray="414.69" stroke-dashoffset={strokeDashOffset} 
                  transform="rotate(-90 70 70)" stroke-linecap="round" />
        </svg>
        <img src={api.getAvatarUrl(profile.avatar_url)} alt={safeNickname} 
             onerror={(e) => { e.target.onerror=null; e.target.src='data:image/svg+xml,%3Csvg xmlns=%22http://www.w3.org/2000/svg%22 width=%22120%22 height=%22120%22 viewBox=%220 0 120 120%22%3E%3Crect width=%22120%22 height=%22120%22 fill=%22%23222%22/%3E%3C/svg%3E'; }}>
      </div>
      <div class="achievements-header__info">
        <h1 class="achievements-header__nickname">@{safeNickname}</h1>
        <div class="achievements-header__rank">{title}</div>
        
        <div class="level-progress-container">
          <div class="level-progress">
            <div class="level-progress__bar" style="--progress-width: {progressPercent}%"></div>
          </div>
          <div class="level-progress__text">
            <span>{levelProgress.karma_in_level || 0} / {levelProgress.karma_for_next || 50}</span>
            <span>Sonraki Seviye: Lvl {level + 1}</span>
          </div>
        </div>
      </div>
    </header>

    <!-- ── Stats Grid ──────────────────── -->
    <div class="achievements-stats">
      <div class="achievements-stat-card btn--squish">
        <span class="achievements-stat-value">Lvl {level}</span>
        <span class="achievements-stat-label">Profil seviyesi</span>
      </div>
      <div class="achievements-stat-card btn--squish">
        <span class="achievements-stat-value">{karma}</span>
        <span class="achievements-stat-label">Toplam karma</span>
      </div>
      <div class="achievements-stat-card btn--squish">
        <span class="achievements-stat-value">{totalUnlocked} / {totalBadges}</span>
        <span class="achievements-stat-label">Rozetler</span>
      </div>
    </div>

    <!-- ── Badge Sections ──────────────── -->
    {#each Object.entries(groupedBadges) as [catKey, catBadges]}
      {@const meta = CATEGORY_META[catKey] || { title: catKey, subtitle: '', icon: 'info' }}
      <section class="achievements-section">
        <h2 class="achievements-section__title">
          {@html icon(meta.icon, 24)}
          {meta.title}
          <span class="achievements-section__subtitle">{meta.subtitle}</span>
        </h2>
        <div class="badge-grid">
          {#each catBadges as badge}
            <div class="badge-item {badge.unlocked ? 'badge-item--unlocked' : 'badge-item--locked'} btn--squish" 
                 title={badge.description || ''}>
              <div class="badge-item__icon">
                {@html icon((badge.icon && icons[badge.icon]) ? badge.icon : 'starFilled', 56)}
              </div>
              <div class="badge-item__name">{badge.name}</div>
              <div class="badge-item__meta">
                {#if badge.unlocked}
                  <span class="badge-item__karma">+{badge.karma_reward}</span>
                  {#if badge.count > 1}
                    <span class="badge-item__count">×{badge.count}</span>
                  {/if}
                {:else}
                  <span class="badge-item__date">Kilitli</span>
                {/if}
              </div>
              {#if badge.is_repeatable}
                <div class="badge-item__repeatable" title="Tekrarlanabilir">🔁</div>
              {/if}
            </div>
          {/each}
        </div>
      </section>
    {/each}
  </div>
{/if}
