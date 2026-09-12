<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";
  import PricingCalculator from "@/components/features/pricing/PricingCalculator.svelte";

  const guideSchema = {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "BreadcrumbList",
        itemListElement: [
          { "@type": "ListItem", position: 1, name: "Ana Sayfa", item: "https://kepce.org/" },
          { "@type": "ListItem", position: 2, name: "KYK Beslenme Yardımı", item: "https://kepce.org/kyk-beslenme-yardimi" },
        ],
      },
      {
        "@type": "Article",
        headline: "KYK Beslenme Yardımı",
        description:
          "KYK yurtlarında beslenme yardımı, fiks menü işleyişi, günlük kota ve tavan fiyat kuralları.",
        datePublished: "2026-08-20",
        dateModified: "2026-08-20",
        inLanguage: "tr-TR",
        mainEntityOfPage: "https://kepce.org/kyk-beslenme-yardimi",
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
      },
    ],
  };
</script>

<Seo
  title="2026-2027 KYK Beslenme Yardımı Ne Kadar? (Günlük Ücret & Limit) | Kepçe"
  description="2026-2027 KYK beslenme yardımı kaç TL oldu? Kahvaltı ve akşam yemeği günlük limitleri, fiks menü işleyişi ve hesaplama rehberi."
  image="https://kepce.org/api/v1/public/og/page/rehber"
  canonical="https://kepce.org/kyk-beslenme-yardimi"
  schema={guideSchema}
/>

<ContentPage
  title="KYK Beslenme Yardımı"
  dateLabel="Yayımlanma tarihi: 2026.08.20"
>
  <p>
    KYK yurtlarında kalan her öğrenciye kahvaltı ve akşam yemeği için günlük
    nakdi beslenme yardımı tanımlanır. Bu tutar öğrencinin hesabına para olarak
    yatmaz. Doğrudan yemekhane kasalarında indirim fişi olarak kullanılır.
  </p>

  <h2>Menü Düzeni ve Fiks Menü Mantığı</h2>
  <p>
    KYK yurt yemekhanelerinde yemek servisi fiks menü (standart tabldot) olarak verilir.
    Her öğünde mutfak tarafından belirlenen 4 kap yemek sunulur: çorba, ana yemek,
    yan yemek ve tamamlayıcı bir ürün (tatlı, meyve veya yoğurt).
  </p>
  <p>
    Devlet tarafından tanımlanan günlük beslenme yardımı tutarı, bu standart tabldot menünün
    ücretini tam olarak karşılayacak şekilde ayarlanır. Dolayısıyla menüyü eksiksiz
    aldığınızda cebinizden herhangi bir ek ücret çıkmaz.
  </p>

  <h2>Kota Nasıl İşler?</h2>
  <p>
    Beslenme yardımı kahvaltı ve akşam yemeği için iki ayrı kota halinde
    tanımlanır. Kahvaltı kotasını yalnızca kahvaltı saatlerinde, akşam kotasını
    ise akşam yemeğinde harcayabilirsiniz.
  </p>
  <p>
    Sabah yemediğiniz yemek akşamki bakiyenize eklenmez. Kullanmadığınız yardım
    hakkı o gün bittiğinde sıfırlanır, sonraki güne devretmez veya nakit olarak geri alınamaz.
  </p>

  <h2>Ekstra Seçenekler ve Limitler</h2>
  <p>
    Yemekhanelerde standart menü haricinde sunulan ilave ürün veya kantin seçimlerinde
    Bakanlığın belirlediği resmi tavan fiyatlar geçerlidir. Yardım limitini aşan
    ekstra taleplerde aradaki fark kasada banka kartı, kredi kartı veya nakit ile ödenir.
  </p>

  <h2>Yurttan İzinli Olduğunuz Günler</h2>
  <p>
    Hafta sonu izne çıktığınızda veya tatil dönemlerinde yemekhaneyi
    kullanmadığınız günlerin beslenme yardımı geçerliliğini yitirir, geriye dönük hak talep
    edilemez.
  </p>
  <p>
    Bugün şehrinizde hangi tabldot menünün çıktığını ve besin değerlerini görmek
    için <a href="/" data-link>ana sayfaya</a> bakabilir, resmi tavan fiyat listesini ve
    yardım hesaplamasını aşağıdaki tablodan inceleyebilirsiniz.
  </p>

  <PricingCalculator />
</ContentPage>
