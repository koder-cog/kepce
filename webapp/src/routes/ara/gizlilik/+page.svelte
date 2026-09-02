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
  <title>Gizlilik Politikası ve Aydınlatma Metni | Kepçe Ara</title>
  <meta
    name="description"
    content="6698 sayılı KVKK ve 5651 sayılı Kanun kapsamında Kepçe Ara veri işleme ilkeleri ve aydınlatma metni."
  />
</svelte:head>

<div class="settings-page c-search-adv-page">
  <article class="content-page">
    <header class="content-page__header">
      <h1 class="content-page__title">
        Gizlilik Politikası ve Aydınlatma Metni
      </h1>
      <div class="content-page__meta">
        <time class="content-page__date">Yürürlük Tarihi: 2 Eylül 2026</time>
      </div>
    </header>

    <div class="content-page__body">
      <p class="u-text-base u-leading-comfortable u-mb-lg">
        Bu metin, 6698 sayılı Kişisel Verilerin Korunması Kanunu ile 5651 sayılı Kanun kapsamında Kepçe Ara hizmetini kullanan ziyaretçileri bilgilendirmek amacıyla hazırlanmıştır.
      </p>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          1. Hizmetin Niteliği
        </h2>
        <p class="u-leading-comfortable">
          Kepçe Ara, açık kaynaklı SearXNG altyapısı üzerinde çalışan bağımsız bir meta arama aracıdır. Aramalarınızı doğrudan arama motorlarına iletmek yerine vekil sunucu üzerinden arındırarak iletir ve gelen sonuçları tarafsız şekilde listeler.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          2. Arama Gizliliği ve Sorgu Verileri
        </h2>
        <p class="u-leading-comfortable">
          Arama kutusuna yazdığınız terimler sunucu veritabanına kaydedilmez. Yapılan arama anlık olarak ilgili motorlara iletilir, sonuçlar tarayıcınıza ulaştıktan sonra bellekten silinir. Arama geçmişi tutulmaz, kullanıcı profili çıkarılmaz ve davranışsal reklam hedeflemesi yapılmaz.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          3. Kanuni Veri Saklama Yükümlülüğü
        </h2>
        <p class="u-leading-comfortable">
          5651 sayılı Kanun uyarınca yer sağlayıcılar, sunucu düzeyindeki temel erişim ve trafik kayıtlarını (bağlantı tarihi, saati, IP adresi ve port bilgisi) kanunen 1 yıldan az ve 2 yıldan fazla olmamak üzere saklamakla yükümlüdür. Bu yasal kayıtlar arama sorgularınızla veya kişisel kimliğinizle eşleştirilmez; yasal sürenin sonunda sistemden kalıcı olarak silinir.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          4. Çerezler ve Tarayıcı Hafızası
        </h2>
        <p class="u-leading-comfortable">
          Kepçe Ara üçüncü taraf takip çerezi kullanmaz. Tema, dil, güvenli arama ve arayüz tercihleriniz yalnızca kendi tarayıcınızın yerel hafızasında tutulur. Bu ayarlar sunucuya iletilmez; tarayıcı geçmişinizi temizleyerek veya ayarlar sayfasından dilediğiniz an silebilirsiniz.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          5. Yönlendirme Koruması
        </h2>
        <p class="u-leading-comfortable">
          Aramalar POST yöntemiyle gerçekleştirilir. Arama sonuçlarında tıkladığınız web sitelerine arama teriminiz yönlendirme başlığı üzerinden aktarılmaz.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          6. Başvuru ve İletişim
        </h2>
        <p class="u-leading-comfortable">
          Kişisel Verilerin Korunması Kanunu kapsamındaki haklarınız ve gizlilik uygulamalarımızla ilgili sorularınız için <a href="mailto:iletisim@kepce.org" class="text-link">iletisim@kepce.org</a> adresinden bize ulaşabilirsiniz.
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
