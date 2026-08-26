<script>
  import { icon } from "@/components/ui/icons.js";
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";

  const aboutSchema = {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "AboutPage",
        "@id": "https://kepce.org/hakkinda#webpage",
        url: "https://kepce.org/hakkinda",
        name: "Hakkında | Kepçe",
        description:
          "Kepçe, KYK yurtlarında çıkan yemekleri öğrencilerin kolayca takip edebilmesi için geliştirilmiş açık kaynaklı ve bağımsız bir platformdur.",
        inLanguage: "tr-TR",
        isPartOf: {
          "@type": "WebSite",
          "@id": "https://kepce.org/#website",
          url: "https://kepce.org/",
          name: "Kepçe",
        },
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
</script>

<Seo
  title="Hakkında | Kepçe"
  description="Kepçe, KYK yurtlarında çıkan yemekleri öğrencilerin kolayca takip edebilmesi için geliştirilmiş açık kaynaklı ve bağımsız bir platformdur."
  image="https://kepce.org/api/v1/public/og/page/hakkinda"
  schema={aboutSchema}
/>

<ContentPage title="Hakkında" dateLabel="Yayımlanma tarihi: 2026.08.05">
  <p>
    Kepçe, KYK yurtlarında kalan öğrencilerin günlük yemek menülerine ve besin
    değerlerine reklamsız, hızlı ve doğrudan ulaşabilmesi için geliştirilmiş açık
    kaynaklı özgür bir yazılımdır.
  </p>

  <h2>Bu site kimin?</h2>
  <p>
    Sitede emeği geçen kişi, şu an bir KYK yurdunda kalan ve her gün aynı
    yemekhanede sıraya giren bir üniversite öğrencisi. Yani dışarıdan bir girişim
    veya şirket değil; doğrudan bu hizmeti her gün kendisi de kullanan bir öğrenci
    projesi.
  </p>
  <p>
    Görüş, öneri veya sorularınız için <a href="/iletisim">iletişim</a> sayfası
    üzerinden mesaj gönderebilirsiniz.
  </p>

  <h2>Güvenilir mi?</h2>
  <p>
    Sitede kullanıcıları takip eden üçüncü taraf reklam veya gizli takip çerezleri
    yer almaz. Kaynak kodları AGPL v3 lisansıyla tamamen açıktır; dileyen herkes
    arka planda çalışan kodları bağımsız olarak inceleyebilir, denetleyebilir ve
    geliştirme önerisinde bulunabilir.
  </p>

  <h2>Bana ne faydası var?</h2>
  <p>Kullanım amacınıza göre:</p>
  <ul>
    <li>
      <strong>Öğrenciyseniz:</strong> Yurdunuzun kahvaltı ve akşam yemeği
      menüsüne reklama boğulmadan, saniyeler içinde göz atabilirsiniz.
    </li>
    <li>
      <strong>Geliştiriciyseniz:</strong> Kendi botunuzu, arayüzünüzü veya
      otomasyonunuzu yazmak için ücretsiz ve açık API servisinden
      faydalanabilirsiniz.
    </li>
  </ul>

  <h2>Masraflar nasıl karşılanıyor? Gelir modeli nedir?</h2>
  <p>
    Sunucu ve alan adı gibi temel altyapı masrafları şahsi bütçeden
    karşılanmaktadır. Sitede reklam veya kâr amacı güden bir gelir modeli yoktur.
    Destek olmak isterseniz <a href="/iletisim">iletişim</a> sayfası üzerinden
    irtibata geçebilirsiniz.
  </p>

  <h2>Projeye nasıl katkı sağlayabilirim?</h2>
  <ul>
    <li>
      <strong>Geliştirici olarak:</strong> <a
        href="https://github.com/koder-cog/kepce"
        target="_blank"
        rel="noopener noreferrer">GitHub deposu</a
      > üzerinden kodları inceleyebilir, hata bildirebilir veya yeni özellikler için
      pull request gönderebilirsiniz.
    </li>
    <li>
      <strong>Kullanıcı olarak:</strong> Yurttaki arkadaşlarınızla paylaşabilir,
      eksik menüleri sisteme yükleyebilir veya karşılaştığınız hataları
      bildirebilirsiniz.
    </li>
  </ul>

  <p>
    Menülere yorum yazmak ve topluluğa katılmak isterseniz <a href="/kayit"
      >kayıt olabilir</a
    >; zaten hesabınız varsa doğrudan <a href="/giris">giriş yapabilirsiniz</a>.
  </p>
</ContentPage>
