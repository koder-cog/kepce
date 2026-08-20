<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";

  const guideSchema = {
    "@context": "https://schema.org",
    "@type": "Article",
    headline: "KYK Yemek Saatleri 2026: Kahvaltı ve Akşam Yemeği Saatleri",
    description: "KYK yurtlarında sabah kahvaltısı ve akşam yemeği saatleri kaçta başlıyor, kaçta bitiyor? Hafta sonu ve Ramazan saat düzeni.",
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
  title="KYK Yemek Saatleri 2026 (Kahvaltı ve Akşam Yemeği) | Kepçe"
  description="KYK yurtlarında sabah kahvaltısı ve akşam yemeği saatleri kaçta başlıyor, kaçta bitiyor? Hafta içi, hafta sonu ve Ramazan saatleri tablosu."
  image="https://kepce.org/api/v1/public/og/page/rehber"
  canonical="https://kepce.org/kyk-yemek-saatleri"
  schema={guideSchema}
/>

<ContentPage
  title="KYK Yemek Saatleri (2026 - 2027)"
  dateLabel="Son Güncelleme: Ağustos 2026 (2026-2027 Akademik Yılı)"
>
  <p>
    Kredi ve Yurtlar Kurumu (KYK) yurtlarında yemekhane saatleri, öğrencilerin ders ve sınav programlarına göre sabah ve akşam olmak üzere iki ayrı öğün şeklinde uygulanır.
  </p>

  <h2>Günlük Öğün Saatleri Tablosu</h2>

  <div class="table-responsive">
    <table>
      <thead>
        <tr>
          <th>Öğün</th>
          <th>Hafta İçi Saatleri</th>
          <th>Hafta Sonu & Resmi Tatil</th>
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

  <h2>Önemli Kurallar ve Bilinmesi Gerekenler</h2>
  <ul>
    <li><strong>Geçiş Kotası:</strong> Sabah kahvaltısı saat 12:00'de (hafta sonu 12:30'da) sona erer. Bu saatten sonra sabah beslenme yardımı hakkı kullanılamaz.</li>
    <li><strong>Parmak İzi / Kartlı Okutma:</strong> Yemekhaneye girişlerde öğrenci kimlik kartı veya parmak izi / yüz tanıma sistemi kullanılır.</li>
    <li><strong>Ramazan Ayı Düzeni:</strong> Ramazan ayında akşam yemeği saatleri iftar vaktine göre ayarlanır ve gece sahur öğünü servisi açılır.</li>
    <li><strong>Hak Devri Yoktur:</strong> Sabah kahvaltısı hakkı akşam yemeğine devredilemez; kullanılmayan öğün yardımı o gün için geçerliliğini yitirir.</li>
  </ul>

  <h2>Bugün Yurdunuzda Ne Yemek Var?</h2>
  <p>
    Yurdunuzda bugün servis edilen tabldot menüsünü, kalori ve besin değerlerini görmek için <a href="/" data-link>Günün Menüsü</a> sayfasına göz atabilirsiniz.
  </p>
</ContentPage>
