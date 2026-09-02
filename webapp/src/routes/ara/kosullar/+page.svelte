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
  <title>Kullanım Koşulları ve Yasal Bildirim | Kepçe Ara</title>
  <meta
    name="description"
    content="5651 sayılı Kanun kapsamında Kepçe Ara kullanım koşulları, sorumluluk reddi ve hukuka aykırı içerik bildirimi."
  />
</svelte:head>

<div class="settings-page c-search-adv-page">
  <article class="content-page">
    <header class="content-page__header">
      <h1 class="content-page__title">Kullanım Koşulları ve Yasal Bildirim</h1>
      <div class="content-page__meta">
        <time class="content-page__date">Yürürlük Tarihi: 2 Eylül 2026</time>
      </div>
    </header>

    <div class="content-page__body">
      <p class="u-text-base u-leading-comfortable u-mb-lg">
        Kepçe Ara meta arama motorunu kullanarak aşağıdaki kullanım koşullarını
        kabul etmiş sayılırsınız. Bu metin, 5651 sayılı Kanun ve ilgili mevzuat
        uyarınca hazırlanmıştır.
      </p>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          1. Hizmetin Niteliği ve Kapsamı
        </h2>
        <p class="u-leading-comfortable">
          Kepçe Ara, kullanıcılara internette arama yapma ve farklı motorlardan
          gelen sonuçları tek ekranda görüntüleme imkânı veren açık kaynaklı bir
          meta arama aracıdır. Kepçe Ara içerik üreticisi değildir; arama
          sorgularını vekil (proxy) sunucu üzerinden ilgili arama motorlarına iletir.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          2. Dış Bağlantılar ve Sorumluluk Sınırı (5651 Sayılı Kanun m. 4)
        </h2>
        <p class="u-leading-comfortable">
          5651 sayılı Kanun'un 4. maddesinin 2. fıkrası uyarınca: <em
            >"İçerik sağlayıcı, bağlantı sağladığı başkasına ait içerikten
            sorumlu değildir."</em
          >
        </p>
        <p class="u-mt-xs u-leading-comfortable">
          Arama sonuçlarında listelenen siteler, başlıklar, açıklamalar ve
          görseller üçüncü taraflara aittir. Kepçe Ara bu harici web
          sayfalarının içeriğini denetleme yükümlülüğüne sahip değildir.
          Ziyaret ettiğiniz dış sitelerin kullanım koşulları ve gizlilik
          ilkeleri doğrudan ilgili sitelerin sorumluluğundadır.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          3. Hukuka Aykırı İçerik Bildirimi (Uyar-Kaldır)
        </h2>
        <p class="u-leading-comfortable">
          5651 sayılı Kanun m. 5/2 uyarınca, yer sağlayıcı hukuka aykırı
          içerikten haberdar edilmesi hâlinde yayından çıkarmakla yükümlüdür.
          Kepçe Ara doğrudan içerik barındırmasa da, arama sonuçlarında
          kişilik haklarını ihlal eden bir bağlantı tespit etmeniz hâlinde
          ilgili URL ve yasal gerekçenizi belirten bildirimlerinizi <a
            href="mailto:iletisim@kepce.org"
            class="text-link">iletisim@kepce.org</a
          > adresine iletebilirsiniz. Bildirilen URL incelenerek arama
          sonuçlarından çıkarılır.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          4. Adil Kullanım (Fair Use)
        </h2>
        <p class="u-leading-comfortable">
          Kepçe Ara bireysel kullanım için ücretsiz sunulmaktadır. Hizmeti
          aksatacak düzeyde otomatik bot istekleri veya kazıma (scraping)
          girişimleri hız sınırlandırması (rate limit) ile engellenebilir.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">5. Açık Kaynak Lisansı</h2>
        <p class="u-leading-comfortable">
          Kepçe Ara yazılımı AGPLv3 (GNU Affero General Public License) ile
          lisanslanmıştır. Kullanıcılar kaynak kodunu inceleme, çalıştırma ve
          geliştirme hakkına sahiptir.
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
