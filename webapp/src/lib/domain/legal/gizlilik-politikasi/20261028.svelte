<section class="giris-kutusu">
  <h2>0. Başlamadan Önce</h2>
  <p>
    Kepçe, KYK yurtlarındaki tabldot menülerini her gün oradan oraya aramadan tek bir yerden
    görebilmek için geliştirdiğim bir projedir.
  </p>
  <p>
    Bu sitede kimsenin kişisel verisini toplamıyorum çünkü verinizi toplasam benim ne işime
    yarayacak? Bunu geçtim, bugünün internetinde çoğu sitenin kullanıcıyı sağılacak birer inek
    olarak görmesinden bıktım çünkü ben de aynı yemekhane sırasını bekleyen ve özgürce otlanmak
    isteyen bir ineğim.
  </p>
  <p>
    Üstelik bir yanda 6698 sayılı KVKK diğer yanda 5651 sayılı İnternet Kanunu yan yana konulunca
    Kafkaesk bir paradoks oluşturuyor. Öğrenci bütçesiyle sunucuyu yurt dışında tutmak zorunda
    olduğumdan, bu iki kanunun arasında ezilmemek ve kimsenin verisinin yükünü taşımamak için
    sistemi Fransa'da hiçbir kişisel veriyi şifresiz depolamayacak şekilde kurdum. Veri sandığı da o sandığı
    açacak anahtar da Türkiye'de internete kapalı bir ortamda duruyor.
  </p>
</section>

<p>
  Sitede iki farklı kullanım durumu var ve sistem ikisinde de olabildiğince az veriyle çalışıyor. Bu
  metin aynı zamanda KVKK kapsamındaki resmi aydınlatma bildirimidir.
</p>

<section>
  <h3>Sadece Menüye Bakıp Çıkanlar</h3>
  <p>
    Sırf kahvaltıyı veya akşam yemeğini öğrenmek isteyen birinin hiçbir kişisel verisini
    toplamıyorum.
  </p>
  <p>
    Sadece seni doğrudan bulunduğun şehrin menüsüne yönlendirebilmek için IP adresinden yaklaşık
    şehir tespitini anlık olarak yapıyorum. Bu bilginin kaydı tutulmuyor ve cihazına takip çerezi
    bırakılmıyor.
  </p>
</section>

<section>
  <h3>Hesap Açanlar ve İçerik Üretenler</h3>
  <p>
    Yemekleri oylamak, yorum yazmak veya fotoğraf yüklemek istersen bir hesap gerekiyor. Bu durumda
    senden e-posta adresi veya telefon numarası istemiyorum.
  </p>
  <p>
    Yalnızca kendi belirleyeceğin bir rumuz ve şifreyle hesap açabilirsin. Kayıt olduğunda sistem
    sana tek seferlik rastgele bir kurtarma kodu üretir. Şifreni unutursan hesabını bu kodla
    kurtarabilirsin. Veri tabanında yalnızca şifrenin ve kurtarma kodunun kriptografik özetleri
    saklanır. Bu kodlar saf matematiksel entropi olduğundan hiçbir gerçek kişiyle eşleştirilemez ve
    tamamen anonimdir.
  </p>
  <p>
    Hesabının onaylanması ve yorum/oy özelliklerini kullanabilmen için bu kodu güvenli bir yere
    kaydettiğine dair teyit vermen istenir. Benzer şekilde, önceki sistemden kalan mevcut hesaplar
    da sisteme girdiklerinde kurtarma kodlarını oluşturup kaydettiklerini beyan ederek hesaplarını
    güvene alırlar.
  </p>
</section>

<section>
  <h2>1. Veri Sorumlusu</h2>
  <p>
    6698 sayılı Kişisel Verilerin Korunması Kanunu uyarınca bu projenin yasal muhatabı ve veri
    sorumlusu, Kepçe projesini bireysel olarak geliştiren Kazım Geleş'tir. Resmi tebligat,
    Uyar-Kaldır bildirimleri veya hak taleplerin için <a href="/iletisim" data-link
      >iletişim sayfası</a
    >
    üzerinden ya da doğrudan
    <a href="mailto:yasal@kepce.org">yasal@kepce.org</a> adresinden bana ulaşabilirsin.
  </p>
</section>

