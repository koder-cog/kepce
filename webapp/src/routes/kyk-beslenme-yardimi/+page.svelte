<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";
  import PricingCalculator from "@/components/features/pricing/PricingCalculator.svelte";

  const guideSchema = {
    "@context": "https://schema.org",
    "@type": "Article",
    headline: "KYK Beslenme Yardımı",
    description:
      "KYK yurtlarında beslenme yardımı, tabldot ve alakart farkı, kota ve limit aşımı kuralları.",
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
  title="2026-2027 KYK Beslenme Yardımı Ne Kadar? (Günlük Ücret & Limit) | Kepçe"
  description="2026-2027 KYK beslenme yardımı kaç TL oldu? Kahvaltı ve akşam yemeği günlük limitleri, tabldot farkı ve hesaplama rehberi."
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
    yatmaz; doğrudan yemekhane kasalarında indirim fişi olarak kullanılır.
  </p>

  <h2>Tabldot ve Alakart Nedir?</h2>
  <p>
    Yemekhaneye ilk kez girenlerin en çok karıştırdığı iki kavram şunlardır:
  </p>
  <p>
    Tabldot: O gün mutfağın belirlediği standart fiks menüdür. Genellikle çorba,
    ana yemek, yan yemek ve tamamlayıcı bir ürün (tatlı, meyve, yoğurt vb.)
    olmak üzere 4 kaptan oluşur. Tabldot menünün toplam fiyatı, o öğün için
    tanımlanan devlet beslenme yardımına tam denk gelecek şekilde ayarlanır.
    Yani standart tabldot menü aldığınızda cebinizden ekstra para çıkmaz.
  </p>
  <p>
    Alakart: Büfede sergilenen yemeklerden kendi isteğinize göre tek tek seçim
    yapmanızdır. Örneğin tabldot ana yemeği yerine tost, sandviç, pide, lahmacun
    veya ekstra tatlı/içecek seçebilirsiniz. Bu ürünlerin her birinin gramajına
    göre resmi birim fiyatı vardır.
  </p>

  <h2>Kota Nasıl İşler?</h2>
  <p>
    Beslenme yardımı kahvaltı ve akşam yemeği için iki ayrı kota halinde
    tanımlanır. Kahvaltı kotasını yalnızca kahvaltı saatlerinde, akşam kotasını
    ise akşam yemeğinde harcayabilirsiniz.
  </p>
  <p>
    Sabah yemediğiniz yemek akşamki bakiyenize eklenmez. Kullanmadığınız yardım
    hakkı o gün bittiğinde sıfırlanır, birikmez veya nakit olarak geri alınamaz.
  </p>

  <h2>Limit Aşımı Durumunda Ne Olur?</h2>
  <p>
    Alakart ürünler seçtiğinizde ve sepet tutarınız o öğünün beslenme yardım
    limitini aştığında, aradaki farkı kasada banka kartı, kredi kartı veya nakit
    ile ödeyerek alışverişinizi tamamlayabilirsiniz.
  </p>

  <h2>Yurttan İzinli Olduğunuz Günler</h2>
  <p>
    Hafta sonu izne çıktığınızda veya tatil dönemlerinde yemekhaneyi
    kullanmadığınız günlerin beslenme yardımı kaybolur, geriye dönük hak talep
    edilemez.
  </p>
  <p>
    Bugün şehrinizde hangi tabldot menünün çıktığını ve besin değerlerini görmek
    için <a href="/" data-link>ana sayfaya</a> bakabilir, alakart büfeden yapacağınız
    seçimlerin resmi tavan fiyatlarını ve fiş farkını ise aşağıdaki hesaplayıcıdan
    inceleyebilirsiniz.
  </p>

  <PricingCalculator />
</ContentPage>
