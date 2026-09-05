<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";

  const guideSchema = {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "BreadcrumbList",
        itemListElement: [
          { "@type": "ListItem", position: 1, name: "Ana Sayfa", item: "https://kepce.org/" },
          { "@type": "ListItem", position: 2, name: "KYK Yemek Saatleri", item: "https://kepce.org/kyk-yemek-saatleri" },
        ],
      },
      {
        "@type": "Article",
        headline: "KYK Yemek Saatleri",
        description:
          "KYK yurtlarında kahvaltı ve akşam yemeği saatleri, yemekhane gerçekleri ve Ramazan düzeni.",
        datePublished: "2026-08-20",
        dateModified: "2026-08-20",
        inLanguage: "tr-TR",
        mainEntityOfPage: "https://kepce.org/kyk-yemek-saatleri",
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
  title="2026-2027 KYK Yemek Saatleri (Sabah & Akşam) | Kepçe"
  description="2026-2027 GSB KYK yurtlarında kahvaltı ve akşam yemeği saatleri kaçta başlıyor, kaçta bitiyor? Hafta sonu, Al-Götür ve Ramazan düzeni rehberi."
  image="https://kepce.org/api/v1/public/og/page/rehber"
  canonical="https://kepce.org/kyk-yemek-saatleri"
  schema={guideSchema}
/>

<ContentPage
  title="KYK Yemek Saatleri"
  dateLabel="Yayımlanma tarihi: 2026.08.20"
>
  <p>
    Yurtta hayatın önemli bir kısmı yemekhanenin çalışma saatlerine göre döner.
    Saatleri ve mutfağın arka planındaki işleyişi bilmek, akşam aç kalmamak veya
    soğuk yemekle karşılaşmamak adına işinize yarayabilir.
  </p>

  <h2>Kahvaltı</h2>
  <p>
    Hafta içi günlerde kahvaltı sabah 06:00'da başlar ve öğlen 12:00'ye kadar
    devam eder. Hafta sonlarında ise 12:30'a kadar verilir. Saat 12:00'yi (veya
    12:30'u) bir dakika bile geçseniz turnike ve barkod sistemi otomatik
    kapanır, o günkü hakkınız yanar.
  </p>
  <p>
    Eğer yemekhanede oturup yemek istemiyorsanız veya okula giderken yanınıza
    almanız gerekiyorsa Al Götür Menüler adlı bir seçenek de bulunur ancak bu
    paketlerin servisi normal kahvaltı saatinden çok daha erken biter (örneğin
    İstanbul yurtlarında 10:30 idi). Yurttan yurda içerik değişmekle birlikte
    gözlemeli olanlar genelde iyi çıkarken diğerleri eh işte seviyesindedir,
    kağıt üzerinde görünen çorbalı varyantın ise fiilen verildiğine pek
    rastlanmaz.
  </p>

  <h2>Akşam Yemeği</h2>
  <p>
    Akşam yemeği servisi 16:00'da başlar. Kapanış saati ise şehre ve döneme göre
    genellikle 22:00 ile 23:00 arasında değişir. Örneğin İstanbul yurtlarında
    eskiden 22:00 olan kapanış saati Mayıs 2026 itibarıyla 23:00'e çekilmiştir.
  </p>
  <p>
    Ancak saat kağıt üzerinde 23:00 yazsa bile işin mutfak tarafında durum biraz
    farklı işler. Akşam saat 21:00'den sonra yemekler soğumaya başlar ve en
    önemlisi popüler yemekler tükenebilir. Tavuk burger umuduyla gidip ıspanak
    gratenle baş başa kalmak istemiyorsanız işinizi erken halletmenizde fayda
    var.
  </p>
  <p>
    Ayrıca geç saatlere kalmak yemekhane çalışanlarını da yorduğu için servis
    kalitesi düşebilir. Kapanışa yakın gitmenin tek olası avantajı, tencerede
    kalan son porsiyonları ziyan olmasın diye kepçeyle biraz daha bol koyma
    ihtimalleridir fakat bu her zaman tutmayan bir kumardır.
  </p>

  <h2>Ramazan Ayı Düzeni</h2>
  <p>
    Ramazan boyunca akşam yemeği yine 16:00'da başlar ve isteyen erken saatte
    yiyebilir. Ancak asıl devasa kuyruklar ve yoğunluk iftara dakikalar kala
    oluşur. Oruç tutuyor olsanız bile iftardan en az 10-15 dakika sonra gitmeniz
    tavsiye edilir; çünkü tam iftar anındaki izdihamda "herkese yetsin"
    telaşıyla porsiyonların biraz daha az konulduğuna şahit olabilirsiniz.
  </p>
  <p>
    İftar için hazırlanan Al Götür menüler ise genellikle 17:00 – 18:00
    civarında çıkarılır. Net bir saati olmasa da iftara yakın dağıtılır ve
    miktarı oldukça azdır. Bazı yurtların iftarda verdiği şerbet gibi ufak
    ikramlar da ilk günlerden sonra genelde baymaya başlar.
  </p>
  <p>
    Gece sahura kalkacaklar için ayrı bir sahur servisi açılır ve kahvaltı hakkı
    burada kullanılır. Oruç tutmuyor olsanız bile gece sahur saatlerinde
    yemekler daha taze çıktığı ve sıcak çorba verildiği için gece yemekhaneye
    inmek gündüzden daha mantıklı bir tercih olabilir.
  </p>

  <h2>Öğün Hakları</h2>
  <p>
    Kahvaltı hakkı akşama devretmez, kullanılmayan öğün gün sonunda sıfırlanır.
    Girişlerde parmak izi, yüz tanıma veya öğrenci kimlik kartı okutulur.
  </p>
  <p>
    Bugünün güncel yemek listesini görmek için <a href="/" data-link
      >ana sayfaya</a
    > göz atabilirsiniz.
  </p>
</ContentPage>
