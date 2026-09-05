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
    SSS sayfaları, yazılımcıların insanlığın okuduğunu anlama konusundaki
    yetersizliğine karşı aldığı umutsuz bir önlemdir. Ben de bu önlemi siz
    sormadan alayım istedim.
  </p>

  <h2 id="verileri-nereden-aliyorsunuz">
    Menü verilerini nereden alıyorsunuz?
  </h2>
  <p>
    Kişisel olarak elde edebildiğim dosyalardan (biz de yurtta kalıyoruz
    sonuçta), veri erişimine olanak sağlayan diğer sitelerden ve diğer yurt
    mağdurlarının gönderdiği eklerden derleniyor. Elinizde menü varsa <a
      href="/menu-gonder"
      data-link>şuracıktan</a
    > göndermeniz beni müteşekkir kılar.
  </p>

  <h2 id="hata-verirse-ne-yapmaliyim">
    Site hata verirse veya menü yanlış çıkarsa ne yapmalıyım?
  </h2>
  <p>
    Önce telefonu yavaşça yere bırakın ve derin bir nefes alın zira mutfaktaki
    aşçının o gün eşiyle kavga etmiş olması, ihalelere fesat karışması veya
    salçanın kamyonda kalması gibi kelebek etkileri menüyü değiştirebilir.
    Ayrıca 81 ilde kuantum dolanıklığına sahip olmadığım için bütün menüleri
    doğrulayamam.
  </p>
  <p>
    Eğer site hata veriyorsa, muhtemelen sunucu o anki yükü kaldıramamıştır.
    Hata koduna ve açıklamasına bakın (404, 500 vb.), sitede standart HTTP
    kodları kullanıldığı için Google amca yolu gösterecektir, eğer gösteremezse <a
      href="/iletisim"
      data-link>iletişim</a
    > sayfamız her zaman açıktır. Menü yanlışsa "Hata Bildir" butonuna bastığınız
    takdirde size bazı güzellikler olabilir (olmayabilir de).
  </p>

  <h2 id="neden-agpl-v3">Neden AGPL v3 lisansı?</h2>
  <p>
    Zannımca "kimse kodu alıp kapatmasın" fikri hoşuma gitti. MIT seçebilirdim,
    Apache seçebilirdim ancak gönlüm en katı copyleft lisanslardan birini seçti.
    Artı olarak bu projeyi açık kaynak yapmak; açıkların bulunmasını ve ben
    mezun olunca bu bayrağı başka bir öğrencinin taşımasını kolaylaştırır...
    eğer taşıyacak biri olursa tabii.
  </p>

  <h2 id="kepce-bot-nedir">Kepçe Bot Nedir?</h2>
  <p>
    Kepçe Bot, projenin arka plandaki otomasyon zımbırtılarının ve günlük menü
    yorumlayıcısının genel adıdır. Kaynak ayıklayıcıdan YZ deneyimine kadar
    birçok fasilitesi vardır.
  </p>
  <p>
    Yemek menülerinin yanında zuhur eden Kepçe Bot ise, bir dil modeli
    kullanılarak ayda bir günlük menüleri yorumlar. Amacı, deneyime ufak tefek
    teknolojik fanteziler katmaktır.
  </p>

  <h2 id="kepce-botu-kapatabilir-miyim">Kepçe Bot'u (YZ) Kapatabilir miyim?</h2>
  <p>
    Tabii lan manyak mısın. Kepçe Bot, varsayılan olarak açık gelse de tamamen
    opsiyoneldir. Menü kartlarında YZ yorumu görmek istemiyorsanız <a
      href="/ayarlar"
      data-link>Ayarlar</a
    > sayfasından tek tıkla kapatabilirsiniz.
  </p>
</ContentPage>
