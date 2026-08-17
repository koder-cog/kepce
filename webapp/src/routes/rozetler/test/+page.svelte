<script>
    import { icon, icons } from "@/components/ui/icons.js";
    import { sanitizeText } from "@/utils/sanitize.js";
    import Seo from "@/components/ui/Seo.svelte";

    const CATEGORY_META = {
        sadakat: {
            title: "Sadakat ve İstikrar",
            subtitle: "Çaresizlik Sınavı",
            icon: "calendar",
        },
        sosyal: {
            title: "Sosyal Etkileşim",
            subtitle: "Yemekhane Dedikodusu",
            icon: "chat",
        },
        denetim: {
            title: "Denetim ve Kalite",
            subtitle: "Sistemin Bedava Bekçileri",
            icon: "search",
        },
        veri: {
            title: "Veri Katkısı",
            subtitle: "Tedarikçi Manyaklar",
            icon: "plus",
        },
    };

    const SAMPLE_BADGES = [
        {
            slug: "ilk_kepce",
            name: "İlk Kepçe",
            description:
                "Menü yayınlandığı an ilk 5 dakika içinde ilk yorumu giren işsiz.",
            category: "sadakat",
            karma_reward: 25,
            is_repeatable: true,
            icon: "soup",
        },
        {
            slug: "demir_mide",
            name: "Demir Mide",
            description: "7 gün üst üste siteye giriş yapan felaketzede.",
            category: "sadakat",
            karma_reward: 50,
            is_repeatable: false,
            icon: "shield",
        },
        {
            slug: "kurumsal_caresizlik",
            name: "Kurumsal Çaresizlik",
            description: "30 gün üst üste siteye giren amansız.",
            category: "sadakat",
            karma_reward: 150,
            is_repeatable: false,
            icon: "calendar",
        },
        {
            slug: "stokholm_sendromu",
            name: "Stokholm Sendromu",
            description: "100 gün üst üste siteye giren yurt kuşu.",
            category: "sadakat",
            karma_reward: 400,
            is_repeatable: false,
            icon: "lock",
        },
        {
            slug: "demirbas",
            name: "Demirbaş",
            description:
                "Bir eğitim-öğretim dönemi boyunca (ekim-haziran) üst üste siteye giren gariban.",
            category: "sadakat",
            karma_reward: 750,
            is_repeatable: true,
            icon: "crown",
        },
        {
            slug: "hucre_hapsi",
            name: "Hücre Hapsi",
            description: "365 gün (1 yıl) aralıksız siteye giren hayatsız.",
            category: "sadakat",
            karma_reward: 1500,
            is_repeatable: false,
            icon: "lock",
        },
        {
            slug: "vefakar",
            name: "Vefakar",
            description:
                "Bir ay boyunca bir şehirdeki tüm menülere upvote veren polyanna.",
            category: "sadakat",
            karma_reward: 100,
            is_repeatable: true,
            icon: "starFilled",
        },
        {
            slug: "klavyesor",
            name: "Klavyeşör",
            description: "Toplam 100 yoruma ulaşan.",
            category: "sosyal",
            karma_reward: 100,
            is_repeatable: false,
            icon: "chat",
        },
        {
            slug: "halkin_adami",
            name: "Halkın Adamı",
            description: "Tek bir yorumuyla 50 upvote alan.",
            category: "sosyal",
            karma_reward: 200,
            is_repeatable: true,
            icon: "voteUpFilled",
        },
        {
            slug: "muzmin_muhalif",
            name: "Müzmin Muhalif",
            description: "Üst üste herhangi bir şeye 50 downvote atan.",
            category: "sosyal",
            karma_reward: 50,
            is_repeatable: false,
            icon: "voteDown",
        },
        {
            slug: "kanaat_onderi",
            name: "Kanaat Önderi",
            description: "Toplam 500 upvote alan.",
            category: "sosyal",
            karma_reward: 500,
            is_repeatable: false,
            icon: "crown",
        },
        {
            slug: "linc_kurbani",
            name: "Linç Kurbanı",
            description: "Tek bir yorumunda 50 downvote yiyen.",
            category: "sosyal",
            karma_reward: 75,
            is_repeatable: true,
            icon: "warning",
        },
        {
            slug: "caylak_gammaz",
            name: "Çaylak Gammaz",
            description: "İlk şikayetini/raporunu yapan hevesli ispiyoncu.",
            category: "denetim",
            karma_reward: 25,
            is_repeatable: false,
            icon: "search",
        },
        {
            slug: "fahri_mufettis",
            name: "Fahri Müfettiş",
            description: "10 başarılı şikayetle sanal egosu okşanan.",
            category: "denetim",
            karma_reward: 500,
            is_repeatable: false,
            icon: "check-circle",
        },
        {
            slug: "isguzar",
            name: "İşgüzar",
            description: "50 başarılı şikayetle uygulamanın ücretsiz amelesi.",
            category: "denetim",
            karma_reward: 5000,
            is_repeatable: false,
            icon: "shield",
        },
        {
            slug: "cesnicibassi",
            name: "Çeşnicibaşı",
            description: "Menüyü şehre ilk sızdırıp onaylatan kobay.",
            category: "veri",
            karma_reward: 50,
            is_repeatable: false,
            icon: "plus",
        },
        {
            slug: "karbonhidrat_elcisi",
            name: "Karbonhidrat Elçisi",
            description:
                "İçinde 'Makarna' veya 'Ekmek' geçen 30 öğünü favorileyen gizli insülin direnci.",
            category: "veri",
            karma_reward: 100,
            is_repeatable: false,
            icon: "soup",
        },
        {
            slug: "tabldot_kahini",
            name: "Tabldot Kâhini",
            description:
                "Menüyü herkesten önce sisteme giren vizyoner (Toplam 10 kez).",
            category: "veri",
            karma_reward: 200,
            is_repeatable: false,
            icon: "sun",
        },
    ];

    let badgesState = $state(
        SAMPLE_BADGES.map((b) => ({ ...b, unlocked: false })),
    );

    let groupedBadges = $derived(() => {
        const grouped = {};
        for (const badge of badgesState) {
            const cat = badge.category || "diger";
            if (!grouped[cat]) grouped[cat] = [];
            grouped[cat].push(badge);
        }
        return grouped;
    });

    function toggleBadge(slug) {
        const badge = badgesState.find((b) => b.slug === slug);
        if (badge) {
            badge.unlocked = !badge.unlocked;
        }
    }
