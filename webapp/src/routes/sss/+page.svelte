<!--
  Kepçe - Sıkça Sorulabilecek Sorular (SSS) Sayfası
-->
<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";

  const sssSchema = {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "BreadcrumbList",
        itemListElement: [
          { "@type": "ListItem", position: 1, name: "Ana Sayfa", item: "https://kepce.org/" },
          { "@type": "ListItem", position: 2, name: "Sıkça Sorulan Sorular", item: "https://kepce.org/sss" },
        ],
      },
      {
        "@type": "WebPage",
        "@id": "https://kepce.org/sss#webpage",
        url: "https://kepce.org/sss",
        name: "Sıkça Sorulabilecek Sorular | Kepçe",
        description:
          "Kepçe'nin çalışma mantığı, KYK yemek menüleri ve sistem hakkında merak edilebilecekler.",
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
</script>

<Seo
  title="Sıkça Sorulabilecek Sorular | Kepçe"
  description="Kepçe'nin çalışma mantığı, KYK yemek menüleri ve sistem hakkında merak edilebilecekler."
  image="https://kepce.org/api/v1/public/og/page/sss"
  schema={sssSchema}
/>

<ContentPage
  title="Sıkça Sorulabilecek Sorular"
  dateLabel="Yayımlanma tarihi: 2026.08.05"
>
  <p>
    Aklınıza takılabilecek teknik veya pratik soruların kısa ve net cevapları.
  </p>

  <h2 id="yorum-denetimi">Menülere yaptığım yorumlar neden denetleniyor?</h2>
  <p>
    Sitede spam, küfür veya bot istilasını önlemek adına otomatik ve yarı-otomatik
    filtreler devrededir. Bu süreç <a href="/kullanim-kosullari" data-link
      >kullanım koşulları</a
    >nda belirtildiği üzere editoryal bir sansür değil, tek kişilik bir projede
    düzeni korumak için pratik bir önlemdir.
  </p>
  <p>
    Yorumunuzun hatalı elendiğini düşünüyorsanız <a href="/iletisim" data-link>iletişim</a> sayfası üzerinden yazabilirsiniz.
  </p>

  <h2 id="verileri-nereden-aliyorsunuz">
    Menü verilerini nereden alıyorsunuz?
  </h2>
  <p>
    Doğrudan yurt panolarından çekilen fotoğraflardan, açık veri kaynaklarından ve
    yurtta kalan diğer öğrencilerin gönderdiği menü belgelerinden derleniyor.
    Elinizde güncel bir menü varsa <a href="/menu-gonder" data-link>Menü Gönder</a> sayfası üzerinden sisteme yükleyebilirsiniz.
  </p>

  <h2 id="hata-verirse-ne-yapmaliyim">
    Site hata verirse veya menü yanlış çıkarsa ne yapmalıyım?
  </h2>
  <p>
    Mutfakta son dakika malzeme eksikliği, aşçının tarifi değiştirmesi veya yemekhanenin
    plansız menü değişikliği yapması gibi durumlar yaşanabilir. 81 ildeki tüm yurtları
    aynı anda fiziksel olarak denetlemek imkansız olduğu için menüde hata görürseniz
    ilgili menü kartındaki <strong>Hata Bildir</strong> butonunu kullanabilirsiniz.
  </p>
  <p>
    Site genelinde bir çökme veya teknik aksaklık yaşanırsa durum sayfasından kontrol
    edebilir, çözülmeyen sorunlar için <a href="/iletisim" data-link>iletişim</a> sayfasından bildirebilirsiniz.
  </p>

  <h2 id="neden-agpl-v3">Neden AGPL v3 lisansı?</h2>
  <p>
    Kodların her zaman açık ve erişilebilir kalması için en katı copyleft lisanslardan
    biri seçildi. Böylece kimse kodu alıp kapatamaz ve yarın bir gün bayrağı başka bir
    öğrenci devralmak istediğinde proje özgürce yaşamaya devam eder.
  </p>

  <h2 id="kepce-bot-nedir">Kepçe Bot nedir?</h2>
  <p>
    Kepçe Bot, projenin arka plandaki veri çekme otomasyonlarının ve menü
    özetleyicisinin genel adıdır. Günlük menüleri inceleyerek öğrenci gözüyle
    küçük değerlendirmeler ve istatistiksel çıkarımlar üretir.
  </p>

  <h2 id="kepce-botu-kapatabilir-miyim">Kepçe Bot'u kapatabilir miyim?</h2>
  <p>
    Tabii ki. Kepçe Bot varsayılan olarak açık gelse de tamamen opsiyoneldir. Menü
    kartlarında YZ yorumu görmek istemiyorsanız <a href="/ayarlar" data-link>Ayarlar</a> sayfasından tek tıkla kapatabilirsiniz.
  </p>
</ContentPage>
