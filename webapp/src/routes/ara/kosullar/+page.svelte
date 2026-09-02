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
    content="5651 sayılı Kanun kapsamında Kepçe Ara kullanım koşulları, sorumluluk sınırları ve hukuka aykırı içerik bildirimi."
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
        Kepçe Ara hizmetini kullanarak aşağıdaki kullanım şartlarını kabul etmiş sayılırsınız. Bu metin, 5651 sayılı Kanun ve ilgili mevzuat çerçevesinde hazırlanmıştır.
      </p>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          1. Hizmetin Kapsamı
        </h2>
        <p class="u-leading-comfortable">
          Kepçe Ara, açık kaynaklı bir meta arama aracıdır. Kullanıcıların arama sorgularını vekil sunucular üzerinden ilgili arama motorlarına iletir ve sonuçları tek bir ekranda derleyip sunar. Kepçe Ara listelenen web sayfalarının içerik üreticisi veya barındırıcısı değildir.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          2. Dış Bağlantılar ve Sorumluluk Sınırı
        </h2>
        <p class="u-leading-comfortable">
          5651 sayılı Kanun uyarınca içerik sağlayıcılar, bağlantı sağladıkları başkasına ait harici içerikten sorumlu değildir.
        </p>
        <p class="u-mt-xs u-leading-comfortable">
          Arama sonuçlarında gösterilen bağlantılar, web sayfaları, başlıklar ve görseller üçüncü taraf sitelere aittir. Kepçe Ara bu harici sayfaları önceden denetleme yükümlülüğüne sahip değildir. Ziyaret edilen dış sitelerin kullanım koşulları ve gizlilik politikaları doğrudan ilgili sitenin sorumluluğundadır.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          3. Hukuka Aykırı İçerik Bildirimi
        </h2>
        <p class="u-leading-comfortable">
          5651 sayılı Kanun gereğince, hukuka aykırı veya kişilik haklarını ihlal eden bir içerikten haberdar edilmesi hâlinde gerekli inceleme yapılır. Arama sonuçlarında hukuka aykırı olduğunu düşündüğünüz bir bağlantı bulunması durumunda, ilgili internet adresi ve yasal gerekçenizi belirten bildiriminizi <a href="mailto:iletisim@kepce.org" class="text-link">iletisim@kepce.org</a> adresine iletebilirsiniz. Yapılan bildirimler incelenerek uygun görülen bağlantılar arama dizininden çıkarılır.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          4. Adil Kullanım
        </h2>
        <p class="u-leading-comfortable">
          Kepçe Ara bireysel kullanım için ücretsiz olarak sunulur. Hizmetin sürekliliğini tehlikeye atacak düzeyde yoğun otomatik bot istekleri veya sisteme zarar vermeye yönelik girişimler istek sınırlamasıyla engellenebilir.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">5. Açık Kaynak Lisansı</h2>
        <p class="u-leading-comfortable">
          Kepçe Ara yazılımı AGPLv3 lisansı ile dağıtılmaktadır. Kaynak kodunu inceleme, çalıştırma ve geliştirme hakları bu lisans koşulları kapsamında korunur.
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