<section>
  <h2>2. İşlenen Veriler ve Toplanma Yöntemi</h2>
  <p>Platformun çalışabilmesi için tamamen otomatik yollarla asgari düzeyde teknik veri işlenir.</p>
  <ul>
    <li>
      Sitede yalnızca kamuya açık yemek listelerini inceleyen <strong>menü okuyucuları</strong> için hiçbir
      kimlik veya trafik verisi kaydedilmez, sunucuda erişim kaydı tutulmaz ve yurt dışında kalıcı olarak
      hiçbir veri barındırılmaz veya üçüncü taraflara aktarılmaz.
    </li>
    <li>
      Sisteme kaydolan <strong>hesap sahipleri</strong> için e-posta, telefon veya kimlik bilgisi istenmez
      ve tutulmaz. Veri tabanında yalnızca anonim rumuz ile şifre ve kurtarma kodunun özetleri saklanır.
    </li>
    <li>
      Yorumlara eklenen <strong>yemek fotoğrafları</strong> üzerindeki GPS konum verileri ile cihaz ayrıntıları
      sunucuya ulaştığı anda temizlenir ve yalnızca tabldot görseli yayında kalır. Arşive katkı amacıyla
      gönderilen menü panosu fotoğrafları ise listeler metne döküldükten sonra kalıcı olarak silinir.
    </li>
    <li>
      5651 sayılı Kanun m. 5/3 gereğince yorum yazan veya görsel yükleyen kullanıcılara ait işlem
      zamanı, IP adresi ve port bilgisi gibi <strong>yasal trafik kayıtları</strong> tutulmak zorundadır.
      Bu kayıtlar Fransa'daki sunucu diskine yazılmaz. Bellekte üretildiği salise asimetrik şifreleme
      ile mühürlenerek doğrudan Türkiye'deki yerel arşive aktarılır ve Fransa belleğinden silinir.
    </li>
    <li>
      Yemeklere ve yorumlara bırakılan <strong>oylar ve tepkiler</strong> mükerrer kullanımı engellemek
      için veri tabanında anonim kullanıcı koduyla tekil olarak eşleştirilir. Tablolarda IP adresi tutulmaz
      ve aynı butona tekrar basıldığında verilen oy geri alınır.
    </li>
  </ul>
</section>

<section>
  <h2>3. İşleme Amaçları</h2>
  <p>
    Sınırlı düzeydeki teknik veriler, 6698 sayılı Kanun'un 5. ve 6. maddeleri kapsamındaki hukuki
    sebeplere dayanılarak işlenir.
  </p>
  <ul>
    <li>
      Kanunlarda açıkça öngörülen <strong>hukuki yükümlülükler</strong> gereğince 5651 sayılı Kanun m.
      5/3 uyarınca yalnızca içerik üreten kullanıcıların yasal trafik kayıtları tutulur ve yetkili adli
      mercilerin usulüne uygun talepleri karşılanır.
    </li>
    <li>
      Kullanım Koşulları çerçevesinde hesabın açılması, oturumun doğrulanması ve paylaşım
      özelliklerinin sunulabilmesi için <strong>sözleşmenin ifası</strong>
      esastır.
    </li>
    <li>
      Temel hak ve özgürlüklere zarar vermemek kaydıyla sitenin kararlı çalışması, bot akınlarının
      önlenmesi, istek sınırlandırması uygulanması ve oylama manipülasyonunun engellenmesi amacıyla <strong
        >meşru menfaat</strong
      > kapsamında güvenlik önlemleri işletilir.
    </li>
  </ul>
  <p>
    Sağlık durumu, gıda alerjisi, çölyak veya inanç tercihleri kesinlikle toplanmaz ve işlenmez.
    Sitedeki alerjen ve diyet filtreleri yalnızca kendi tarayıcında yerel olarak çalışır, sunucuya
    gönderilmez. Yorum alanlarında kendine veya başkalarına ait sağlık verilerini paylaşmaman
    gerekir.
  </p>
</section>

