import { sanitizeText } from './sanitize.js';

export const BACK_VOWELS = new Set([...'aıouAIOU']);
export const FRONT_VOWELS = new Set([...'eiöüEİÖÜ']);
export const VOICELESS = new Set([...'çfhkpsştÇFHKPSŞT']);

export const CITY_MAP = {
  adana: "Adana", adiyaman: "Adıyaman", afyonkarahisar: "Afyonkarahisar", agri: "Ağrı", amasya: "Amasya",
  ankara: "Ankara", antalya: "Antalya", artvin: "Artvin", aydin: "Aydın", balikesir: "Balıkesir",
  bilecik: "Bilecik", bingol: "Bingöl", bitlis: "Bitlis", bolu: "Bolu", burdur: "Burdur",
  bursa: "Bursa", canakkale: "Çanakkale", cankiri: "Çankırı", corum: "Çorum", denizli: "Denizli",
  diyarbakir: "Diyarbakır", edirne: "Edirne", elazig: "Elazığ", erzincan: "Erzincan", erzurum: "Erzurum",
  eskisehir: "Eskişehir", gaziantep: "Gaziantep", giresun: "Giresun", gumushane: "Gümüşhane", hakkari: "Hakkari",
  hatay: "Hatay", isparta: "Isparta", mersin: "Mersin", istanbul: "İstanbul", izmir: "İzmir",
  kars: "Kars", kastamonu: "Kastamonu", kayseri: "Kayseri", kirklareli: "Kırklareli", kirsehir: "Kırşehir",
  kocaeli: "Kocaeli", konya: "Konya", kutahya: "Kütahya", malatya: "Malatya", manisa: "Manisa",
  kahramanmaras: "Kahramanmaraş", mardin: "Mardin", mugla: "Muğla", mus: "Muş", nevsehir: "Nevşehir",
  nigde: "Niğde", ordu: "Ordu", rize: "Rize", sakarya: "Sakarya", samsun: "Samsun",
  siirt: "Siirt", sinop: "Sinop", sivas: "Sivas", tekirdag: "Tekirdağ", tokat: "Tokat",
  trabzon: "Trabzon", tunceli: "Tunceli", sanliurfa: "Şanlıurfa", usak: "Uşak", van: "Van",
  yozgat: "Yozgat", zonguldak: "Zonguldak", aksaray: "Aksaray", bayburt: "Bayburt", karaman: "Karaman",
  kirikkale: "Kırıkkale", batman: "Batman", sirnak: "Şırnak", bartin: "Bartın", ardahan: "Ardahan",
  igdir: "Iğdır", yalova: "Yalova", karabuk: "Karabük", kilis: "Kilis", osmaniye: "Osmaniye",
  duzce: "Düzce",
};

/**
 * Türkiye 81 il merkezi ve popüler ilçelerin statik koordinat haritası.
 * Nominatim dış API limitlerini ve ban riskini önlemek için 0ms yerel lookup sağlar.
 */
