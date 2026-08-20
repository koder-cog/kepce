<!--
  Kepçe - Sıkça Sorulabilecek Sorular (SSS) Sayfası
  İlk Svelte 5 bileşeni. Mevcut CSS sınıflarını (content-page) kullanır.
-->
<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";

  const faqSchema = {
    "@context": "https://schema.org",
    "@type": "FAQPage",
    mainEntity: [
      {
        "@type": "Question",
        name: "Menülere yaptığım yorumlar neden denetleniyor?",
        acceptedAnswer: {
          "@type": "Answer",
          text: "Tek kişi olduğum için işleri kolaylaştırmak adına ön denetim kararı aldım. Niyet web tabanlı 1984 değil, üşengeçlik.",
        },
      },
      {
        "@type": "Question",
        name: "Menü verilerini nereden alıyorsunuz?",
        acceptedAnswer: {
          "@type": "Answer",
          text: "Kişisel olarak elde edebildiğim dosyalardan, veri erişimine olanak sağlayan diğer sitelerden ve diğer yurt öğrencilerinin gönderdiği menülerden derleniyor.",
        },
      },
      {
        "@type": "Question",
        name: "Site hata verirse veya menü yanlış çıkarsa ne yapmalıyım?",
        acceptedAnswer: {
          "@type": "Answer",
          text: "Mutfaktaki kelebek etkileri menüyü değiştirebilir. Menü yanlışsa Hata Bildir butonu veya iletişim sayfası üzerinden bildirebilirsiniz.",
        },
      },
      {
        "@type": "Question",
        name: "Neden AGPL v3 lisansı?",
        acceptedAnswer: {
          "@type": "Answer",
          text: "Kimse kodu alıp kapatmasın diye en katı copyleft lisanslardan biri seçildi. Projenin açık kaynak olması geliştirilmesini ve bayrağın devredilmesini kolaylaştırır.",
        },
      },
      {
        "@type": "Question",
        name: "Kepçe Bot Nedir?",
        acceptedAnswer: {
          "@type": "Answer",
          text: "Projenin arka plandaki otomasyon araçlarının ve dil modeli kullanarak menüleri yorumlayan sistemin genel adıdır.",
        },
      },
      {
        "@type": "Question",
        name: "Kepçe Bot'u (YZ) Kapatabilir miyim?",
        acceptedAnswer: {
          "@type": "Answer",
          text: "Evet. Kepçe Bot varsayılan olarak açık gelse de ayarlardan tamamen kapatılabilir, opsiyoneldir.",
        },
      },
    ],
  };
</script>

<Seo
  title="Sıkça Sorulabilecek Sorular | Kepçe"
  description="Kepçe platformu, KYK yurt yemek menüleri, verilerin doğruluğu ve topluluk kuralları hakkında sıkça sorulan sorular."
  image="https://kepce.org/api/v1/public/og/page/sss"
  schema={faqSchema}
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

  <h2 id="yorum-denetimi">Menülere yaptığım yorumlar neden denetleniyor?</h2>
  <p>
    Normalde yasalara göre ön denetimden geçirilme zorunluluğu bulunmamakta
    ancak tek kişi olduğum için işleri kolaylaştırmak adına böyle bir karar
    aldım ama bu olay <a href="/kullanim-kosullari" data-link
      >kullanım koşulları</a
    >nda da belirtildiği gibi editöryel değildir.
  </p>
  <p>
    Eğer yorumunuz onaylanmıyorsa <a href="/iletisim" data-link>iletişim</a> sayfasından
    iletişime geçebilirsiniz. Niyet web tabanlı 1984 değil, üşengeçlik.
  </p>

  <!--
    <h2 id="karma-seviye-ne-ise-yariyor">
      Karma puanı veya seviyeler nedir ve işe yarıyorlar?
    </h2>
    <p>
      Karma, kazandığınız pozitif oylar veya rozetler ile elde ettiğiniz deneyim
      puanlarıdır. Seviye ise bu karma puanlarının toplamı ile aşılan bir
      deneyim eşiğidir. Şimdilik sadece profilinizi süsleyen, ufak bir dijital
      tatmin aracı ve muhtemel psikolojik bir kast sisteminden ibaretler.
    </p>
    -->

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
    opsiyoneldir. Amaç sadece deneyimi bir tık daha katlanılabilir kılmak.
  </p>
</ContentPage>
