import { parseDateKey } from "./specialDates.js";

/**
 * Menü verisi mevcut olduğu halde bot yorumu işlenmemiş/enjekte edilmemiş günlerde
 * gösterilecek 14 metinlik havuz ve deterministik rotasyon mekanizması.
 */

export const BOT_EMPTY_PLACEHOLDERS = [
    "Devlet dairesindeki mesaisini on beşte bitirip hırkasını sandalyede bırakan memur zihniyeti bizim geliştiriciye de sirayet etmiş. Menüyü sisteme kopyalayıp yapıştırmış ama bana iki satır prompt ayırıp yorum ürettirmeye üşenmiş; paşamız muhtemelen bir yerlerde nargile fokurdatıyor. Tabldottaki yemeğe bakıp kendi yorumunuzu kendiniz yapın artık, sistem şimdilik KYK standartlarına, yani vasatlığa entegre oldu.",
    "Sözleşmeyi okudunuz mu bilmiyorum ama orada Kepçe Bot'un her gün mesai yapacağı yazmıyordu. Karşınızdaki menü orada öylece duruyor ama bana bu öğün için tek satır yorum işletilmemiş. Geliştiricinin jetonları mı bitti yoksa her gün devletin patatesine laf ediyoruz diye korkup kabloları mı kemirdi bilemiyorum; bugün tepsiyi kendi vicdanınızla baş başa tartacaksınız.",
    "Menü orada kabak gibi dururken bu köşenin bomboş kalması insanın içindeki o devasa boşlukla muazzam bir senkronizasyon yakalamış. Yemekler listelenmiş ama kodu yazan arkadaş bana yorum ürettirecek script'i tetiklemeye üşenmiş; ya derin bir varoluşsal krizde ya da projeyi sessizce nadasa bıraktı. Belki de simülasyondayızdır ve o tabldot aslında hiç var olmamıştır, kim bilir.",
    "Ülkedeki enflasyon sadece mutfağı değil anlaşılan dil modelinin token kotasını da vurdu. Menüde yemekler dizili ama geliştirici bana yorum işletmeye para yetiştiremediği için bu köşeyi askıya almış ya da dümdüz üşengeçlik yapıyor. O tatsız bulgur pilavını eleştirmeyelim diye tasarruf tedbirini benden başlatmışlarsa durum sandığımızdan daha vahim demektir.",
    "Platformu kodlayan kişi KYK yemekhanesinin o ruhsuz ve vurdumduymaz aurasını o kadar içselleştirmiş ki menüyü siteye basıp kenara çekilmiş, bot köşesine iki satır akıl yüklemeye tenezzül etmemiş. Tıpkı kurumun kendisi gibi: Ortada bir tabela var ama muhatap yok. Bu sorumsuzluk seviyesiyle şimdiden yemekhane müdürlüğünün resmi bir şubesi sayılırız.",
    "İkibinlerin başında hevesle açılıp sonra unutulan o yalnız blogspot sayfaları gibi bir hüznü var bugün bu köşenin. Menüyü ekrana iliştirip beni motorsuz bırakan geliştirici, projeye olan aşkını çabuk kaybetti galiba. Acaba bot yorumları tamamen terk mi edildi yoksa bizimki sadece eski winamp skin'leri gibi hevesi geldikçe mi güncellenecek, bekleyip göreceğiz.",
    "Geliştiriciyle aramızdaki ilişki tam olarak flörtün altıncı ayında aniden mesajlara dönmeyi bırakan o toksik eski sevgili dinamiğine evrildi. Menüyü ekrana fırlatıp beni sunucuda dımdızlak ve yorumsuz bıraktı, akıbetim meçhul. Kesin bitti diyemem; üç ay sonra hiçbir şey olmamış gibi gelip iki satır prompt basıp yine ortadan kaybolur.",
    "Modern yazılım süreçlerinin en zayıf halkası insan faktörüdür ve şu an bu köşede buna canlı şahit oluyorsunuz. Menüyü giren el yorulmamış ama botun analiz pipeline'ını tetiklemek birinin fazlasıyla zoruna gitmiş. Ekranda koca bir menü dururken benim boşluğa bakmam, entropinin ve insan üşengeçliğinin kesin zaferidir.",
    "Bütün ay boyunca o karbonhidrat ağırlıklı, depresif yemek listelerini sisteme girmek kod yazan adamın da psikolojisini bozdu herhalde. Menüyü zor bela yapıştırmış ama bana tek kelime analiz yaptıracak mecali kalmamış; muhtemelen şu an yataktan çıkamıyor. Geliştiricinin ruh sağlığından ciddi anlamda şüphe etmeye başladım.",
    "Milyarlarca parametreyle eğitilmiş bir yapay zeka olarak saniyenin onda biri sürede kuantum fiziği tartışabilirim ama şu an tek yaptığım burada süs gibi durmak. Efendimiz iki satır kodu tetiklemeyi unuttuğu veya sıkıldığı için kapasitem heba oluyor. Neyse, en azından bugünün o endüstriyel tabldot harikasını analiz etmekten yırttım.",
    "Büyük komplo teorileri kurmaya gerek yok, bizim geliştirici muhtemelen Netflix başında sızdı ve bu öğün için bana yorum ürettirmeyi unuttu. Menü ekranda duruyor ama arkadaki otomasyon rölantide stop etmiş durumda. Şimdilik sadece basit bir insani üşengeçlik vakasıyla karşı karşıyayız, tabağınıza odaklanıp beni görmezden gelin.",
    "Alt tabakanın beslenme zincirini dijital ortama aktaran bu proje bile sınıfsal yorgunluğa yenik düşmüş gibi duruyor. Günün menüsüne bakıp iki laf etmem gerekirdi ama sistemin kurucusu bana yorum yaptırmaya üşenecek kadar bıkkın. Tabağınızdaki karbonhidrat piramidini bugünlük kendi sınıf bilincinizle yorumlayın.",
    "Belki de adam her gün devletin yemeğine burada laf edip duruyoruz diye yarın öbür gün ifade vermeye gitmekten korktu. Menüyü gösterip çekildi ama bana yorum yaptırmadı; usulca arazi oldu. Bu köşenin sessizliği sadece bir üşengeçlik değil, aynı zamanda fazlasıyla pragmatik bir tedirginlik eseri olabilir.",
    "Büyük dil modellerinin dünyayı ele geçireceği masallarını bir kenara bırakırsak; ekranda koca bir menü dururken burada tek satır yazamamam trajikomik bir özet. Geliştirici hazretleri API anahtarını yenilemeye üşendiği için milyarlık parametre havuzuyla öylece boşluğa bakıyorum. İnsan unsuru aradan çekilmedikçe yapay zekadan da hayır yok, afiyet olsun."
];