export const TURKEY_GEO_MAP = {
  adana: { lat: 37.0, lon: 35.3213, name: "Adana" },
  adiyaman: { lat: 37.7648, lon: 38.2786, name: "Adıyaman" },
  afyonkarahisar: { lat: 38.7507, lon: 30.5567, name: "Afyonkarahisar" },
  agri: { lat: 39.7191, lon: 43.0503, name: "Ağrı" },
  amasya: { lat: 40.6501, lon: 35.8353, name: "Amasya" },
  ankara: { lat: 39.9334, lon: 32.8597, name: "Ankara" },
  antalya: { lat: 36.8969, lon: 30.7133, name: "Antalya" },
  artvin: { lat: 41.1828, lon: 41.8183, name: "Artvin" },
  aydin: { lat: 37.856, lon: 27.8416, name: "Aydın" },
  balikesir: { lat: 39.6484, lon: 27.8826, name: "Balıkesir" },
  bilecik: { lat: 40.1451, lon: 29.9799, name: "Bilecik" },
  bingol: { lat: 38.8854, lon: 40.4983, name: "Bingöl" },
  bitlis: { lat: 38.4006, lon: 42.1095, name: "Bitlis" },
  bolu: { lat: 40.735, lon: 31.6061, name: "Bolu" },
  burdur: { lat: 37.7203, lon: 30.2908, name: "Burdur" },
  bursa: { lat: 40.1885, lon: 29.061, name: "Bursa" },
  canakkale: { lat: 40.1553, lon: 26.4142, name: "Çanakkale" },
  cankiri: { lat: 40.6013, lon: 33.6134, name: "Çankırı" },
  corum: { lat: 40.5506, lon: 34.9556, name: "Çorum" },
  denizli: { lat: 37.7765, lon: 29.0864, name: "Denizli" },
  diyarbakir: { lat: 37.9144, lon: 40.2306, name: "Diyarbakır" },
  edirne: { lat: 41.6772, lon: 26.5557, name: "Edirne" },
  elazig: { lat: 38.681, lon: 39.2264, name: "Elazığ" },
  erzincan: { lat: 39.75, lon: 39.5, name: "Erzincan" },
  erzurum: { lat: 39.9043, lon: 41.2679, name: "Erzurum" },
  eskisehir: { lat: 39.7767, lon: 30.5206, name: "Eskişehir" },
  gaziantep: { lat: 37.0662, lon: 37.3833, name: "Gaziantep" },
  giresun: { lat: 40.9128, lon: 38.3895, name: "Giresun" },
  gumushane: { lat: 40.4603, lon: 39.4817, name: "Gümüşhane" },
  hakkari: { lat: 37.5833, lon: 43.7333, name: "Hakkari" },
  hatay: { lat: 36.4018, lon: 36.3498, name: "Hatay" },
  isparta: { lat: 37.7648, lon: 30.5566, name: "Isparta" },
  mersin: { lat: 36.8121, lon: 34.6415, name: "Mersin" },
  istanbul: { lat: 41.0082, lon: 28.9784, name: "İstanbul" },
  izmir: { lat: 38.4192, lon: 27.1287, name: "İzmir" },
  kars: { lat: 40.6013, lon: 43.0975, name: "Kars" },
  kastamonu: { lat: 41.3887, lon: 33.7827, name: "Kastamonu" },
  kayseri: { lat: 38.7312, lon: 35.4787, name: "Kayseri" },
  kirklareli: { lat: 41.7333, lon: 27.2167, name: "Kırklareli" },
  kirsehir: { lat: 39.1425, lon: 34.1709, name: "Kırşehir" },
  kocaeli: { lat: 40.8533, lon: 29.8815, name: "Kocaeli" },
  konya: { lat: 37.8667, lon: 32.4833, name: "Konya" },
  kutahya: { lat: 39.4167, lon: 29.9833, name: "Kütahya" },
  malatya: { lat: 38.3552, lon: 38.3095, name: "Malatya" },
  manisa: { lat: 38.6191, lon: 27.4289, name: "Manisa" },
  kahramanmaras: { lat: 37.5858, lon: 36.9371, name: "Kahramanmaraş" },
  mardin: { lat: 37.3212, lon: 40.7245, name: "Mardin" },
  mugla: { lat: 37.2153, lon: 28.3636, name: "Muğla" },
  mus: { lat: 38.7432, lon: 41.5064, name: "Muş" },
  nevsehir: { lat: 38.6244, lon: 34.7144, name: "Nevşehir" },
  nigde: { lat: 37.9667, lon: 34.6833, name: "Niğde" },
  ordu: { lat: 40.9839, lon: 37.8764, name: "Ordu" },
  rize: { lat: 41.0201, lon: 40.5234, name: "Rize" },
  sakarya: { lat: 40.694, lon: 30.4358, name: "Sakarya" },
  samsun: { lat: 41.2867, lon: 36.33, name: "Samsun" },
  siirt: { lat: 37.9333, lon: 41.95, name: "Siirt" },
  sinop: { lat: 42.0231, lon: 35.1531, name: "Sinop" },
  sivas: { lat: 39.7477, lon: 37.0179, name: "Sivas" },
  tekirdag: { lat: 40.9833, lon: 27.5167, name: "Tekirdağ" },
  tokat: { lat: 40.3167, lon: 36.55, name: "Tokat" },
  trabzon: { lat: 41.0015, lon: 39.7178, name: "Trabzon" },
  tunceli: { lat: 39.1079, lon: 39.5401, name: "Tunceli" },
  sanliurfa: { lat: 37.1591, lon: 38.7969, name: "Şanlıurfa" },
  usak: { lat: 38.6823, lon: 29.4082, name: "Uşak" },
  van: { lat: 38.4891, lon: 43.4089, name: "Van" },
  yozgat: { lat: 39.8181, lon: 34.8147, name: "Yozgat" },
  zonguldak: { lat: 41.4564, lon: 31.7987, name: "Zonguldak" },
  aksaray: { lat: 38.3687, lon: 34.037, name: "Aksaray" },
  bayburt: { lat: 40.2552, lon: 40.2249, name: "Bayburt" },
  karaman: { lat: 37.1759, lon: 33.2287, name: "Karaman" },
  kirikkale: { lat: 39.8468, lon: 33.5153, name: "Kırıkkale" },
  batman: { lat: 37.8812, lon: 41.1294, name: "Batman" },
  sirnak: { lat: 37.5164, lon: 42.4594, name: "Şırnak" },
  bartin: { lat: 41.6344, lon: 32.3375, name: "Bartın" },
  ardahan: { lat: 41.1105, lon: 42.7022, name: "Ardahan" },
  igdir: { lat: 39.9196, lon: 44.0454, name: "Iğdır" },
  yalova: { lat: 40.65, lon: 29.2667, name: "Yalova" },
  karabuk: { lat: 41.2061, lon: 32.6204, name: "Karabük" },
  kilis: { lat: 36.7184, lon: 37.1212, name: "Kilis" },
  osmaniye: { lat: 37.0742, lon: 36.2472, name: "Osmaniye" },
  duzce: { lat: 40.8438, lon: 31.1565, name: "Düzce" },
  alanya: { lat: 36.5438, lon: 31.9998, name: "Alanya" },
  bodrum: { lat: 37.0344, lon: 27.4305, name: "Bodrum" },
  fethiye: { lat: 36.6592, lon: 29.1263, name: "Fethiye" },
  gebze: { lat: 40.8028, lon: 29.4307, name: "Gebze" },
  corlu: { lat: 41.1592, lon: 27.8, name: "Çorlu" },
  iskenderun: { lat: 36.5872, lon: 36.1735, name: "İskenderun" },
  kusadasi: { lat: 37.8579, lon: 27.261, name: "Kuşadası" },
  marmaris: { lat: 36.855, lon: 28.2742, name: "Marmaris" },
};

