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
  <title>Gizlilik Politikası ve KVKK Aydınlatma Metni | Kepçe Ara</title>
  <meta
    name="description"
    content="6698 sayılı KVKK ve 5651 sayılı Kanun kapsamında Kepçe Ara veri işleme ilkeleri ve aydınlatma metni."
  />
</svelte:head>

<div class="settings-page c-search-adv-page">
  <article class="content-page">
    <header class="content-page__header">
      <h1 class="content-page__title">
        Gizlilik Politikası ve KVKK Aydınlatma Metni
      </h1>
      <div class="content-page__meta">
        <time class="content-page__date">Yürürlük Tarihi: 2 Eylül 2026</time>
      </div>
    </header>

    <div class="content-page__body">
      <p class="u-text-base u-leading-comfortable u-mb-lg">
        Bu metin, 6698 sayılı Kişisel Verilerin Korunması Kanunu ("KVKK") m. 10
        ile 5651 sayılı Kanun uyarınca, <strong>Kepçe Ara</strong> meta arama motorunu
        kullanan ziyaretçilerin bilgilendirilmesi amacıyla hazırlanmıştır.
      </p>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          1. Veri Sorumlusu ve Hizmet Niteliği
        </h2>
        <p class="u-leading-comfortable">
          Kepçe Ara, açık kaynaklı SearXNG altyapısını kullanan bir meta arama
          aracıdır. Arama terimlerini anonimleştirerek üçüncü taraf motorlara
          iletir ve sonuçları derler.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          2. İşlenen Veriler ve Sıfır-Log İlkesi
        </h2>
        <p class="u-leading-comfortable">
          Kullanıcı gizliliği şu esaslara dayanır:
        </p>
        <ul class="u-mt-xs u-pl-md u-leading-comfortable">
          <li>
            <strong>Arama Terimleri:</strong> Arama kutusuna yazdığınız sorgular
            sunucu veri tabanına kaydedilmez. Sorgu anlık olarak motorlara iletilir
            ve sonuçlar tarayıcınıza gönderildikten sonra bellekten silinir.
          </li>
          <li>
            <strong>IP Adresi ve Trafik Verileri:</strong> Arama oturumları ile IP
            adresleri arasında profil oluşturulmaz. 5651 sayılı Kanun m. 4 ve 5
            kapsamında bağlantı sağlanan harici içeriklere ilişkin arama kayıtları
            tutulmaz.
          </li>
          <li>
            <strong>Kullanıcı Profili ve Reklam:</strong> Kullanıcı profili oluşturulmaz;
            reklam çerezi veya takip pikseli kullanılmaz.
          </li>
        </ul>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          3. Çerezler ve Yerel Depolama (localStorage)
        </h2>
        <p class="u-leading-comfortable">
          Kepçe Ara üçüncü taraf takip çerezi kullanmaz. Motor, tema, dil,
          güvenli arama ve ara yüz seçenekleriniz yalnızca tarayıcınızın yerel
          depolama alanında (<code>localStorage</code>) tutulur. Bu veriler
          sunucuya gönderilmez. Bu tercihleri, tarayıcı verilerinizi temizleyerek
          veya ayarlar sayfasından dilediğiniz zaman silebilirsiniz.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          4. POST Yöntemi ve Yönlendirme (Referrer) Koruması
        </h2>
        <p class="u-leading-comfortable">
          Aramalar varsayılan olarak HTTP POST yöntemiyle yapılır. Arama
          teriminiz tarayıcı geçmişine yazılmaz ve sonuçlardaki bağlantılara
          tıklandığında hedef siteye HTTP Referrer başlığı üzerinden aktarılmaz.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">
          5. İlgili Kişinin Hakları (KVKK Madde 11)
        </h2>
        <p class="u-leading-comfortable">
          KVKK'nin 11. maddesi uyarınca veri sahipleri kişisel verilerinin
          işlenip işlenmediğini öğrenme hakkına sahiptir. Kepçe Ara kişisel veri
          kaydı tutmadığından sistemlerimizde saklanan bir kullanıcı profili
          yoktur.
        </p>
      </section>

      <section class="u-mb-lg">
        <h2 class="u-text-lg u-font-bold u-mb-xs">6. İletişim</h2>
        <p class="u-leading-comfortable">
          Gizlilik uygulamalarımızla ilgili sorularınız için <a
            href="mailto:iletisim@kepce.org"
            class="text-link">iletisim@kepce.org</a
          > adresinden bize ulaşabilirsiniz.
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
