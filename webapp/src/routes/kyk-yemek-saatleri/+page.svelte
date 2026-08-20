<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";

  const guideSchema = {
    "@context": "https://schema.org",
    "@type": "Article",
    headline: "KYK Yemek Saatleri",
    description: "KYK yurtlarında sabah kahvaltısı ve akşam yemeği saatleri, hafta sonu düzeni ve yemekhane kuralları.",
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
  title="KYK Yemek Saatleri | Kepçe"
  description="KYK yurtlarında sabah kahvaltısı ve akşam yemeği saatleri kaçta başlıyor, kaçta bitiyor? Hafta içi ve hafta sonu yemekhane saatleri."
  image="https://kepce.org/api/v1/public/og/page/rehber"
  canonical="https://kepce.org/kyk-yemek-saatleri"
  schema={guideSchema}
/>

<ContentPage
  title="KYK Yemek Saatleri"
  dateLabel="Yayımlanma tarihi: 2026.08.20"
>
  <p>
    Yurtta hayatın büyük bir kısmı yemekhanenin açılış ve kapanış saatlerine göre şekillenir. Sabah dersine yetişirken ya da akşam yurda dönerken kapıda kalmamanız için genel saat düzeni şöyledir:
  </p>

  <h2>Sabah Kahvaltısı</h2>
  <p>
    Hafta içi günlerde kahvaltı servisi sabah <strong>06:00</strong>'da başlar ve öğlen <strong>12:00</strong>'ye kadar devam eder. Hafta sonları ve resmi tatillerde ise öğrenciler biraz daha uyuyabilsin diye yarım saat ötelenerek <strong>06:30 – 12:30</strong> saatleri arasında verilir.
  </p>
  <p>
    Saat 12:00'yi (hafta sonu 12:30'u) bir dakika bile geçseniz sistem kapanır ve o günkü kahvaltı hakkınız yanar. Bu yüzden son dakikaya bırakmamanız tavsiye edilir.
  </p>

  <h2>Akşam Yemeği</h2>
  <p>
    Akşam yemeği hem hafta içi hem de hafta sonu <strong>16:00</strong>'da başlar ve gece <strong>22:30</strong>'a kadar sürer. Bazı büyük nöbetçi yurtlarda veya sınav dönemlerinde bu süre 23:00'e kadar uzatılabilse de genel standart 22:30'dur.
  </p>

  <h2>Ramazan Ayı Düzeni</h2>
  <p>
    Ramazan boyunca akşam yemeği saatleri iftar vaktine göre ayarlanır. Ayrıca sahura kalkacak öğrenciler için gece saatlerinde sahur servisi açılır ve sabah kahvaltısı hakkı sahur öğününde kullanılır.
  </p>

  <h2>Bilmenizde Fayda Olan Ufak Detaylar</h2>
  <p>
    Sabah kahvaltısı hakkınızı akşam yemeğine aktaramazsınız. Her öğünün yardımı yalnızca kendi saat aralığında geçerlidir ve kullanılmayan öğün o gün için sıfırlanır.
  </p>
  <p>
    Yemekhaneye girişlerde parmak izi, yüz tanıma veya öğrenci kimlik kartı kullanılır. Turnikeden veya kasadan geçerken sistem doğrudan o öğünün hakkını düşer.
  </p>
  <p>
    Bugün yurdunuzda hangi yemeklerin çıktığını merak ediyorsanız doğrudan <a href="/" data-link>Günün Menüsü</a> sayfasına bakabilirsiniz.
  </p>
</ContentPage>
