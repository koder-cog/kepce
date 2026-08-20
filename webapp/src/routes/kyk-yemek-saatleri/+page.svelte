<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";

  const guideSchema = {
    "@context": "https://schema.org",
    "@type": "Article",
    headline: "KYK Yemek Saatleri",
    description: "KYK yurtlarında kahvaltı ve akşam yemeği saatleri, yemekhane gerçekleri ve Ramazan düzeni.",
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
  description="KYK yurtlarında kahvaltı ve akşam yemeği saatleri kaçta başlıyor, kaçta bitiyor? Yemekhane gerçekleri ve Ramazan düzeni."
  image="https://kepce.org/api/v1/public/og/page/rehber"
  canonical="https://kepce.org/kyk-yemek-saatleri"
  schema={guideSchema}
/>

<ContentPage
  title="KYK Yemek Saatleri"
  dateLabel="Yayımlanma tarihi: 2026.08.20"
>
  <p>
    Yurtta hayatın önemli bir kısmı yemekhanenin çalışma saatlerine göre döner. Saatleri ve mutfağın arka planındaki işleyişi bilmek, akşam aç kalmamak veya soğuk yemekle karşılaşmamak adına işinize yarayabilir.
  </p>

  <h2>Kahvaltı</h2>
  <p>
    Hafta içi günlerde kahvaltı sabah 06:00'da başlar ve öğlen 12:00'ye kadar devam eder. Hafta sonları ve resmi tatillerde ise 06:30 – 12:30 arasında verilir.
  </p>
  <p>
    Saat 12:00'yi (hafta sonu 12:30'u) bir dakika bile geçseniz turnike ve barkod sistemi otomatik kapanır, o günkü hakkınız yanar.
  </p>

  <h2>Akşam Yemeği ve Yemekhane Gerçekleri</h2>
  <p>
    Akşam yemeği servisi 16:00'da başlar. Kapanış saati ise şehre ve döneme göre genellikle 22:00 ile 23:00 arasında değişir. Örneğin İstanbul yurtlarında eskiden 22:00 olan kapanış saati Mayıs 2026 itibarıyla 23:00'e çekilmiştir.
  </p>
  <p>
    Ancak saat kağıt üzerinde 23:00 yazsa bile işin mutfak tarafında durum biraz farklı işler. Akşam saat 21:00'den sonra yemekler soğumaya başlar ve en önemlisi popüler yemekler tükenebilir. Tavuk burger umuduyla gidip ıspanak gratenle baş başa kalmak istemiyorsanız işinizi erken halletmenizde fayda var.
  </p>
  <p>
    Ayrıca geç saatlere kalmak yemekhane çalışanlarını da yorduğu için servis kalitesi düşebilir. Kapanışa yakın gitmenin tek olası avantajı, tencerede kalan son porsiyonları ziyan olmasın diye kepçeyle biraz daha bol koyma ihtimalleridir; fakat bu her zaman tutmayan bir kumardır.
  </p>

  <h2>Ramazan Ayı Düzeni</h2>
  <p>
    Ramazan boyunca akşam yemeği iftar saatine göre ayarlanır. Normalde yalnızca kahvaltılarda verilen Al-Götür paketleri, Ramazan ayında iftar için de hazırlanabilir. Bazı yurtlar iftarda şerbet gibi küçük ikramlar çıkarsa da birkaç günden sonra genelde aynı tat bayabilir.
  </p>
  <p>
    Gece sahura kalkacaklar için ayrı bir sahur servisi açılır ve kahvaltı hakkı burada kullanılır. Oruç tutmuyor olsanız bile gece sahur saatlerinde yemekler daha taze çıktığı ve sıcak çorba verildiği için gece yemekhaneye inmek gündüzden daha mantıklı bir tercih olabilir.
  </p>

  <h2>Öğün Hakları</h2>
  <p>
    Kahvaltı hakkı akşama devretmez, kullanılmayan öğün gün sonunda sıfırlanır. Girişlerde parmak izi, yüz tanıma veya öğrenci kimlik kartı okutulur.
  </p>
  <p>
    Bugünün güncel yemek listesini görmek için <a href="/" data-link>ana sayfaya</a> göz atabilirsiniz.
  </p>
</ContentPage>