</script>

<Seo title="Rozet Test Atölyesi - Kepçe" noindex={true} />

<div class="achievements-page fade-in">
    <header class="achievements-header achievements-header--centered">
        <h1 class="achievements-header__nickname">Rozet test atölyesi</h1>
        <p class="achievements-header__rank">
            Rozetlerin kilidini açmak / kapatmak için üzerlerine tıkla.
        </p>
    </header>

    {#each Object.entries(groupedBadges()) as [catKey, catBadges]}
        {@const meta = CATEGORY_META[catKey] || {
            title: catKey,
            subtitle: "",
            icon: "info",
        }}
        <section class="achievements-section">
            <h2 class="achievements-section__title">
                {@html icon(meta.icon, 24)}
                {meta.title}
                <span class="achievements-section__subtitle"
                    >{meta.subtitle}</span
                >
            </h2>
            <div class="badge-grid">
                {#each catBadges as badge}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div
                        class="badge-item {badge.unlocked
                            ? 'badge-item--unlocked'
                            : 'badge-item--locked'} btn--squish badge-test-item"
                        title={badge.description || ""}
                        onclick={() => toggleBadge(badge.slug)}
                    >
                        <div class="badge-item__icon">
                            {@html icon(
                                badge.icon && icons[badge.icon]
                                    ? badge.icon
                                    : "starFilled",
                                56,
                            )}
                        </div>
                        <div class="badge-item__name">
                            {@html sanitizeText(badge.name)}
                        </div>
                        <div class="badge-item__meta">
                            {#if badge.unlocked}
                                <span class="badge-item__karma"
                                    >+{badge.karma_reward}</span
                                >
                            {:else}
                                <span class="badge-item__date">Kilitli</span>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        </section>
    {/each}
</div>