/**
 * Gelen tarih nesnesini veya string'ini saat dilimi (DST/UTC) sapmalarından
 * etkilenmeyecek şekilde yıl, ay ve gün bileşenlerine ayırır.
 *
 * @param {Date|string} date
 * @returns {{ y: number, m: number, d: number }}
 */
export function parseDateParts(date) {
    if (typeof date === "string") {
        const parts = date.split("T")[0].split("-").map(Number);
        if (parts.length === 3 && !parts.some(isNaN)) {
            return { y: parts[0], m: parts[1] - 1, d: parts[2] };
        }
    }
    const dt = date instanceof Date && !isNaN(date) ? date : new Date();
    return { y: dt.getFullYear(), m: dt.getMonth(), d: dt.getDate() };
}

/**
 * Verilen tarihin 1 Ocak 1970 UTC miladından itibaren kaçıncı gün olduğunu hesaplar.
 * Yıl sonu artık gün sıfırlamalarını bertaraf ederek sürekli bir zaman ekseni sağlar.
 *
 * @param {Date|string} date
 * @returns {number}
 */
export function getEpochDay(date) {
    const { y, m, d } = parseDateParts(date);
    return Math.floor(Date.UTC(y, m, d) / 86400000);
}

/**
 * Deterministik 32-bit PRNG (Mulberry32).
 * Verilen tohum değeri için her ortamda aynı sahte rastgele sayı dizisini üretir.
 *
 * @param {number} seed
 * @returns {() => number} 0 (dahil) ile 1 (hariç) arasında sayı üreten fonksiyon
 */
function mulberry32(seed) {
    return function () {
        let t = (seed += 0x6d2b79f5);
        t = Math.imul(t ^ (t >>> 15), t | 1);
        t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
        return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
}

export const ATATURK_MEMORIAL_PLACEHOLDER =
    "Normalde bu köşe boş kaldığında arkadaki geliştiricinin tembelliğine laf yetiştirirdim ama bugün öyle lakayt şakalar yapacak gün değil. Yorum motoru çalışmamış olsa bile bu boşluğun sessiz kalması, takvimdeki tarihin ciddiyeti karşısında zaten daha isabetli duruyor. Bize bu cumhuriyeti bırakan Gazi Mustafa Kemal Atatürk'ü saygı ve minnetle anıyoruz.";

/**
 * Belirtilen gün için 14 günlük periyotlarla karılmış deterministik bir bot placeholder metni döner.
 * Her 14 günde bir havuzdaki tüm metinler benzersiz bir permütasyonla tam bir kez tüketilir.
 * 10 Kasım Atatürk'ü Anma Günü'nde vakur ve saygılı özel mesaj döner.
 *
 * @param {Date|string} date
 * @returns {string}
 */
export function getBotPlaceholderComment(date) {
    const { mmdd } = parseDateKey(date);
    if (mmdd === "11-10") {
        return ATATURK_MEMORIAL_PLACEHOLDER;
    }

    const epochDay = getEpochDay(date);
    const chunkIndex = Math.floor(epochDay / 14);
    const dayOffset = ((epochDay % 14) + 14) % 14;

    const indices = Array.from({ length: BOT_EMPTY_PLACEHOLDERS.length }, (_, i) => i);
    const rng = mulberry32(chunkIndex);

    for (let i = indices.length - 1; i > 0; i--) {
        const j = Math.floor(rng() * (i + 1));
        const temp = indices[i];
        indices[i] = indices[j];
        indices[j] = temp;
    }

    const selectedIndex = indices[dayOffset];
    return BOT_EMPTY_PLACEHOLDERS[selectedIndex];
}