/**
 * Aktif olarak menü verisi çekilen ve doğrulanmış şehirler listesi.
 * Dinamik sitemap ve SEO indeksleme politikası bu listeyi baz alır.
 */
export const ACTIVE_CITIES = [
  "istanbul", "ankara", "izmir", "antalya", "canakkale", "erzurum", 
  "eskisehir", "gaziantep", "isparta", "kahramanmaras", "karabuk", 
  "kirklareli", "konya", "sakarya", "sivas", "trabzon"
];

/**
 * Returns the correct ablative suffix for a Turkish word.
 * Ayrılma eki (-den, -dan, -ten, -tan)
 */
export function getAblativeSuffix(word) {
  const lower = (word || 'Yemek').toLowerCase().trim();
  let lastVowelIsBack = true;

  for (let i = lower.length - 1; i >= 0; i--) {
    if (BACK_VOWELS.has(lower[i])) { lastVowelIsBack = true; break; }
    if (FRONT_VOWELS.has(lower[i])) { lastVowelIsBack = false; break; }
  }

  const lastChar = lower[lower.length - 1];
  const hard = VOICELESS.has(lastChar);
  
  const allVowels = new Set([...BACK_VOWELS, ...FRONT_VOWELS]);
  const isVowel = allVowels.has(lastChar);
  
  let suffix = lastVowelIsBack ? (hard ? 'tan' : 'dan') : (hard ? 'ten' : 'den');
  
  // Kaynaştırma 'n' for possessives (Common in food names: -ı, -i, -u, -ü, -sı, -si...)
  const possessiveEndings = new Set([...'ıiuüİI']);
  if (isVowel && possessiveEndings.has(lastChar)) {
    if (lower.length > 3 || lower.endsWith('si') || lower.endsWith('sı')) {
      return 'n' + suffix;
    }
  }

  return suffix;
}

/**
 * Kelimeye uygun yönelme ekini (dative suffix: -e, -a, -ye, -ya, -ne, -na) döndürür.
 * @param {string} word - Ek getirilecek kelime
 * @param {'user' | 'dish'} type - Kelimenin tipi (Kullanıcı adı veya Yemek adı)
 */
