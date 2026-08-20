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
    KYK yurtlarında barınan öğrencilere her gün sabah kahvaltısı ve akşam yemeği için devlet tarafından nakdi beslenme yardımı tanımlanır. Bu tutar yemekhanelerde doğrudan harcanabilir.
  </p>

  <h2>Beslenme Yardımı Nasıl Kullanılır?</h2>
  <p>
    Yemekhanede tabldot veya alakart yemek seçtikten sonra kasada öğrenci kartınızı okutarak veya parmak izi / yüz tanıma ile yardım tutarını anında düşebilirsiniz.
  </p>

  <h2>Bilinmesi Gerekenler</h2>
  <ul>
    <li><strong>Öğün Ayrımı:</strong> Beslenme yardımı sabah ve akşam için ayrıdır. Sabah hakkı sadece sabah kahvaltısında, akşam hakkı akşam yemeğinde geçerlidir.</li>
    <li><strong>Limit Aşımı:</strong> Seçtiğiniz yemeklerin toplam tutarı yardımı aşarsa aradaki fark kartla kasada ödenir.</li>
    <li><strong>Devir Yoktur:</strong> Kullanılmayan yardım sonraki güne devretmez, nakit paraya çevrilemez.</li>
    <li><strong>İzin Durumu:</strong> İzinli veya yurtta olmadığınız günlerin yemek yardımı birikmez.</li>
  </ul>

  <h2>Günün Menüsü</h2>
  <p>
    Yurdunuzda bugün çıkan menüyü ve besin değerlerini kontrol etmek için <a href="/" data-link>Günün Menüsü</a> veya geçmiş yemekler için <a href="/arsiv" data-link>Arşiv</a> sayfamıza bakabilirsiniz.
  </p>
</ContentPage>
