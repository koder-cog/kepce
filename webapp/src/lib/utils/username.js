/**
 * Öğrenci ve yurt yemekhanesi temalı rastgele kullanıcı adı üreticisi.
 *
 * Üretilen adlar daima 3-32 karakter aralığında, İngilizce alfabe ve
 * 4 haneli sayı içerir (Reddit tarzı PascalCase: TuzluMercimek7893).
 */

export const USERNAME_ADJECTIVES = Object.freeze([
  // Tat ve Kıvam
  'aci',
  'tuzlu',
  'tuzsuz',
  'tatli',
  'eksi',
  'yagli',
  'yagsiz',
  'kuru',
  'suluca',
  'lapa',
  'salcali',
  'soslu',
  'sade',
  'karisik',
  'lezzetli',
  'lezzetsiz',
  'kemikli',
  'kemiksiz',
  'bulyonlu',
  'sirkeli',
  'sirkesiz',
  'limonlu',

  // Pişme & Tazelik Durumu
  'bayat',
  'taze',
  'sicak',
  'soguk',
  'ilik',
  'buzlu',
  'kaynar',
  'haslak',
  'yanik',
  'pismis',
  'cig',
  'kizarmis',
  'haslanmis',
  'donuk',
  'kayis',
  'curuk',
  'patlak',
  'yamuk',
  'tasli',
  'gevrek',

  // Kişilik & Yemekhane Rolleri
  'gurme',
  'ac',
  'doymus',
  'doyumsuz',
  'obur',
  'istahsiz',
  'hizli',
  'yavas',
  'komik',
  'sinirli',
  'mutlu',
  'uykulu',
  'kivrak',
  'saskin',
  'kitlik',
  'seckin',
  'kacak',
  'nobetci',
  'burslu',
  'yurtlu',
  'kutsal',
  'sabahci',
  'gececi',
  'yorgun',
  'fukara',
  'cilgin',
  'tembel',
  'caliskan',
  'caylak',
  'comez',
  'mudavim',
  'kurnaz',
  'usengec',
  'gariban',
  'kederli',
  'umutlu',
  'hararetli',
  'durgun',
  'nazli',
  'efsane',
  'keskin',
  'hazir',
  'sabirli',
  'nizami',
  'vegan',
  'bedava',
  'stokcu',
  'kaynakci',
  'taseron'
]);

export const USERNAME_NOUNS = Object.freeze([
  // Araç-Gereç & Donanım
  'kepce',
  'tabldot',
  'tepsi',
  'kasik',
  'catal',
  'bicak',
  'tuzluk',
  'pecete',
  'sefertas',
  'ketil',
  'termos',
  'kupa',
  'bardak',
  'surahi',
  'cezve',
  'fincan',
  'kupon',
  'kart',
  'turnike',
  'fis',
  'kuyruk',
  'kasa',
  'tencere',
  'tabak',
  'zimba',

  // Sebze, Bakliyat & Garnitür
  'biber',
  'domates',
  'patates',
  'fasulye',
  'nohut',
  'pirinc',
  'mercimek',
  'bezelye',
  'brokoli',
  'marul',
  'salata',
  'tursu',
  'pure',
  'garnitur',
  'limon',
  'sirke',

  // Ana Yemekler & Et Ürünleri
  'pilav',
  'makarna',
  'corba',
  'tarhana',
  'ezogelin',
  'manti',
  'dolma',
  'sarma',
  'kofte',
  'tavuk',
  'et',
  'ciger',
  'taslik',
  'sucuk',
  'sosis',
  'kebab',
  'menemen',

  // Hamur İşi & Kahvaltılık
  'ekmek',
  'somun',
  'borek',
  'pogaca',
  'gevrek',
  'simit',
  'margarin',
  'bulyon',

  // İçecek & Meyve & Tatlı
  'ayran',
  'su',
  'cacik',
  'komposto',
  'hosaf',
  'elma',
  'armut',
  'karpuz',
  'kavun',
  'mandalina',
  'tatli',
  'sutlac',
  'helva',
  'revani',
  'irmik',
  'kadayif',
  'baklava',

  // Mekan & Yurt Hayatı
  'ranza',
  'bavul',
  'terlik',
  'dolap',
  'oda',
  'blok',
  'kat',
  'koridor',
  'etut',
  'kantin',
  'reyon',
  'kampus',

  // Figürler & Tipler
  'asci',
  'sef',
  'baskan',
  'mudur',
  'memur',
  'muhendis',
  'gazi',
  'ajan',
  'avci',
  'sansar',
  'sabotajci',
  'porsiyon'
]);

/**
 * Belirtilen aralıkta kriptografik güvenli rastgele tam sayı üretir.
 * Tarayıcı veya test ortamında Web Crypto API yoksa Math.random tabanlı yedek çalışır.
 *
 * @param {number} min - Alt sınır (dahil)
 * @param {number} max - Üst sınır (dahil)
 * @returns {number}
 */
export function getCryptoRandomInt(min, max) {
  const range = max - min + 1;
  const cryptoObj = typeof globalThis !== 'undefined' ? globalThis.crypto : null;

  if (cryptoObj?.getRandomValues) {
    const buffer = new Uint32Array(1);
    cryptoObj.getRandomValues(buffer);
    return min + (buffer[0] % range);
  }

  return Math.floor(Math.random() * range) + min;
}

/**
 * Verilen diziden rastgele bir eleman seçer.
 *
 * @template T
 * @param {readonly T[]} list
 * @returns {T}
 */
export function pickRandom(list) {
  if (!list || list.length === 0) return '';
  const idx = getCryptoRandomInt(0, list.length - 1);
  return list[idx];
}

/**
 * Kelimenin ilk harfini büyük, kalanını küçük harfe dönüştürür.
 *
 * @param {string} word
 * @returns {string}
 */
export function capitalize(word) {
  if (!word) return '';
  return word.charAt(0).toUpperCase() + word.slice(1).toLowerCase();
}

/**
 * Reddit tarzı [Sıfat][İsim][4_haneli_sayı] (PascalCase) biçiminde geçerli bir rastgele kullanıcı adı üretir.
 * Örnek: TuzluMercimek7893
 *
 * 32 karakter tavanını asla aşmayacak şekilde kelime havuzundaki uzunluklar
 * kontrol altındadır; her ihtimale karşı güvenlik kesme filtresi uygulanır.
 *
 * @returns {string}
 */
export function generateRandomUsername() {
  const adj = capitalize(pickRandom(USERNAME_ADJECTIVES));
  const noun = capitalize(pickRandom(USERNAME_NOUNS));
  const num = getCryptoRandomInt(1000, 9999);

  let candidate = `${adj}${noun}${num}`;

  if (candidate.length > 32) {
    candidate = candidate.slice(0, 32);
  }

  return candidate;
}

export const USERNAME_REGEX = /^[a-zA-Z][a-zA-Z0-9]*(?:[_-][a-zA-Z0-9]+)*$/;

/**
 * Kullanıcı adının sistem kuralına (3-32 karakter, harfle başlama, İngilizce harf, rakam,
 * tekil alt çizgi veya tire) uyup uymadığını doğrular.
 *
 * @param {string} username
 * @returns {boolean}
 */
export function isValidUsername(username) {
  if (typeof username !== 'string') return false;
  const trimmed = username.trim();
  if (trimmed.length < 3 || trimmed.length > 32) return false;
  return USERNAME_REGEX.test(trimmed);
}
