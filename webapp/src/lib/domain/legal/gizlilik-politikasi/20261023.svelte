<p>
  Kepçe'yi tasarlarken temel ilkemiz <strong>"veri minimizasyonu ve sıfır bilgi"</strong>
  oldu: İhtiyacımız olmayan hiçbir kimlik verisini (açık e-posta adresi, gerçek ad-soyad, telefon numarası
  vb.) veri tabanımıza kaydetmiyoruz. Bu metin, teknik gizlilik mimarimizi ve 6698 sayılı Kişisel
  Verilerin Korunması Kanunu (KVKK) kapsamındaki aydınlatma bildirimimizi içerir.
</p>

<section>
  <h2>1. Veri Sorumlusu ve Tanıtıcı Bilgiler</h2>
  <p>
    5651 sayılı Kanun m. 3 uyarınca tanıtıcı bilgi ve KVKK kapsamında muhatap, <strong>Kazım Geleş</strong>
    tarafından bireysel ve kâr amacı gütmeyen açık kaynak projesi olarak geliştirilen
    <strong>kepce.org</strong> platformudur. Yasal bildirimler ve hak talepleri için
    <a href="/iletisim" data-link>iletişim sayfamız</a> veya
    <a href="mailto:yasal@kepce.org">yasal@kepce.org</a> e-posta adresimiz üzerinden bizimle bağlantı
    kurabilirsin.
  </p>
</section>

<section>
  <h2>2. Hangi Özellik İçin Hangi Teknik Süreci İşletiyoruz?</h2>
  <p>
    Kepçe'de veri satışı, davranışsal reklam hedeflemesi veya kullanıcı profilleme faaliyetleri yürütülmez.
    Özellikleri sunarken açık kişisel veri tutmamak adına şu teknik mimariyi uyguluyoruz:
  </p>
  <ul>
    <li>
      <strong>Şehir Yönlendirmesi (Anonim Ziyaretçiler):</strong> Seni doğrudan bulunduğun şehrin
      menüsüne yönlendirmek için IP adresinden yaklaşık şehir bilgini sunucu belleğinde (RAM) anlık
      olarak tespit ederiz. Bu bilgi diske yazılmaz ve loglanmaz. Kamuya açık menüleri okuyan anonim
      ziyaretçilerden hiçbir kişisel veri toplanmaz ve erişim kaydı tutulmaz.
    </li>
    <li>
      <strong>Sıfır Bilgi Hesap, Google Girişi ve Şifre Sıfırlama:</strong> Oylama yapmak,
      yorum yazmak veya fotoğraf paylaşmak için Google ile tek tıkla ya da e-posta doğrulamasıyla giriş
      yapabilirsin. Ancak Kepçe; <strong>e-posta adresini, gerçek adını, soyadını, Google kimliğini veya
      profil fotoğrafını veri tabanına açık metin olarak kaydetmez</strong>. E-posta adresin ve Google
      tanımlayıcın, sunucu belleğinde tek yönlü kriptografik özetleme (HMAC-SHA256 Blind Index) işleminden
      geçirilerek geri döndürülemez bir doğrulama koduna dönüştürülür. Doğrulama bağlantısı veya şifre
      sıfırlama kodu talep ettiğinde, girdiğin e-posta adresi yalnızca o anki doğrulama iletisini
      göndermek için bellekte işlenir ve işlem tamamlandığında RAM'den silinir. Sitede yalnızca kendi
      seçtiğin anonim rumuz görünür.
    </li>
    <li>
      <strong>Oylama ve Reaksiyonlar:</strong> Yemek oylamalarında IP adresi diske kaydedilmez. Mükerrer
      oyları engellemek için yalnızca sunucu belleğinde tutulan ve her gece otomatik olarak sıfırlanan
      geçici kriptografik özetler kullanılır.
    </li>
    <li>
      <strong>Yorum ve Fotoğraf Paylaşımı (5651 Trafik Kayıtları):</strong> Sitede yorum yazdığında veya
      fotoğraf yüklediğinde, 5651 sayılı Kanun m. 2/1-j ve m. 5/3 uyarınca yalnızca içerik üretilen bu
      işlemlere münhasır olmak üzere kanuni trafik bilgilerini (IP adresi, port bilgisi, işlem zamanı,
      hizmet türü ve aktarılan veri miktarı) tutmakla yükümlüyüz. Kanunun m. 2/1-j hükmündeki
      <em>"varsa abone kimlik bilgileri"</em> ibaresi doğrultusunda sistemimizde gerçek kimlik veya açık
      e-posta tutulmadığından, bu kayıtlar yalnızca anonim kullanıcı UUID'si ile eşleştirilir.
    </li>
    <li>
      <strong>Fotoğraflarda EXIF ve GPS Temizliği:</strong> Yorumlara yemek fotoğrafı veya sisteme menü
      panosu görseli yüklediğinde, dosyanın içindeki tüm gizli meta verileri (EXIF, GPS konum
      koordinatları, cihaz bilgisi) sunucu belleğinde otomatik olarak sileriz. Menü doğrulama amacıyla
      gönderilen pano fotoğrafları sitede yayımlanmaz; yemek isimleri metne aktarıldıktan sonra görsel
      dosyası imha edilir.
    </li>
  </ul>
