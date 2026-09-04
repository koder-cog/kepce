<script>
  let { basePath = "" } = $props();
</script>

<footer class="site-footer">
  <div class="site-footer__inner">
    <div class="site-footer__brand">
      <div class="site-footer__brand-title">Kepçe Ara</div>
      <p class="site-footer__brand-desc">
        Arama geçmişinizi kaydetmeyen, profil çıkarmayan ve reklam satmayan açık
        kaynaklı meta arama motoru. SearXNG altyapısıyla çalışır.
      </p>
    </div>

    <div class="site-footer__col">
      <div class="site-footer__col-title">Proje</div>
      <a
        href="https://github.com/koder-cog/kepce"
        target="_blank"
        rel="noopener noreferrer"
        class="site-footer__link">Kepçe Kaynak Kodu</a
      >
      <a
        href="https://github.com/searxng/searxng"
        target="_blank"
        rel="noopener noreferrer"
        class="site-footer__link">SearXNG Kaynak Kodu</a
      >
      <a
        href="https://github.com/koder-cog/kepce/issues"
        target="_blank"
        rel="noopener noreferrer"
        class="site-footer__link">Hata / Öneri Bildir</a
      >
    </div>

    <div class="site-footer__col">
      <div class="site-footer__col-title">Yasal</div>
      <a href={`${basePath}/gizlilik`} class="site-footer__link"
        >Gizlilik Politikası</a
      >
      <a href={`${basePath}/kosullar`} class="site-footer__link"
        >Kullanım Koşulları</a
      >
      <a href={`${basePath}/iletisim`} class="site-footer__link">İletişim</a>
    </div>

    <div class="site-footer__col">
      <div class="site-footer__col-title">Kepçe</div>
      <a href="/" class="site-footer__link">Yemekhane Menüleri</a>
      <a href={`${basePath}/ayarlar`} class="site-footer__link"
        >Arama Ayarları</a
      >
      <a
        href="https://reddit.com/r/kepce"
        target="_blank"
        rel="noopener noreferrer"
        class="site-footer__link">Subreddit</a
      >
    </div>
  </div>

  <div class="site-footer__bottom">
    <div class="site-footer__agpl">
      <div
        class="site-footer__agpl-badge"
        role="img"
        aria-label="AGPLv3"
      ></div>
    </div>
  </div>
</footer>
