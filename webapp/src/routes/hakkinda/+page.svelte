<script>
  import { icon } from "@/components/ui/icons.js";
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";
</script>

<Seo
  title="Hakkında - Kepçe"
  description="Kepçe; KYK yurtlarında kalan öğrencilerin günlük yemek menülerine şeffaf, reklamsız ve hızlıca ulaşması için geliştirilmiş açık kaynaklı ve bağımsız bir platformdur."
  image="https://kepce.org/api/v1/public/og/page/hakkinda"
/>

<ContentPage title="Hakkında" dateLabel="Yayımlanma tarihi: 2026.08.05">
  <p>
    Kepçe, temel olarak API sağlayıcı görevini üstlenen açık kaynaklı ve kâr
    amacı gütmeyen özgür bir yazılımdır. Kepçe'nin temel değerlerinde
    kullanıcıların öğün verilerine reklamsız, hızlı ve doğrudan ulaşabilmesi
    vardır.
  </p>

  <h2>Bu site kimin?</h2>
  <p>
    Benim... Peki kimim ben? Güncel olarak bir KYK yurdunda kalan ve KYK
    tabldotlarından beslenen sıradan bir üniversite öğrencisiyim. Yani doğrudan
    sizin gibi biriyim ve bu sitenin asıl kullanıcısıyım.
  </p>
  <p>
    Eğer benimle site ile ilgili belirli bir konuda iletişime geçmek isterseniz <a
      href="/iletisim">iletişim</a
    > sayfamıza bekleriz.
  </p>

  <h2>Güvenilir mi?</h2>
  <p>
    Proje, güncel güvenlik standartları ve kabul görmüş pratikler üzerine inşa
    edildiği için temel güvenlik standartlarını karşılamaktadır. Artı olarak
    projenin kaynak kodları açıktır; yani istenirse kodlar bağımsız olarak
    incelenebilir, denetlenebilir ve herhangi bir hata veya geliştirme önerisi
    için kodlara katkıda bulunulabilir.
  </p>

  <h2>Bana ne faydası var?</h2>
  <p>Sizin kim olduğunuza göre değişir:</p>
  <ul>
    <li>
      Düz kullanıcıysanız, yurdunuzun menüsüne reklamlara boğulmadan, saniyeler
      içinde bakıp çıkabilirsiniz.
    </li>
    <li>
      Geliştiriciyseniz ve kendi arayüzünüzü geliştirmek veya otomasyon kurmak
      istiyorsanız; Kepçe size diğer platformların sunmadığı ücretsiz bir API
      sağlar.
    </li>
  </ul>

  <h2>Masraflar kimin cebinden çıkıyor? Gelir modeliniz nedir?</h2>
  <p>
    Alan adı ve sunucu gibi temel altyapı masrafları şu an tamamen benim
    cebimden çıkıyor. Sitede kurulu bir bağış sistemi yok ancak çorbada tuzum
    bulunsun derseniz <a href="/iletisim">iletişim</a> sayfası üzerinden benimle
    iletişime geçebilirsiniz.
  </p>

  <h2>Projeye nasıl katkı sağlayabilirim?</h2>
  <ul>
    <li>
      Eğer geliştirici iseniz <a
        href="https://github.com/koder-cog/kepce"
        target="_blank"
        rel="noopener noreferrer">GitHub</a
      > depomuz üzerinden kodları inceleyebilir, hata çözümlerinde bulunabilir veya
      yeni özellikler için PR gönderebilirsiniz.
    </li>
    <li>
      Eğer kullanıcı iseniz sitede gördüğünüz hataları bize bildirebilirsiniz
      veya diğer insanlara göstererek daha fazla kişinin Kepçe'den haberdar
      olmasını sağlayabilirsiniz. En büyük katkı uygulamanın kullanılmasıdır.
    </li>
  </ul>

  <p>
    Menülere yorum yazmak, toplulukla sohbet etmek veya API'den faydalanmak
    isterseniz <a href="/kayit">kayıt olabilir</a>; halihazırda bir Kepçe
    kullanıcısıysanız doğrudan <a href="/giris">giriş yapabilirsiniz</a>.
  </p>
</ContentPage>