</section>

<section>
  <h2>3. Hukuki Sebepler ve Kanuni İstisnalar</h2>
  <p>
    Sınırlı düzeydeki teknik veriler, KVKK m. 5 ve m. 28'de belirtilen şu hukuki dayanaklarla işlenir:
  </p>
  <ul>
    <li>
      <strong>Kanunlarda açıkça öngörülmesi ve hukuki yükümlülük (KVKK m. 5/2-a ve m. 5/2-ç):</strong>
      5651 sayılı Kanun uyarınca yalnızca içerik üreten kullanıcılara ait yasal trafik bilgilerini tutma
      zorunluluğumuz ve yetkili adli mercilerden gelebilecek resmî müzekkerelerin karşılanması.
    </li>
    <li>
      <strong>Suç işlenmesinin önlenmesi istisnası (KVKK m. 28/2-a):</strong> 5651 sayılı Kanun uyarınca
      suç işlenmesinin önlenmesi veya suç soruşturması için tutulması zorunlu olan şifreli trafik kayıtları
      bakımından KVKK m. 28/2-a kapsamındaki istisna hükümleri uygulanır.
    </li>
    <li>
      <strong>Hakkın tesisi ve meşru menfaat (KVKK m. 5/2-e ve m. 5/2-f):</strong> Temel hak ve
      özgürlüklere zarar vermemek kaydıyla platform güvenliğini sağlamak, bot saldırılarını ve oylama
      manipülasyonunu önlemek.
    </li>
    <li>
      <strong>Özel nitelikli kişisel verilerin işlenmemesi (KVKK m. 6):</strong> Kepçe; sağlık, alerji,
      çölyak, dini inanç veya özel diyet tercihlerine ilişkin sunucu tarafında hiçbir veri toplamaz.
      Alerjen ve diyet filtreleri yalnızca kullanıcının kendi tarayıcısında (<code>localStorage</code>) çalışır.
      Kullanıcıların yorumlarda kendilerine veya üçüncü kişilere ait sağlık verisi paylaşmaması
      zorunludur.
    </li>
  </ul>
</section>

<section>
  <h2>4. Altyapı Güvenliği, Sıfır Bilgi Şifrelemesi ve Veri Paylaşımı</h2>
  <p>
    Kepçe'de hiçbir veri üçüncü şahıslara, veri simsarlarına veya reklamcılara satılmaz ve aktarılmaz.
  </p>
  <p>
    Veri tabanımızda gerçek ad, soyadı, açık e-posta adresi veya harici profil kimliği tutulmadığından,
    sunucu altyapımıza aktarılan herhangi bir açık kimlik verisi bulunmamaktadır. Teknik doğrulama ve
    iletişim iletilerinin gönderimi sırasında üçüncü parti e-posta iletim sağlayıcısının (SMTP) zorunlu
    geçici kuyruk ve teslimat günlükleri hariç tutulur.
  </p>
  <p>
    5651 sayılı Kanun uyarınca içerik üretilen işlemlerde tutulması zorunlu olan trafik kayıtları ise
    veri tabanında açık metin (plaintext) olarak <strong>tutulmaz</strong>; sunucu belleğinde anlık
    olarak asimetrik açık anahtar algoritmasıyla (X25519 SealedBox) şifrelenir. Bu şifreyi çözebilecek
    özel anahtar (Private Key) sunucuda bulunmaz; yalnızca Türkiye Cumhuriyeti sınırları içinde
    çevrimdışı olarak muhafaza edilir. Sunucu sağlayıcısı dahil hiçbir üçüncü tarafın bu kayıtlara
    erişimi teknik olarak mümkün değildir. Kayıtlar yalnızca Türkiye Cumhuriyeti mahkemeleri veya
    Cumhuriyet başsavcılıkları tarafından usulüne uygun resmî bir talep geldiğinde çözülerek ilgili
    adli makamla paylaşılır.
  </p>
</section>

