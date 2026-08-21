<script>
  import { api } from "@/api/index.js";
  import Loader from "@/components/ui/Loader.svelte";
  import TabBar from "@/components/ui/TabBar.svelte";
  import { icon } from "@/components/ui/icons.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import { sanitizeText } from "@/utils/sanitize.js";
  import { afterNavigate } from "$app/navigation";
  import { slide } from "svelte/transition";
  import { getDuration } from "@/lib/dom/motion.js";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import { getCitiesData } from "@/stores/city.svelte.js";
  import Seo from "@/components/ui/Seo.svelte";

  const subTabs = [
    { id: "top_rated", label: "En Beğenilenler", icon: icon("starFilled", 18) },
    {
      id: "worst_rated",
      label: "Nefret Tablosu",
      icon: icon("strongLanguage", 18),
    },
    { id: "pulse", label: "Genel vaziyet", icon: icon("usage", 18) },
  ];

  let activeSubTab = $state("top_rated");
  let selectedCity = $state("");
  let selectedTimeframe = $state("");
  let cachedCities = $state([]);
  let timeframes = [
    { value: "", label: "Tümü" },
    { value: "daily", label: "Dün" },
    { value: "weekly", label: "Geçen Hafta" },
    { value: "monthly", label: "Geçen Ay" },
    { value: "yearly", label: "Geçen Yıl" },
  ];

  let isLoading = $state(true);
  let errorMsg = $state(null);
  let errorCode = $state(null);
  let contentData = $state(null);
  let currentLoadToken = 0;

  afterNavigate(async () => {
    try {
      cachedCities = (await getCitiesData()) || [];
    } catch (e) {
      cachedCities = [];
    }
    loadContent();
  });

  async function loadContent() {
    isLoading = true;
    errorMsg = null;
    contentData = null;

    const token = ++currentLoadToken;
    try {
      let data;
      if (activeSubTab === "top_rated") {
        data = await api.getTopDishes(10, selectedCity, selectedTimeframe);
      } else if (activeSubTab === "worst_rated") {
        data = await api.getWorstDishes(10, selectedCity, selectedTimeframe);
      } else if (activeSubTab === "pulse") {
        data = (await api.getTrendingTags(15)) || [];
      }
      if (token !== currentLoadToken) return;
      contentData = data;
    } catch (err) {
      if (token !== currentLoadToken) return;
      errorMsg = err.message || "Bir hata oluştu.";
      errorCode = err.status || 500;
    } finally {
      if (token === currentLoadToken) {
        isLoading = false;
      }
    }
  }

  function handleCityChange() {
    loadContent();
  }

  function handleTimeframeChange() {
    loadContent();
  }

  const tagSentiments = {
    yenur: "positive",
    şaşırttı: "positive",
    "tam bir şifa": "positive",
    "protein bombası": "positive",
    "süper olmuş": "positive",
    efsane: "positive",
    doyurucu: "positive",
    "harika görünüyor": "positive",
    favorim: "positive",
    "bugün çok güzel": "positive",
    "bıktım artık": "negative",
    rezaletti: "negative",
    "mide fesadı": "negative",
    yenmez: "negative",
    tatsızdı: "negative",
    berbattı: "negative",
    kötüydü: "negative",
    olmamış: "negative",
    "yiyeceklere yazık olmuş": "negative",
    "olay yerinde olacağım": "positive",
    "dışarıdan söyleyin": "negative",
    "uzak durun": "negative",
    "ekmek arası yapın": "neutral",
    "koşun gelin": "positive",
    "idare eder": "neutral",
    "boykot zamanı": "negative",
    "herkesi davet ediyorum": "positive",
    güzeldi: "positive",
    "fena değildi": "neutral",
    koşun: "positive",
  };

  function getMonthNameForPulse() {
    const monthNames = [
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
    const currentDate = new Date();
    const pastMonthIdx = (currentDate.getMonth() - 1 + 12) % 12;
    return monthNames[pastMonthIdx];
  }

  function actionStagger(node, idx) {
    node.style.setProperty("--stagger-idx", idx);
  }

  function actionPulseBar(node, { width }) {
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        node.style.setProperty("--width", `${width}%`);
      });
    });
    return {
      update({ width: newWidth }) {
        node.style.setProperty("--width", `${newWidth}%`);
      },
    };
  }

  let cityOptions = $derived([
    { value: "", label: "Tümü" },
    ...cachedCities.map((c) => ({
      value: c.slug,
      label: sanitizeText(c.name),
    })),
  ]);
