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
  if (c.is_reply && c.target_user.username) {
    const target = c.target_user.username;
    
    if (target === '@self') {
      return `kendi yorumuna yanıt verdi`;
    }

    const dative = getDativeSuffix(target, 'user');

    // Görüntülenen metin için HTML sanitize işlemi yapıyoruz (XSS koruması)
    const safeDisplay = sanitizeText(target);

    // URL yapısı için boşlukları ve Türkçe karakterleri uygun formata çeviriyoruz
    const safeUrl = encodeURIComponent(target);

    return `<a href="/biri/${safeUrl}" data-link class="comment-card__target"><strong>${safeDisplay}</strong></a>'${dative} yanıt verdi`;
  }

  if (c.dish && c.dish.name) {
    const dishName = c.dish.name;
    const dative = getDativeSuffix(dishName, 'dish');
    const safeDisplay = sanitizeText(dishName);

    return `<strong>${safeDisplay}</strong>'${dative} yorum yaptı`;
  }

  return `menüye yorum yaptı`;
}