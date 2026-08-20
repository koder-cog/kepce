<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";

  const guideSchema = {
    "@context": "https://schema.org",
    "@type": "Article",
    headline: "KYK Beslenme Yardımı",
    description: "KYK yurtlarında günlük yemek yardımı, sabah ve akşam öğün limitleri ve kullanım kuralları.",
    datePublished: "2026-08-20",
    dateModified: "2026-08-20",
    inLanguage: "tr-TR",
    author: {
      "@type": "Organization",
      name: "Kepçe",
      url: "https://kepce.org/",
    },
    publisher: {
      "@type": "Organization",
      name: "Kepçe",
      logo: "https://kepce.org/icon-512.png",
    },
  };
</script>

<Seo
  title="KYK Beslenme Yardımı | Kepçe"
  description="KYK yurtlarında günlük yemek yardımı ne kadar? Sabah ve akşam beslenme yardımı kullanım kuralları ve limit aşımı detayları."
  image="https://kepce.org/api/v1/public/og/page/rehber"
  canonical="https://kepce.org/kyk-beslenme-yardimi"
  schema={guideSchema}
/>

<ContentPage
  title="KYK Beslenme Yardımı"
  dateLabel="Yayımlanma tarihi: 2026.08.20"
>
  <p>
    KYK yurtlarında barınan her öğrenciye devlet tarafından sabah kahvaltısı ve akşam yemeği için günlük nakdi beslenme yardımı tanımlanır. Bu tutar yemekhanelerde geçerlidir ve tabldot ya da alakart yemeklerde kullanılabilir.
  </p>

  <h2>Beslenme Yardımı Nasıl Kullanılır?</h2>
  <p>
    Yemekhanede tabldot veya alakart yemeklerinizi seçtikten sonra kasada öğrenci kimlik kartınızı okutarak veya parmak izi / yüz tanıma sistemiyle yardım tutarınızı anında düşebilirsiniz.
  </p>

  <h2>Sabah ve Akşam Kotası</h2>
  <p>
    Beslenme yardımı sabah kahvaltısı ve akşam yemeği için iki ayrı kota şeklinde verilir. Sabah için tanımlanan tutarı yalnızca sabah kahvaltısı saatlerinde, akşam tutarını ise akşam yemeğinde harcayabilirsiniz.
  </p>
  <p>
    Sabah yemediğiniz yemek akşamki bakiyenize eklenmez. Kullanılmayan hak gün bittiğinde sıfırlanır.
  </p>

  <h2>Limit Aşımı Durumunda Ne Olur?</h2>
  <p>
    Eğer seçtiğiniz yemeklerin toplam fiyatı o öğün için tanımlanan beslenme yardımını aşarsa, aradaki farkı banka kartı, kredi kartı veya nakit ile kasada ödeyebilirsiniz.
  </p>

  <h2>Yurtta Olmadığınız Günler</h2>
  <p>
    Hafta sonu izinli olduğunuzda veya tatillerde yemekhaneyi kullanmadığınız günlerin yemek yardımı birikmez veya nakit olarak talep edilemez.
  </p>
  <p>
    Bugünkü yemek listesini ve besin değerlerini incelemek için <a href="/" data-link>Günün Menüsü</a> sayfasına bakabilirsiniz.
  </p>
</ContentPage>