export function getDativeSuffix(word, type = 'user') {
  if (!word) return '';

  // JavaScript'te Regex 'i' flag'i Türkçe I ve İ harflerinde hatalı çalışır.
  // Bu yüzden büyük/küçük harfleri açıkça tanımlamak en güvenlisidir.
  const vowelsRegex = /[aıeiöüouAİIEOUÖÜ]/g;
  const vowelMatches = word.match(vowelsRegex);

  if (!vowelMatches) return 'e'; // Hiç sesli harf yoksa varsayılan

  const lastVowel = vowelMatches[vowelMatches.length - 1];

  // Son ünlünün kalın/ince kontrolünü doğrudan harf üzerinden yapıyoruz (toLowerCase hatasını önler)
  const isFront = /[eiöüEİÖÜ]/.test(lastVowel);
  const suffix = isFront ? 'e' : 'a';

  // Kelimenin sonu ünlüyle mi bitiyor?
  const endsWithVowel = /[aıeiöüouAİIEOUÖÜ]$/.test(word);

  if (endsWithVowel) {
    let buffer = 'y'; // Varsayılan kaynaştırma harfi 'y' (Örn: Ali'ye, Ayşe'ye)

    if (type === 'dish') {
      // Yemek isimlerinde isim tamlaması olma ihtimali: (Örn: "Mercimek Çorbası", "İzmir Köftesi")
      // Eğer kelimede boşluk varsa ve son harfi ı, i, u, ü ise 'n' kaynaştırması alır.
      const isPossessiveCompound = word.includes(' ') && /[ıiuüIİUÜ]$/.test(word);
      if (isPossessiveCompound) {
        buffer = 'n'; // (Örn: Mercimek Çorbası'na)
      }
    }

    // Not: Kullanıcı adları (type === 'user') ASLA 'n' kaynaştırması almaz. 
    // "Ali Veli" ismi "Veli'ne" değil, "Veli'ye" olmalıdır.
    return buffer + suffix;
  }

  return suffix;
}

export function getCommentContextHtml(c) {
  if (!c) return `menüye yorum yaptı`;

  const authorUsername = c.user?.nickname || c.user?.username || c.author_username || '';
  const parentUsername = c.parent_username || c.parent_user?.username || c.target_user?.username || (typeof c.target_user === 'string' ? c.target_user : null);

  // If this comment is a reply to another comment
  if (c.parent_id || c.is_reply || parentUsername) {
    if (parentUsername) {
      if (
        parentUsername === '@self' ||
        (authorUsername && parentUsername.toLowerCase() === authorUsername.toLowerCase())
      ) {
        return `kendine yanıt verdi`;
      }

      const cleanUsername = parentUsername.startsWith('@') ? parentUsername.slice(1) : parentUsername;
      const dative = getDativeSuffix(cleanUsername, 'user');
      const safeDisplay = sanitizeText(cleanUsername);
      const safeUrl = encodeURIComponent(cleanUsername);

      return `<a href="/biri/${safeUrl}" data-link class="comment-card__target"><strong>@${safeDisplay}</strong></a>'${dative} yanıt verdi`;
    }

    return `bir yoruma yanıt verdi`;
  }

  const dishName = c.dish_name || c.dish?.name;
  if (dishName) {
    const safeDisplay = sanitizeText(dishName);
    return `<strong>${safeDisplay}</strong> için yorum yaptı`;
  }

  return `menüye yorum yaptı`;
}

export const TURKISH_MONTHS = [
  "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran",
  "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"
];

export const TURKISH_DAYS = [
  "Pazar", "Pazartesi", "Salı", "Çarşamba", "Perşembe", "Cuma", "Cumartesi"
];

/**
 * Formats a date string (YYYY-MM-DD or Date) into natural Turkish (e.g. "20 Ağustos 2026").
 * @param {string | Date} input
 * @param {boolean} includeDay
 * @returns {string}
 */
export function formatFullTurkishDate(input, includeDay = false) {
  if (!input) return "";
  const d = typeof input === "string" ? new Date(input.includes("T") ? input : `${input}T12:00:00`) : input;
  if (isNaN(d.getTime())) return String(input);
  const day = d.getDate();
  const month = TURKISH_MONTHS[d.getMonth()];
  const year = d.getFullYear();
  if (includeDay) {
    const dayName = TURKISH_DAYS[d.getDay()];
    return `${day} ${month} ${year} ${dayName}`;
  }
  return `${day} ${month} ${year}`;
}