</script>

<Seo
  title="Yemek İstatistikleri ve Analizleri - Kepçe"
  description="KYK yurtlarında en çok sevilen ve en az beğenilen yemekler, puanlamalar ve öğrenci oylama istatistikleri."
  image="https://kepce.org/api/v1/public/og/page/istatistikler"
/>

<h1 class="sr-only">KYK Yemek İstatistikleri ve Analizleri</h1>

<TabBar bind:activeId={activeSubTab} tabs={subTabs} onChange={loadContent} />

<div id="stats-tab-content" class="c-tab-content">
  {#if isLoading}
    <div class="stats-placeholder">
      <Loader size={48} />
    </div>
  {:else if errorMsg}
    <EmptyState statusCode={errorCode} desc={errorMsg} />
  {:else if contentData}
    {#if activeSubTab === "top_rated" || activeSubTab === "worst_rated"}
      {@const isDanger = activeSubTab === "worst_rated"}
      {@const visibleDishes = contentData.filter((d) =>
        isDanger ? (d.score ?? 0) < 0 : (d.score ?? 0) > 0,
      )}
      <section class="stat-card">
        <h2 class="stats-section-title">
          <span>{isDanger ? "Nefret Tablosu" : "En Sevilen Yemekler"}</span>
          <div class="stats-filters">
            <Dropdown
              options={timeframes}
              bind:value={selectedTimeframe}
              onChange={handleTimeframeChange}
              placeholder="Zaman Aralığı"
            />
            <Dropdown
              options={cityOptions}
              bind:value={selectedCity}
              onChange={handleCityChange}
              placeholder="Şehir seçin"
            />
          </div>
        </h2>
        <div class="stats-list">
          {#if visibleDishes.length === 0}
            <EmptyState
              iconName={"info"}
              title={"Veri bekleniyor"}
              desc={"Henüz yeterli oy verisi toplanmadı."}
            />
          {:else}
            {#each visibleDishes as dish, idx}
              {@const isTop = idx < 3}
              {@const rankClass = isTop ? "rank-top" : ""}
              {@const pct = Math.round((dish.average_rating || 0) * 100)}
              <div class="stats-item {rankClass}" use:actionStagger={idx}>
                <span
                  class="stats-item__rank {isTop && isDanger
                    ? idx === 0
                      ? 'stats-item__rank--negative'
                      : 'stats-item__rank--disclaimer'
                    : ''}"
                >
                  {#if isTop && !isDanger}
                    {#if idx === 0}{@html icon(
                        "starFilled",
                        24,
                      )}{:else if idx === 1}{@html icon(
                        "starFilledHalfLeft",
                        20,
                      )}{:else}{@html icon("star", 20)}{/if}
                  {:else if isTop && isDanger}
                    {#if idx === 0}{@html icon(
                        "strongLanguage",
                        24,
                      )}{:else}{@html icon("strongLanguage", 20)}{/if}
                  {:else}
                    #{idx + 1}
                  {/if}
                </span>
                <div class="stats-item__content">
                  <div class="stats-item__name">{sanitizeText(dish.name)}</div>
                  <div class="stats-item__category">
                    {sanitizeText(dish.category || "Ana Yemek")}
                  </div>
                </div>
                <div class="stats-item__value">
                  <div class="stats-item__score">{pct}%</div>
                  <div class="stats-item__meta">{dish.vote_count} oy</div>
                </div>
              </div>
            {/each}
          {/if}
        </div>
      </section>
    {:else if activeSubTab === "pulse"}
      {@const sentimentTags = contentData.filter(
        (t) => t.category === "sentiment",
      )}
      {@const recommendationTags = contentData.filter(
        (t) => t.category === "recommendation",
      )}
      {@const maxSentiment =
        sentimentTags.length > 0
          ? Math.max(...sentimentTags.map((t) => t.count))
          : 1}
      {@const maxRecommendation =
        recommendationTags.length > 0
          ? Math.max(...recommendationTags.map((t) => t.count))
          : 1}
      {@const monthlyJargon = contentData.filter(
        (t) => t.category === "jargon",
      )}
      {@const maxJargon =
        monthlyJargon.length > 0
          ? Math.max(...monthlyJargon.map((t) => t.count))
          : 1}

      <section class="stat-card">
        <h2 class="stats-section-title">Genel vaziyet</h2>
        <div class="pulse-grid">
          <div class="pulse-column">
            <h3 class="pulse-column-title">Sindirim raporu</h3>
            <div class="tag-stats-container">
              {#if sentimentTags.length > 0}
                {#each sentimentTags as tag, idx}
                  {@const sentiment =
                    tag.sentiment ||
                    tagSentiments[tag.name.toLowerCase()] ||
                    "neutral"}
                  {@const width = Math.max((tag.count / maxSentiment) * 100, 5)}
                  <div class="tag-stat-bar-wrapper" use:actionStagger={idx}>
                    <div class="tag-stat-header">
                      <span class="tag-stat-name">{sanitizeText(tag.name)}</span
                      >
                      <span class="tag-stat-count">{tag.count} Kez</span>
                    </div>
                    <div class="tag-stat-track">
                      <div
                        class="tag-stat-fill tag-stat-fill--{sentiment}"
                        use:actionPulseBar={{ width }}
                      ></div>
                    </div>
                  </div>
                {/each}
              {:else}
                <div class="pulse-empty">Henüz veri yok.</div>
              {/if}
            </div>
          </div>

          <div class="pulse-column">
            <h3 class="pulse-column-title">Kamu spotu</h3>
            <div class="tag-stats-container">
              {#if recommendationTags.length > 0}
                {#each recommendationTags as tag, idx}
                  {@const sentiment =
                    tag.sentiment ||
                    tagSentiments[tag.name.toLowerCase()] ||
                    "neutral"}
                  {@const width = Math.max(
                    (tag.count / maxRecommendation) * 100,
                    5,
                  )}
                  <div
                    class="tag-stat-bar-wrapper"
                    use:actionStagger={idx + sentimentTags.length}
                  >
                    <div class="tag-stat-header">
                      <span class="tag-stat-name">{sanitizeText(tag.name)}</span
                      >
                      <span class="tag-stat-count">{tag.count} Kez</span>
                    </div>
                    <div class="tag-stat-track">
                      <div
                        class="tag-stat-fill tag-stat-fill--{sentiment}"
                        use:actionPulseBar={{ width }}
                      ></div>
                    </div>
                  </div>
                {/each}
              {:else}
                <div class="pulse-empty">Henüz veri yok.</div>
              {/if}
            </div>
          </div>

          <div class="pulse-column">
            <h3 class="pulse-column-title">
              Aylık Jargon ({getMonthNameForPulse()})
            </h3>
            <div class="tag-stats-container">
              {#if monthlyJargon.length > 0}
                {#each monthlyJargon as word, idx}
                  {@const width = Math.max((word.count / maxJargon) * 100, 5)}
                  <div
                    class="tag-stat-bar-wrapper"
                    use:actionStagger={idx +
                      sentimentTags.length +
                      recommendationTags.length}
                  >
                    <div class="tag-stat-header">
                      <span class="tag-stat-name"
                        >{sanitizeText(word.name)}</span
                      >
                      <span class="tag-stat-count">{word.count} Kez</span>
                    </div>
                    <div class="tag-stat-track">
                      <div
                        class="tag-stat-fill tag-stat-fill--primary"
                        use:actionPulseBar={{ width }}
                      ></div>
                    </div>
                  </div>
                {/each}
              {:else}
                <div class="pulse-empty">Henüz veri yok.</div>
              {/if}
            </div>
          </div>
        </div>
      </section>
    {/if}
  {/if}
</div>
