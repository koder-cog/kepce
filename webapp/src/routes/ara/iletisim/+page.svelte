<script>
  import { page } from "$app/stores";
  import { icon } from "@/components/ui/icons.js";
  import "@/styles/pages/_content.css";

  let isSubdomain = $derived(
    $page.url.hostname.startsWith("ara.") ||
      $page.url.hostname === "ara.localhost",
  );
  let basePath = $derived(isSubdomain ? "" : "/ara");
</script>

<svelte:head>
  <title>İletişim | Kepçe Ara</title>
  <meta
    name="description"
    content="Kepçe Ara geri bildirim, hata bildirimi ve iletişim kanalları."
  />
</svelte:head>

<div class="settings-page c-search-adv-page">
  <article class="content-page">
    <header class="content-page__header">
      <h1 class="content-page__title">İletişim ve Geri Bildirim</h1>
      <div class="content-page__meta">
        <time class="content-page__date">Kepçe Açık Kaynak Topluluğu</time>
      </div>
    </header>

    <div class="content-page__body">
      <p class="u-text-base u-leading-comfortable u-mb-lg">
        Hata bildirmek veya öneride bulunmak için aşağıdaki kanalları kullanabilirsiniz.
      </p>

      <div class="c-boxed-list u-mb-lg">
        <a
          href="https://github.com/koder-cog/kepce/issues"
          target="_blank"
          rel="noopener noreferrer"
          class="c-list-row c-list-row--clickable c-list-row--tall"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">GitHub Issues (Hata ve Öneri)</div>
            <div class="c-list-row__desc">Hata bildirimleri ve özellik talepleri için açık depo</div>
          </div>
          <div class="c-list-row__control">
            {@html icon("externalLink", 16)}
          </div>
        </a>

        <a
          href="mailto:iletisim@kepce.org"
          class="c-list-row c-list-row--clickable c-list-row--tall"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">E-posta</div>
            <div class="c-list-row__desc">iletisim@kepce.org</div>
          </div>
          <div class="c-list-row__control">
            {@html icon("externalLink", 16)}
          </div>
        </a>

        <a
          href="https://reddit.com/r/kepce"
          target="_blank"
          rel="noopener noreferrer"
          class="c-list-row c-list-row--clickable c-list-row--tall"
        >
          <div class="c-list-row__info">
            <div class="c-list-row__title">Reddit Topluluğu</div>
            <div class="c-list-row__desc">r/kepce üzerinden topluluk tartışmaları</div>
          </div>
          <div class="c-list-row__control">
            {@html icon("externalLink", 16)}
          </div>
        </a>
      </div>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">SearXNG Altyapısı</h2>
        <p class="u-leading-comfortable">
          Arama motoru altyapısıyla ilgili doğrudan katkı veya hata bildirimleri için <a href="https://github.com/searxng/searxng" target="_blank" rel="noopener noreferrer" class="text-link">SearXNG GitHub deposunu</a> ziyaret edebilirsiniz.
        </p>
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
