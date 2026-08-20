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
  description="KYK yurtlarında sabah kahvaltısı ve akşam yemeği saatleri kaçta başlıyor, kaçta bitiyor? Hafta içi ve hafta sonu yemekhane saat tablosu."
  image="https://kepce.org/api/v1/public/og/page/rehber"
  canonical="https://kepce.org/kyk-yemek-saatleri"
  schema={guideSchema}
/>

<ContentPage
  title="KYK Yemek Saatleri"
  dateLabel="Yayımlanma tarihi: 2026.08.20"
>
  <p>
    Kredi ve Yurtlar Kurumu yurtlarında yemekhane saatleri, öğrencilerin ders programlarına göre sabah ve akşam olmak üzere iki ayrı öğün şeklinde uygulanır.
  </p>

  <h2>Öğün Saatleri</h2>

  <div class="table-responsive">
    <table>
      <thead>
        <tr>
          <th>Öğün</th>
          <th>Hafta İçi</th>
          <th>Hafta Sonu ve Tatiller</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td><strong>Sabah Kahvaltısı</strong></td>
          <td>06:00 – 12:00</td>
          <td>06:30 – 12:30</td>
        </tr>
        <tr>
          <td><strong>Akşam Yemeği</strong></td>
          <td>16:00 – 22:30</td>
          <td>16:00 – 22:30</td>
        </tr>
      </tbody>
    </table>
  </div>

  <h2>Bilinmesi Gerekenler</h2>
  <ul>
    <li><strong>Saat Sınırı:</strong> Sabah kahvaltısı saat 12:00'de (hafta sonu 12:30'da) biter. Bu saatten sonra sabah hakkı kullanılamaz.</li>
    <li><strong>Giriş Sistemi:</strong> Yemekhaneye girişlerde öğrenci kartı, parmak izi veya yüz tanıma sistemi kullanılır.</li>
    <li><strong>Ramazan Düzeni:</strong> Ramazan ayında akşam yemeği iftar vaktine göre ayarlanır ve gece sahur servisi açılır.</li>
    <li><strong>Hak Devri:</strong> Sabah hakkı akşama devretmez, kullanılmayan öğün o gün için yanar.</li>
  </ul>

  <h2>Günün Menüsü</h2>
  <p>
    Yurdunuzda bugün çıkan tabldot menüsünü ve yemekleri görmek için <a href="/" data-link>Günün Menüsü</a> sayfasına bakabilirsiniz.
  </p>
</ContentPage>
