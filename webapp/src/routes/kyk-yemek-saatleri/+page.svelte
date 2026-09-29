<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";
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
            name: "KYK Yemek Saatleri",
            item: "https://kepce.org/kyk-yemek-saatleri",
          },
        ],
      },
      {
        "@type": "WebPage",
        "@id": "https://kepce.org/kyk-yemek-saatleri#webpage",
        url: "https://kepce.org/kyk-yemek-saatleri",
        name: "KYK Yemek Saatleri | Kepçe",
        description:
          "KYK yurtlarında kahvaltı ve akşam yemeği saatleri, hafta sonu saatleri, Al Götür paketleri ve Ramazan ayı düzeni.",
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
    { label: "Hafta içi kahvaltı", value: "06:00 - 12:00" },
    { label: "Hafta sonu kahvaltı", value: "06:00 - 12:30" },
    { label: "Al Götür kahvaltı", value: "06:00 - 10:30" },
    { label: "Sahur", value: "00:00 - 06:00" },
    { label: "Akşam yemeği", value: "16:00 - 22:00 / 23:00", wide: true },
  ];

  let { data } = $props();
</script>

<Seo
  title={data?.seoTitle || "KYK Yemek Saatleri | Kepçe"}
  description={data?.seoDescription ||
    "KYK yurtlarında kahvaltı ve akşam yemeği saatleri, hafta sonu saatleri, Al Götür paketleri ve Ramazan ayı düzeni."}
  image={data?.ogImage || "https://kepce.org/api/v1/public/og/page/rehber"}
  canonical="https://kepce.org/kyk-yemek-saatleri"
  schema={guideSchema}
/>

<ContentPage
  title="KYK Yemek Saatleri"
  dateLabel="Son güncelleme: 28 Eylül 2026"
>
  <GuideSummary
    title="Öğün Saatleri"
    rows={summaryRows}
    note="Saatler yurttan yurda farklılık gösterebilir. Kapanış saatinden sonra turnike sistemi otomatik olarak kapanır ve kullanılmayan yemek hakkı yanar."
  />

  <p>
    Yurtta günlük düzenin önemli bir kısmı yemekhanenin çalışma saatlerine göre
    şekillenir. Yemek saatlerini ve mutfağın işleyişini bilmek, akşam aç
    kalmanızı veya soğuk yemekle karşılaşmanızı önler.
  </p>

  <h2>Kahvaltı</h2>
  <p>
    Hafta içi kahvaltı 06:00'da başlar ve 12:00'ye kadar sürer. Hafta sonlarında
    ise servis 12:30'a kadar uzatılır. Belirlenen sürenin sonunda turnike ve
    barkod sistemi kapanır. Kullanılmayan kahvaltı hakkı akşama devredilmez.
  </p>
  <p>
    Yemekhanede oturacak vakti olmayan veya erken çıkması gereken öğrenciler
    için Al Götür seçeneği de bulunur ancak bu paketlerin servisi normal
    kahvaltıdan daha erken, genellikle 10:30'da biter. Paket içeriği yurttan
    yurda değişir. Genelde gözleme veya tost gibi sıcak seçenekler verilirken
    bazı günler soğuk sandviç ve meyve suyu tipi paketler çıkar.
  </p>

  <h2>Akşam Yemeği</h2>
  <p>
    Akşam yemeği servisi 16:00'da başlar. Kapanış saati ise şehre ve döneme göre
    genellikle 22:00 ile 23:00 arasında değişir. Örneğin İstanbul yurtlarında
    kapanış saati 23:00 olarak uygulanır.
  </p>
  <p>
    Kapanış saati kâğıt üzerinde 23:00 yazsa da mutfakta işler biraz farklı
    ilerler. Saat 21:00'den sonra yemekler soğumaya başlar ve popüler menüler
    hızla tükenir. Tavuk burger umuduyla gidip ıspanak gratenle baş başa kalmak
    istemiyorsanız yemeği son saatlere bırakmamakta fayda var.
  </p>
  <p>
    Kapanışa yakın gitmenin tek olası avantajı, tencerede kalan son
    porsiyonların ziyan olmasın diye kepçeyle biraz daha bol konulmasıdır fakat
    bu her zaman tutmayan bir kumardır.
  </p>

  <h2>Ramazan Ayı Düzeni</h2>
  <p>
    Ramazan boyunca akşam yemeği servisi yine 16:00'da başlar ve isteyen erken
    saatte yiyebilir. Ancak asıl yoğunluk iftara dakikalar kala yaşanır. Oruç
    tutuyor olsanız bile yemekhaneye iftardan 10-15 dakika sonra gitmek çok daha
    akıllıcadır çünkü tam ezan anındaki izdihamda "herkese yetsin" telaşıyla
    tabaklara porsiyonların biraz daha çekinerek konulduğuna rastlayabilirsiniz.
  </p>
  <p>
    İftar için hazırlanan Al Götür paketleri genellikle 17:00-18:00 arasında
    çıkarılır ve sayısı oldukça azdır. Bazı yurtların iftarda ikram ettiği
    boyalı şerbetler ise genelde ilk birkaç günden sonra kimsenin yüzüne
    bakmadığı bir ayrıntıya dönüşür.
  </p>
  <p>
    Gece sahura kalkacaklar için ayrı bir servis açılır (00:00 - 06:00). Sahur
    menüsü sabah kahvaltısı ile aynı ürünleri sunar, tek farkı menüye fazladan
    sıcak çorba eklenmesidir. Sahur servisi takvim günü mantığıyla o günün yeni
    başlayan kahvaltı kotasını kullanır. Dolayısıyla sahurda yemek alan öğrenci,
    aynı günün sabah kahvaltısı hakkını tüketmiş sayılır.
  </p>

  <h2>Öğün Hakları</h2>
  <p>
    Yemekhane girişlerinde parmak izi, yüz tanıma ya da öğrenci kimlik kartı
    okutularak öğün hakkı sorgulanır. Günlük bütçe kotaları, standart tabldot
    kapsamı ve tavan fiyat kuralları hakkında ayrıntılı bilgi için
    <a href="/kyk-beslenme-yardimi" data-link>KYK Beslenme Yardımı</a>
    rehberine göz atabilirsiniz.
  </p>
  <p>
    Bugünün güncel yemek listesini görmek için <a href="/" data-link
      >ana sayfaya</a
    > göz atabilirsiniz.
  </p>
</ContentPage>
