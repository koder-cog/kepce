<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";

  const guideSchema = {
    "@context": "https://schema.org",
    "@type": "Article",
    headline: "KYK Beslenme Yardımı 2026: Günlük Yemek Yardımı ve Kullanım Kuralları",
    description: "KYK yurtlarında günlük yemek yardımı ne kadar? Sabah kahvaltısı ve akşam yemeği beslenme yardımı limitleri ve aşım kuralları.",
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
  title="KYK Beslenme Yardımı 2026 (Günlük Yemek Yardımı) | Kepçe"
  description="KYK yurtlarında günlük yemek yardımı ve limit ne kadar? Sabah kahvaltısı ve akşam yemeği beslenme yardımı kullanım şartları ve detayları."
  image="https://kepce.org/api/v1/public/og/page/rehber"
  canonical="https://kepce.org/kyk-beslenme-yardimi"
  schema={guideSchema}
/>

<ContentPage
  title="KYK Beslenme Yardımı Rehberi (2026 - 2027)"
  dateLabel="Son Güncelleme: Ağustos 2026 (2026-2027 Akademik Yılı)"
>
  <p>
    Gençlik ve Spor Bakanlığı (GSB) tarafından KYK yurtlarında barınan tüm öğrencilere her gün sabah kahvaltısı ve akşam yemeği için nakdi beslenme yardımı tanımlanır. Bu yardım yurt ücretine dahil olup doğrudan yemekhanelerde geçerlidir.
  </p>

  <h2>Beslenme Yardımı Nasıl Kullanılır?</h2>
  <p>
    Yemekhanede tabldot veya alakart yemek seçimi yaptıktan sonra kasada kimlik kartınızı okutarak veya biyometrik doğrulama (parmak izi / yüz tanıma) ile yardım limitinizi anında düşebilirsiniz.
  </p>

  <h2>Temel Kurallar ve Sık Yapılan Hatalar</h2>
  <ul>
    <li><strong>Öğün Başına Limit:</strong> Beslenme yardımı sabah ve akşam öğünleri için ayrı ayrı tanımlanmıştır. Sabah hakkı sadece sabah kahvaltısı saatlerinde, akşam hakkı ise sadece akşam yemeği saatlerinde geçerlidir.</li>
    <li><strong>Limit Aşımı:</strong> Belirlenen menü veya sepet tutarı günlük beslenme yardımını aşarsa aradaki farkı banka/kredi kartı ile kasada ödeyebilirsiniz.</li>
    <li><strong>Nakit Olarak Alınamaz:</strong> Beslenme yardımı nakit paraya dönüştürülemez ve bir sonraki güne devredilemez. Kullanılmayan haklar gün sonunda sıfırlanır.</li>
    <li><strong>Yurt Dışı / İzin Durumu:</strong> Yurttan izinli olduğunuz günlerde veya tatil dönemlerinde yemekhaneye gitmediğiniz takdirde o günün yemek yardımı birikmez.</li>
  </ul>

  <h2>Günlük Menüyü Takip Edin</h2>
  <p>
    Yurdunuzun bugünkü yemek listesini ve besin değerlerini kontrol etmek için <a href="/" data-link>Günün Menüsü</a> veya geçmiş yemekleri incelemek için <a href="/arsiv" data-link>Arşiv</a> sayfamızı ziyaret edebilirsiniz.
  </p>
</ContentPage>