<section>
  <h2>4. Verileri Saklama</h2>
  <p>
    5651 sayılı Kanun gereği tutulması zorunlu olan trafik kayıtları, Fransa sunucusunun belleğinde
    oluştuğu anda açık anahtarlı şifreleme ile mühürlenir. Fransa'daki sunucu diskine hiçbir zaman
    trafik logu yazılmaz. Şifrelenmiş paketler doğrudan Türkiye sınırları içindeki yerel arşive
    aktarılır ve Fransa'daki bellekten derhal silinir. O sandığı açabilecek özel anahtar da
    Türkiye'de, internete kapalı fiziksel bir ortamda saklanır.
  </p>
  <p>
    Bu mimari sayesinde Fransa'daki sunucu sağlayıcısı dahil hiç kimse bu şifreli kayıtları açıp
    okuyamaz ve Fransa'da hiçbir kişisel veri depolanmaz. Kayıtlar yalnızca Türkiye Cumhuriyeti
    mahkemeleri veya cumhuriyet savcılıkları tarafından iletilen usulüne uygun resmi bir adli talep
    olduğunda Türkiye'deki anahtarla çözülerek yetkili makamlara sunulur.
  </p>
  <p>
    Şifreli trafik kayıtları yasal süresi olan en az bir ve en fazla iki yılın sonunda anahtara bile
    gerek kalmadan kalıcı olarak imha edilir. Kendi paylaştığın yorumları, fotoğrafları veya
    hesabını profil ayarlarından dilediğin an tek dokunuşla silebilirsin. Sildiğin içerikler siteden
    anında kaldırılır.
  </p>
</section>

<section>
  <h2>5. Çerezler ve Güvenlik Araçları</h2>
  <p>
    Sitenin temel işlevlerini yürütebilmek için tarayıcında ve ağ katmanında yalnızca zorunlu teknik
    araçlar kullanılır.
  </p>
  <ul>
    <li>
      Giriş yaptığında oturumunun açık kalabilmesi için yalnızca takip veya reklam amacı taşımayan
      teknik bir <strong>oturum çerezi</strong> kullanılır.
    </li>
    <li>
      Seçtiğin şehir, açık veya koyu tema modu, bot görünürlüğü ve alerjen filtreleri gibi <strong
        >yerel tercihler</strong
      > kimliğinle eşleştirilmeden yalnızca kendi tarayıcının yerel hafızasında tutulur.
    </li>
    <li>
      Hangi menülerin ne kadar ziyaret edildiğini anlamak için çerez bırakmayan, IP kaydetmeyen ve
      ziyaretçi davranışı izlemeyen açık kaynaklı Umami <strong>anonim sayacı</strong> çalışır.
    </li>
    <li>
      Sunucuları kötü niyetli botlardan ve aşırı istek saldırılarından korumak amacıyla Cloudflare <strong
        >ağ güvenliği</strong
      > katmanından yararlanılır.
    </li>
  </ul>
</section>

<section>
  <h2>6. Yasal Hakların</h2>
  <p>6698 sayılı Kanun madde 11 uyarınca veri sahibi olarak şu haklara sahipsin:</p>
  <ul>
    <li>Kişisel verilerinin işlenip işlenmediğini öğrenme ve işlenmişse bilgi talep etme,</li>
    <li>İşlenme amacını ve bunların amacına uygun kullanılıp kullanılmadığını öğrenme,</li>
    <li>Yurt içinde veya yurt dışında verilerin aktarıldığı üçüncü kişileri bilme,</li>
    <li>Eksik veya yanlış işlenmiş olması hâlinde bunların düzeltilmesini isteme,</li>
    <li>
      Kanun'un 7. maddesinde öngörülen şartlar çerçevesinde silinmesini veya yok edilmesini isteme
      ve bu işlemlerin verilerin aktarıldığı üçüncü kişilere bildirilmesini talep etme,
    </li>
    <li>
      İşlenen verilerin münhasıran otomatik sistemler vasıtasıyla analiz edilmesi suretiyle aleyhine
      bir sonucun ortaya çıkmasına itiraz etme. Kepçe bünyesinde kullanıcılar hakkında davranışsal
      profil çıkarma veya otomatik karar alma mekanizması işletilmez.
    </li>
    <li>
      Kanuna aykırı olarak işlenmesi sebebiyle zarara uğraman hâlinde zararın giderilmesini talep
      etme.
    </li>
  </ul>
  <p>
    Bu haklarınla ilgili taleplerini veya Uyar-Kaldır bildirimlerini <a
      href="mailto:yasal@kepce.org">yasal@kepce.org</a
    > adresine iletebilirsin. Başvuruları en kısa sürede ve kanuni süre olan otuz günü aşmadan ücretsiz
    olarak yanıtlarım.
  </p>
</section>
