<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";
  import PricingCalculator from "@/components/features/pricing/PricingCalculator.svelte";
  import GuideSummary from "@/components/features/guide/GuideSummary.svelte";

  const guideSchema = {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "BreadcrumbList",
        itemListElement: [
          {
            "@type": "ListItem",
            position: 1,
            name: "Ana Sayfa",
            item: "https://kepce.org/",
          },
          {
            "@type": "ListItem",
            position: 2,
            name: "KYK Beslenme Yardımı",
            item: "https://kepce.org/kyk-beslenme-yardimi",
          },
        ],
      },
      {
        "@type": "WebPage",
        "@id": "https://kepce.org/kyk-beslenme-yardimi#webpage",
        url: "https://kepce.org/kyk-beslenme-yardimi",
        name: "KYK Beslenme Yardımı | Kepçe",
        description:
          "KYK beslenme yardımı tutarları, standart tabldot menü kapsamı, kahvaltı ve akşam yemeği kotaları ile tavan fiyat kuralları.",
        inLanguage: "tr-TR",
      },
      {
        "@type": "Organization",
        "@id": "https://kepce.org/#organization",
        name: "Kepçe",
        url: "https://kepce.org/",
        logo: "https://kepce.org/icon-512.png",
      },
    ],
  };

  const summaryRows = [
    { label: "Kahvaltı", value: "110 ₺" },
    { label: "Akşam yemeği", value: "190 ₺" },
    { label: "Günlük toplam", value: "300 ₺", wide: true },
  ];

  let { data } = $props();
</script>

<Seo
  title={data?.seoTitle || "KYK Beslenme Yardımı | Kepçe"}
  description={data?.seoDescription ||
    "KYK beslenme yardımı tutarları, standart tabldot menü kapsamı, kahvaltı ve akşam yemeği kotaları ile tavan fiyat kuralları."}
  image={data?.ogImage || "https://kepce.org/api/v1/public/og/page/rehber"}
  canonical="https://kepce.org/kyk-beslenme-yardimi"
  schema={guideSchema}
/>

<ContentPage
  title="KYK Beslenme Yardımı"
  dateLabel="Son güncelleme: 28 Eylül 2026"
>
  <GuideSummary
    title="2026-2027 Tutarları"
    rows={summaryRows}
    note="Belirtilen tutarlar nakit harçlık değildir. Yemekhanede standart tabldot menüyü karşılamak üzere tanımlanan günlük bütçedir, kantinde harcanamaz."
  />

  <p>
    KYK yurtlarında barınan öğrencilere kahvaltı ve akşam yemeği için günlük
    beslenme yardımı tanımlanır. Bu bütçe banka hesabına nakit olarak yatmaz.
    Yemekhane kasalarında doğrudan o günkü tabldot menü ücretini karşılar.
  </p>

  <h2>Menü Düzeni</h2>
  <p>
    KYK yemekhanelerinde yemekler tabldot olarak sunulur. Kahvaltıda çay, ekmek
    ve suyun yanına peynir, zeytin, yumurta ve gününe göre sıcak bir seçenek
    eklenerek kahvaltı tabağı verilir.
  </p>
  <p>
    Akşam yemeğinde ise ekmek ve suyun haricinde mutfağın belirlediği dört beş
    kaplık klasik tabldot sunulur: çorba, ana yemek, yan yemek (pilav, makarna
    vb.) ve tamamlayıcı bir ürün (tatlı, meyve veya yoğurt).
  </p>
  <p>
    Tanımlanan günlük beslenme yardımı, her iki öğünde de bu standart menüleri
    tam olarak karşılayacak şekilde ayarlanır. Dolayısıyla standart tabldotu
    aldığınızda cebinizden fazladan bir ücret çıkmaz.
  </p>

  <h2>Kota Nasıl İşler?</h2>
  <p>
    Beslenme yardımı kahvaltı ve akşam yemeği için iki ayrı kota hâlinde
    tanımlanır. Kahvaltı kotasını yalnızca kahvaltı saatlerinde, akşam kotasını
    ise yalnızca akşam yemeğinde kullanabilirsiniz. Hafta içi, hafta sonu ve
    Ramazan servis saatlerinin ayrıntıları için
    <a href="/kyk-yemek-saatleri" data-link>KYK Yemek Saatleri</a>
    rehberine bakabilirsiniz.
  </p>
  <p>
    Sabah yemediğiniz yemek akşamki bakiyenize eklenmez. Kullanılmayan yardım
    hakkı gün sonunda sıfırlanır, sonraki güne devretmez veya nakit olarak
    alınamaz.
  </p>

  <h2>Ek Ürünler ve Tavan Fiyat</h2>
  <p>
    Yemekhanelerde standart menü haricinde sunulan ilave ürün veya kantin
    seçimlerinde yurtlar için belirlenen tavan fiyat tarifesi geçerlidir. Yardım
    limitini aşan ilave taleplerde aradaki fark kasada ödenir.
  </p>

  <h2>Hafta Sonu ve İzinli Günler</h2>
  <p>
    Hafta sonları beslenme yardımı açısından özel bir ayrıcalık veya kısıtlama
    yoktur, haklar hafta içiyle tamamen aynı işler. İster hafta sonu iznine
    çıkın ister tatilde memlekete gidin, yurtta bulunmayıp yemekhaneyi
    kullanmadığınız günlerin beslenme yardımı devretmez, gün sonunda silinir ve
    geriye dönük talep edilemez.
  </p>
  <p>
    Günün menüsünü görmek için <a href="/" data-link>ana sayfaya</a> göz
    atabilirsiniz. "Kendi tepsimi dizsem bu bütçeye denkleşir mi?" diyorsanız
    aşağıdaki simülatörle bütçeyi tutturmayı deneyebilirsiniz.
  </p>

  <PricingCalculator />
</ContentPage>