<section>
  <h2>5. Saklama Süreleri ve İmha Politikası</h2>
  <ul>
    <li>
      <strong>Anonim ziyaretçiler:</strong> Kamuya açık menüleri okuyan ziyaretçilere ait hiçbir trafik
      kaydı oluşturulmaz ve saklanmaz.
    </li>
    <li>
      <strong>Zorunlu 5651 trafik kayıtları:</strong> İçerik üretilen işlemlere ait şifreli trafik
      kayıtları, 5651 sayılı Kanun m. 5/3 uyarınca <strong>bir yıldan az ve iki yıldan fazla olmamak
      üzere</strong> saklanır ve sürenin sonunda otomatik olarak kalıcı biçimde yok edilir.
    </li>
    <li>
      <strong>Menü panosu fotoğrafları:</strong> Menü listesini doğrulamak için gönderilen pano
      fotoğrafları, liste metne aktarıldığı anda veya incelenmemesi hâlinde en geç otuz gün içinde
      otomatik olarak silinir.
    </li>
    <li>
      <strong>Hesap ve içerik silme:</strong> Hesabını veya içeriğini sildiğinde rumuzun, kriptografik
      oturum özetin ve paylaştığın içerik kamuya açık sistemden derhal kaldırılır.
    </li>
  </ul>
</section>

<section>
  <h2>6. Hesabını ve İçeriklerini Nasıl Silebilirsin?</h2>
  <ul>
    <li>
      <strong>Profil ve yorum ayarlarından silme:</strong> Giriş yapmış durumdayken kendi yazdığın
      yorumları, yüklediğin fotoğrafları veya profil ayarlarından hesabını tek dokunuşla kalıcı olarak
      silebilirsin.
    </li>
    <li>
      <strong>İletişim kanalıyla talep etme:</strong> Hesabına erişemiyorsan veya herhangi bir içerikle
      ilgili silme talebin varsa <a href="mailto:yasal@kepce.org">yasal@kepce.org</a> adresine yazarak
      işlem yapılmasını talep edebilirsin.
    </li>
  </ul>
</section>

<section>
  <h2>7. Çerezler, Yerel Depolama ve Güvenlik Servisleri</h2>
  <ul>
    <li>
      <strong>Oturum çerezi:</strong> Giriş yaptığında oturumunu doğrulamak için kullanılan teknik çerez,
      tarayıcı betiklerinin erişemediği <code>HttpOnly</code> ve <code>SameSite</code> korumalıdır; takip amacıyla kullanılmaz.
    </li>
    <li>
      <strong>Yerel tercih verileri (LocalStorage):</strong> Seçtiğin şehir, tema modu, maskelenmiş
      hesap ipucu ve alerjen/diyet filtrelerin yalnızca kendi tarayıcının yerel hafızasında tutulur;
      sunucuya gönderilmez.
    </li>
    <li>
      <strong>Umami:</strong> Çerez kullanmayan, IP kaydetmeyen ve kişisel veri toplamayan açık kaynaklı
      anonim sayfa sayacıdır.
    </li>
    <li>
      <strong>Cloudflare ve Turnstile:</strong> DDoS saldırılarını ve otomatik botları filtrelemek için
      kullanılan ağ güvenlik katmanıdır.
    </li>
  </ul>
</section>

<section>
  <h2>8. Kanuni Hakların ve Başvuru Usulü</h2>
  <p>KVKK m. 11 uyarınca herkes veri sorumlusuna başvurarak kendisiyle ilgili:</p>
  <ul>
    <li>Kişisel veri işlenip işlenmediğini öğrenme ve işlenmişse bilgi talep etme,</li>
    <li>İşlenme amacını ve amacına uygun kullanılıp kullanılmadığını öğrenme,</li>
    <li>Yurt içinde veya yurt dışında verilerin aktarıldığı üçüncü kişileri bilme,</li>
    <li>Eksik veya yanlış işlenmişse düzeltilmesini isteme,</li>
    <li>KVKK m. 7 şartları çerçevesinde silinmesini veya yok edilmesini isteme,</li>
    <li>Düzeltme ve silme işlemlerinin aktarıldığı üçüncü kişilere bildirilmesini isteme,</li>
    <li>Münhasıran otomatik sistemlerle analiz edilmesi suretiyle aleyhine bir sonucun ortaya çıkmasına itiraz etme,</li>
    <li>Kanuna aykırı işleme sebebiyle zarara uğrama durumunda zararın giderilmesini talep etme haklarına sahiptir.</li>
  </ul>
  <p>
    KVKK m. 13 uyarınca taleplerini <a href="mailto:yasal@kepce.org">yasal@kepce.org</a> adresine veya
    <a href="/iletisim" data-link>iletişim sayfamızdaki</a> kanallara iletebilirsin. Başvurular en kısa
    sürede ve en geç otuz gün içinde ücretsiz olarak sonuçlandırılır.
  </p>
</section>
