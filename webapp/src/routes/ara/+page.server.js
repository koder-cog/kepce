import { redirect } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { resolveBang } from "$lib/search/bangs.js";
import {
  solveUnitConversion,
  solveWorldTime,
  solveTdkDefinition,
  solveCryptoPrice,
} from "$lib/search/instantSolvers.js";
import { CITY_MAP, TURKEY_GEO_MAP } from "@/utils/turkish.js";
import { apiGet, normalizeMenuList, istanbulToday } from "@/lib/server/api.js";

// Bellek içi LRU Arama Önbelleği (10 dk TTL)
const searchCache = new Map();
const CACHE_TTL_MS = 10 * 60 * 1000;

function getCached(key) {
  const item = searchCache.get(key);
  if (!item) return null;
  // Hatalı veya süresi geçmiş kayıtları önbellekten derhal temizle
  if (Date.now() - item.ts > CACHE_TTL_MS || item.data?.error) {
    searchCache.delete(key);
    return null;
  }
  return item.data;
}

function setCached(key, data) {
  // Başarısız veya hatalı yanıtları ASLA önbelleğe alma!
  if (!data || data.error) return;
  if (searchCache.size > 500) {
    const oldestKey = searchCache.keys().next().value;
    searchCache.delete(oldestKey);
  }
  searchCache.set(key, { ts: Date.now(), data });
}

/**
 * URL'lerdeki bilinen izleme ve analitik parametrelerini (UTM, Facebook, Google, Yandex vb.) temizler.
 */
function cleanTrackingParams(rawUrl) {
  if (!rawUrl) return rawUrl;
  try {
    const url = new URL(rawUrl);
    const trackingKeys = [
      "utm_source", "utm_medium", "utm_campaign", "utm_term", "utm_content", "utm_id",
      "fbclid", "gclid", "gclsrc", "dclid", "msclkid", "yclid", "mc_eid",
      "igshid", "_hsenc", "_hsmi", "wickedid", "wt_zmc", "s_kwcid"
    ];
    let changed = false;
    for (const key of trackingKeys) {
      if (url.searchParams.has(key)) {
        url.searchParams.delete(key);
        changed = true;
      }
    }
    return changed ? url.toString() : rawUrl;
  } catch {
    return rawUrl;
  }
}

// Türkçe karakter normalizasyonu
function normalizeTr(str) {
  return str
    .toLowerCase()
    .replace(/ü/g, "u")
    .replace(/ö/g, "o")
    .replace(/ş/g, "s")
    .replace(/ç/g, "c")
    .replace(/ğ/g, "g")
    .replace(/ı/g, "i")
    .replace(/[^a-z0-9\s]/g, "")
    .trim();
}

/**
 * Çift kademeli niyet ve varlık süzgeci (Dual-gate intent filter).
 * Kullanıcının araması hem bir şehir/üniversite hem de yemek/yurt/menü niyeti taşıyorsa,
 * veya doğrudan bir platform modülü (fiyat, arşiv vb.) aranıyorsa özel Kepçe kartı üretir.
 * "Ankara kedisi", "İstanbul sözleşmesi" gibi aramalar filtrelenir.
 */
function matchKepceIntent(query) {
  if (!query) return null;
  const clean = query
    .toLowerCase()
    .replace(/[''’`]/g, "")
    .replace(/[^\w\sğüşıöçĞÜŞİÖÇ]/g, " ")
    .trim();

  const tokens = clean.split(/\s+/).filter(Boolean);
  if (tokens.length === 0) return null;

  // 1. Doğrudan Platform Modül Niyetleri
  if (clean.includes("fiyat") && (clean.includes("hesap") || clean.includes("tarife") || clean.includes("tabldot") || clean.includes("ne kadar") || clean.includes("kac para") || clean.includes("kaç para"))) {
    return {
      type: "module",
      title: "KYK Tabldot & Yemek Fiyatı Hesaplama",
      subtitle: "Kepçe Hesaplama Aracı",
      description: "Güncel GSB beslenme yardımı, tabldot sınırları ve ekstra ürün fiyatlarını hesaplayın.",
      href: "https://kepce.org/kyk-beslenme-yardimi",
      badge: "Araç",
      cta: "Fiyat Hesapla",
    };
  }

  if (clean.includes("arsiv") || clean.includes("arşiv") || (clean.includes("gecmis") && (clean.includes("menu") || clean.includes("menü")))) {
    return {
      type: "module",
      title: "Geçmiş Menü Arşivi",
      subtitle: "Tarihsel Yemekhane Kayıtları",
      description: "Tüm illerin geçmiş aylardaki ve yıllardaki KYK yemekhane menülerini inceleyin.",
      href: "https://kepce.org/arsiv",
      badge: "Arşiv",
      cta: "Arşive Git",
    };
  }

  if (clean.includes("menu yukle") || clean.includes("menü yükle") || clean.includes("menu gonder") || clean.includes("menü gönder")) {
    return {
      type: "module",
      title: "Yemekhane Menüsü Gönder",
      subtitle: "Topluluk Katkısı",
      description: "Yurdunuzun güncel yemek listesini veya fotoğrafını sisteme yükleyin.",
      href: "https://kepce.org/menu-gonder",
      badge: "Katkı",
      cta: "Menü Yükle",
    };
  }

  if (clean === "kepce" || clean === "kepçe" || clean === "kepce nedir" || clean === "kepçe nedir") {
    return {
      type: "module",
      title: "Kepçe - Açık Menü ve Yemekhane Platformu",
      subtitle: "Hakkında",
      description: "81 ilin KYK yurt yemekhanesi menüleri, beslenme saatleri, öğrenci yorumları ve şeffaf beslenme yardımı takibi.",
      href: "https://kepce.org/hakkinda",
      badge: "Platform",
      cta: "Keşfet",
    };
  }

  // 2. Şehir + Yemek/Yurt Niyeti Eşleştirmesi (Çift Kademeli)
  const mealKeywords = ["yemek", "menu", "menü", "tabldot", "kyk", "yurt", "yurdu", "yurtlar", "yemekhane", "kahvalti", "kahvaltı", "ogle", "öğle", "aksam", "akşam", "saatleri", "yardim", "yardım"];
  const hasMealIntent = tokens.some((t) => mealKeywords.some((mk) => t.includes(mk)));

  // Eğer yemek/yurt/menü niyeti YOKSA kesinlikle şehir kartı basma (False-positive engeli)
  if (!hasMealIntent) return null;

  // Popüler Üniversite Eşleşmeleri (Kampüs çevresi KYK Yurtları uyarısıyla)
  const uniCityMap = {
    itu: { slug: "istanbul", name: "İstanbul", uni: "İTÜ Çevresi" },
    odtu: { slug: "ankara", name: "Ankara", uni: "ODTÜ Çevresi" },
    boun: { slug: "istanbul", name: "İstanbul", uni: "Boğaziçi Çevresi" },
    hacettepe: { slug: "ankara", name: "Ankara", uni: "Hacettepe Çevresi" },
    yildiz: { slug: "istanbul", name: "İstanbul", uni: "YTÜ Çevresi" },
    ege: { slug: "izmir", name: "İzmir", uni: "Ege Üniversitesi Çevresi" },
    deu: { slug: "izmir", name: "İzmir", uni: "Dokuz Eylül Çevresi" },
    marmara: { slug: "istanbul", name: "İstanbul", uni: "Marmara Çevresi" },
    gazi: { slug: "ankara", name: "Ankara", uni: "Gazi Üniversitesi Çevresi" },
  };

  for (const [uniKey, uniInfo] of Object.entries(uniCityMap)) {
    if (clean.includes(uniKey)) {
      return {
        type: "city_menu",
        slug: uniInfo.slug,
        title: `${uniInfo.name} KYK Yurtları Yemek Menüsü`,
        subtitle: `${uniInfo.uni} GSB Yurt Menüsü (Üniversite rektörlük menüsü değildir)`,
        description: `${uniInfo.name} genelindeki KYK yurt yemekhanelerinde bugünün sabah kahvaltısı ve akşam tabldot listesi.`,
        href: `https://kepce.org/${uniInfo.slug}`,
        badge: "KYK Menüsü",
        cta: "Detayları Kepçe'de incele",
      };
    }
  }

  // 81 İl Eşleştirmesi
  const normCityKeys = Object.keys(CITY_MAP);
  for (const slug of normCityKeys) {
    const cityName = CITY_MAP[slug];
    const cityNameNorm = normalizeTr(cityName);
    if (tokens.some((t) => t === slug || t === cityNameNorm || t.startsWith(slug) || t.startsWith(cityNameNorm))) {
      return {
        type: "city_menu",
        slug,
        title: `${cityName} KYK Yurtları Yemek Menüsü`,
        subtitle: "Günlük GSB Yurt Yemekhanesi Listesi",
        description: `${cityName} genelindeki tüm KYK yurtlarında geçerli bugünkü sabah kahvaltısı ve akşam tabldot menüsü.`,
        href: `https://kepce.org/${slug}`,
        badge: "KYK Menüsü",
        cta: "Detayları Kepçe'de incele",
      };
    }
  }

  return null;
}

// Open-Meteo ve Coğrafi Konum Servisi (TURKEY_GEO_MAP öncelikli)
async function fetchPlaceDetails(query, customFetch = fetch) {
  try {
    const qNorm = normalizeTr(query);
    let lat = null;
    let lon = null;
    let displayName = null;
    let country = "Türkiye";

    // 1. Ülke genel araması kontrolü (Bursa'daki mobilyacı koordinatının dönmesini önle)
    if (qNorm === "turkiye" || qNorm === "turkey") {
      return {
        name: "Türkiye",
        country: "Türkiye",
        lat: 38.9637,
        lon: 35.2433,
        weather: null,
      };
    }

    // 2. Yerel statik Türkiye haritasına bak (0ms, 81 il merkezi)
    if (TURKEY_GEO_MAP[qNorm]) {
      const g = TURKEY_GEO_MAP[qNorm];
      lat = g.lat;
      lon = g.lon;
      displayName = g.name;
    }

    // 3. Yerel haritada bulunamadıysa Nominatim'e başvur (Sadece yer ve idari sınırlar için)
    if (lat === null || lon === null) {
      const geoUrl = `https://nominatim.openstreetmap.org/search?q=${encodeURIComponent(query)}&format=json&limit=1&addressdetails=1`;
      const geoRes = await customFetch(geoUrl, {
        headers: { "User-Agent": "Kepce/1.0 (bilgi@kepce.org)" },
        signal: AbortSignal.timeout(2000),
      });

      if (!geoRes.ok) return null;
      const geoData = await geoRes.json();
      if (!geoData || geoData.length === 0) return null;

      const top = geoData[0];
      // Dükkan, havalimanı ve rastgele işletmeleri yer olarak kabul etme (Alaska / Mobilyacı engeli)
      const badClasses = ["shop", "amenity", "aeroway", "craft", "office", "club"];
      if (badClasses.includes(top.class)) {
        return null;
      }

      lat = parseFloat(top.lat);
      lon = parseFloat(top.lon);
      displayName = top.name || top.display_name.split(",")[0];
      country = top.address?.country || "";
    }

    // Open-Meteo ile güncel ve 3 günlük hava tahmini
    const weatherUrl = `https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&current=temperature_2m,weather_code&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto&forecast_days=3`;
    const wRes = await customFetch(weatherUrl, { signal: AbortSignal.timeout(2000) });

    let weather = null;
    if (wRes.ok) {
      const wData = await wRes.json();
      weather = {
        currentTemp: Math.round(wData.current?.temperature_2m ?? 0),
        weatherCode: wData.current?.weather_code ?? 0,
        daily: (wData.daily?.time || []).map((t, idx) => ({
          date: t,
          maxTemp: Math.round(wData.daily.temperature_2m_max[idx]),
          minTemp: Math.round(wData.daily.temperature_2m_min[idx]),
          code: wData.daily.weather_code[idx],
        })),
      };
    }

    return {
      name: displayName,
      country,
      lat,
      lon,
      weather,
    };
  } catch {
    return null;
  }
}

const ACRONYMS_MAP = {
  odtu: ["middle east technical university", "orta dogu teknik universitesi", "metu"],
  metu: ["middle east technical university", "orta dogu teknik universitesi", "odtu"],
  itu: ["istanbul teknik universitesi", "istanbul technical university"],
  boun: ["bogazici universitesi", "bogazici"],
  bogazici: ["bogazici universitesi", "boun"],
  ytu: ["yildiz teknik universitesi", "yildiz technical"],
  yildiz: ["yildiz teknik universitesi", "yildiz technical", "ytu"],
  tubitak: ["turkiye bilimsel ve teknolojik arastirma kurumu"],
  yok: ["yuksekogretim kurulu"],
  osym: ["olcme secme ve yerlestirme merkezi"],
  meb: ["milli egitim bakanligi"],
  tdk: ["turk dil kurumu"],
  tcmb: ["turkiye cumhuriyet merkez bankasi"],
  tbmm: ["turkiye buyuk millet meclisi"],
  mit: ["massachusetts institute of technology", "milli istihbarat teskilati"],
  cern: ["conseil europeen pour la recherche nucleaire"],
  nasa: ["national aeronautics and space administration"],
};

// Sorgu-Başlık Uyum Filtresi (Relevance Gate)
function isRelevantInfobox(query, boxOrTitle) {
  if (!query || !boxOrTitle) return false;

  // Anlam ayrımı (disambiguation) sayfalarını bilgi kartı olarak öne çıkarma
  if (typeof boxOrTitle === "object") {
    if (boxOrTitle.entityType === "disambiguation") return false;
    const cLow = (boxOrTitle.content || "").toLowerCase();
    if (cLow.includes("aşağıdaki anlamlara gelebilir") || cLow.includes("anlam ayrımı")) {
      return false;
    }
  }

  const qNorm = normalizeTr(query);
  const title = typeof boxOrTitle === "string" ? boxOrTitle : (boxOrTitle.title || boxOrTitle.infobox || "");
  const tNorm = normalizeTr(title);

  if (tNorm.includes(qNorm) || qNorm.includes(tNorm)) return true;

  // Kısaltma eşleşmesi kontrolü (örn: "odtu" -> "middle east technical university")
  if (ACRONYMS_MAP[qNorm]) {
    const expansions = ACRONYMS_MAP[qNorm];
    if (expansions.some((exp) => tNorm.includes(exp) || exp.includes(tNorm))) {
      return true;
    }
  }

  // Eğer box objesi verilmişse url ve kimlik kontrolü yap
  if (typeof boxOrTitle === "object") {
    if (boxOrTitle.id && normalizeTr(boxOrTitle.id).includes(qNorm)) return true;
    if (Array.isArray(boxOrTitle.urls)) {
      for (const u of boxOrTitle.urls) {
        const uStr = `${u.title || ""} ${u.url || ""}`.toLowerCase();
        if (uStr.includes(qNorm)) return true;
      }
    }
  }

  const qWords = qNorm.split(/\s+/).filter((w) => w.length > 1);
  const tWords = tNorm.split(/\s+/).filter((w) => w.length > 1);

  if (qWords.length === 0 || tWords.length === 0) return false;

  const qInT = qWords.filter((w) => tWords.some((tw) => tw.includes(w) || w.includes(tw)));
  if (qInT.length / qWords.length >= 0.4) return true;

  const tInQ = tWords.filter((w) => qWords.some((qw) => qw.includes(w) || w.includes(qw)));
  if (tInQ.length / tWords.length >= 0.4) return true;

  return false;
}

// Gereksiz dış bağlantıları filtreleme ve sadeleştirme (P18, Q123 gibi Wikidata çöpleri engellenir)
const JUNK_URL_PATTERNS = [/^[pq]\d+$/i, /musicbrainz/i, /^commons/i, /wikidata/i, /property:/i];

function cleanUrls(urls) {
  if (!urls || urls.length === 0) return [];

  const hasTrWiki = urls.some(
    (u) =>
      (u.url || "").includes("tr.wikipedia.org") ||
      (u.title || "").toLowerCase() === "vikipedi",
  );

  return urls
    .filter((link) => {
      const t = (link.title || "").trim().toLowerCase();
      const u = (link.url || "").toLowerCase();

      if (JUNK_URL_PATTERNS.some((p) => p.test(t) || p.test(u))) return false;
      if (t === "wikidata" || u.includes("wikidata.org") || u.includes("/wiki/property:")) return false;
      if (t.includes("musicbrainz") || u.includes("musicbrainz.org")) return false;
      if (t === "kaynak" || t === "source" || t.length <= 2) return false;

      // Türkçe Vikipedi varsa İngilizce Vikipedi kopyasını gösterme
      if (hasTrWiki && (u.includes("en.wikipedia.org") || t.includes("(en)"))) {
        return false;
      }

      return true;
    })
    .map((link) => {
      let title = link.title || "";
      const tLower = title.toLowerCase();
      if (tLower.includes("official") || tLower.includes("resmî") || tLower.includes("resmi")) {
        title = "Resmî site";
      } else if (tLower.includes("wikipedia") || tLower.includes("vikipedi")) {
        title = "Vikipedi";
      } else if (tLower.includes("openstreetmap")) {
        title = "Harita";
      }
      return {
        ...link,
        title,
      };
    })
    .filter((link) => link.title && link.title.length > 2);
}

// Nitelik etiketlerini insan diline ve kısa forma dönüştürme
const LABEL_REPLACEMENTS = {
  "başkanı veya başkanları": "Yönetici",
  "başkan veya başkanlar": "Yönetici",
  "belediye başkanı": "Belediye Başkanı",
  "ortaya çıkışı": "Kuruluş",
  "yüzölçümü": "Alan",
  "yüzölçüm": "Alan",
  "alanı": "Alan",
  "nüfusu": "Nüfus",
  "nüfus": "Nüfus",
  "vatandaşlığı": "Vatandaşlık",
  "doğum tarihi": "Doğum",
  "ölüm tarihi": "Ölüm",
  "etkin yılları": "Etkin Yıllar",
  "çalışan sayısı": "Çalışan Sayısı",
  "kuruluş tarihi": "Kuruluş",
  "genel merkez": "Genel Merkez",
  "posta kodu": "Posta Kodu",
};

function formatAttrLabel(label) {
  if (!label) return "";
  const lower = label.trim().toLowerCase();
  return LABEL_REPLACEMENTS[lower] || label.trim();
}

// Birim ve tarih formatlama
const DAY_NAMES_TR = /\s+(pazartesi|salı|çarşamba|perşembe|cuma|cumartesi|pazar)$/i;

function formatAttrValue(label, value) {
  if (!value) return "";
  let str = String(value).trim();

  // Tarih sonundaki gün adlarını temizleme
  str = str.replace(DAY_NAMES_TR, "");

  // Büyük sayıları binlik basamaklara ayırma (örn: 592713 -> 592.713)
  if (/^\d{4,9}$/.test(str)) {
    const num = parseInt(str, 10);
    if (!isNaN(num)) {
      return num.toLocaleString("tr-TR");
    }
  }

  // Alan için km² ekleme ve yanlış m² formatlarını düzeltme
  const labelLower = (label || "").toLowerCase();
  if (labelLower.includes("alan") || labelLower.includes("yüzölçüm")) {
    str = str.replace(/\bkm2\b/gi, "km²").replace(/\bm[²2]\b/gi, "km²");
    if (/^\d+(?:[.,\s]\d+)*$/.test(str)) {
      return `${str} km²`;
    }
  }

  return str;
}

// Varlık Tipi Sınıflandırıcısı
function classifyEntity(infobox, query) {
  const text = `${infobox.title} ${infobox.content || ""}`.toLowerCase();
  const labels = (infobox.attributes || []).map((a) => a.label.toLowerCase());

  // 0. Anlam Ayrımı (Disambiguation) Kontrolü
  if (
    text.includes("aşağıdaki anlamlara gelebilir") ||
    text.includes("anlam ayrımı") ||
    text.includes("birden fazla anlama gelebilir") ||
    infobox.type === "disambiguation" ||
    (infobox.title && infobox.title.toLowerCase().includes("anlam ayrımı"))
  ) {
    return "disambiguation";
  }

  // 1. Kurum / Üniversite (Organization) Kontrolü
  const isOrgText =
    text.includes("üniversite") ||
    text.includes("university") ||
    text.includes("enstitü") ||
    text.includes("institute") ||
    text.includes("college") ||
    text.includes("vakıf") ||
    text.includes("kurumu") ||
    text.includes("şirketi") ||
    text.includes("kulübü");

  const orgLabels = [
    "rektör",
    "genel merkez",
    "ceo",
    "kuruluş tarihi",
    "yönetim kurulu başkanı",
    "çalışan sayısı",
  ];

  if ((isOrgText || labels.some((l) => orgLabels.includes(l))) && !labels.some((l) => ["doğum tarihi", "ölüm tarihi", "eşi"].includes(l))) {
    return "organization";
  }

  // 2. Kişi / Biyografi (Person) Kontrolü
  const personLabels = [
    "doğum tarihi",
    "ölüm tarihi",
    "vatandaşlığı",
    "eşi",
    "çocukları",
    "mesleği",
    "etkin yılları",
    "eğitimi",
  ];
  if (labels.some((l) => personLabels.includes(l))) {
    return "person";
  }
  const personKeywords = [
    "türk siyasetçi",
    "türk oyuncu",
    "türk yazar",
    "türk futbolcu",
    "türk akademisyen",
    "devlet adamı",
    "cumhurbaşkanı",
    "başbakan",
    "şair",
    "yazar",
    "oyuncu",
    "müzisyen",
    "besteci",
    "ressam",
    "futbolcu",
    "basketbolcu",
    "bilim insanı",
    "profesör",
    "tarihçi",
    "şarkıcı",
    "sanatçı",
    "müzisyen",
    "şarkı yazarı",
    "internet ünlüsü",
    "fenomen",
    "spiker",
    "sunucu",
    "yönetmen",
    "yapımcı",
  ];
  if (personKeywords.some((k) => text.includes(k))) {
    return "person";
  }

  // 3. Yer / Coğrafya (Place) Kontrolü
  const placeLabels = [
    "başkenti",
    "nüfus",
    "nüfusu",
    "alanı",
    "yüzölçümü",
    "rakımı",
    "koordinatları",
    "su hacmi",
    "en yüksek noktası",
    "derinliği",
    "konumu",
    "bölgesi",
  ];
  if (labels.some((l) => placeLabels.includes(l))) {
    return "place";
  }
  const placeKeywords = [
    "başkent",
    "şehir",
    "gölü",
    "göl",
    "dağı",
    "dağ",
    "nehri",
    "nehir",
    "ilçesi",
    "ilçe",
    "adası",
    "ada",
    "körfezi",
    "şelalesi",
    "kenti",
    "bölgesi",
    "kasabası",
    "vadisi",
    "kanyonu",
    "denizi",
    "boğazı",
    "plajı",
  ];
  if (placeKeywords.some((k) => text.includes(k))) {
    return "place";
  }

  // 4. Varsayılan: Nesne / Kavram / Araç (Thing)
  return "thing";
}

// ── Canlı Döviz ve Matematik Motoru (Instant Answer Engine) ─────────────────────────
const CURRENCY_CODES = {
  dolar: { code: "USD", name: "Amerikan Doları" },
  usd: { code: "USD", name: "Amerikan Doları" },
  dollar: { code: "USD", name: "Amerikan Doları" },
  "$": { code: "USD", name: "Amerikan Doları" },
  euro: { code: "EUR", name: "Euro" },
  avro: { code: "EUR", name: "Euro" },
  eur: { code: "EUR", name: "Euro" },
  "€": { code: "EUR", name: "Euro" },
  sterlin: { code: "GBP", name: "İngiliz Sterlini" },
  gbp: { code: "GBP", name: "İngiliz Sterlini" },
  pound: { code: "GBP", name: "İngiliz Sterlini" },
  "£": { code: "GBP", name: "İngiliz Sterlini" },
  tl: { code: "TRY", name: "Türk Lirası" },
  try: { code: "TRY", name: "Türk Lirası" },
  lira: { code: "TRY", name: "Türk Lirası" },
  "₺": { code: "TRY", name: "Türk Lirası" },
  yen: { code: "JPY", name: "Japon Yeni" },
  jpy: { code: "JPY", name: "Japon Yeni" },
  "¥": { code: "JPY", name: "Japon Yeni" },
  frank: { code: "CHF", name: "İsviçre Frangı" },
  chf: { code: "CHF", name: "İsviçre Frangı" },
  aud: { code: "AUD", name: "Avustralya Doları" },
  cad: { code: "CAD", name: "Kanada Doları" },
  cny: { code: "CNY", name: "Çin Yuanı" },
  yuan: { code: "CNY", name: "Çin Yuanı" },
  rmb: { code: "CNY", name: "Çin Yuanı" },
  rub: { code: "RUB", name: "Rus Rublesi" },
  ruble: { code: "RUB", name: "Rus Rublesi" },
  sar: { code: "SAR", name: "Suudi Arabistan Riyali" },
  riyal: { code: "SAR", name: "Suudi Arabistan Riyali" },
  aed: { code: "AED", name: "BAE Dirhemi" },
  dirhem: { code: "AED", name: "BAE Dirhemi" },
  sek: { code: "SEK", name: "İsveç Kronu" },
  nok: { code: "NOK", name: "Norveç Kronu" },
  dkk: { code: "DKK", name: "Danimarka Kronu" },
  kron: { code: "SEK", name: "İsveç Kronu" },
  krw: { code: "KRW", name: "Güney Kore Wonu" },
  won: { code: "KRW", name: "Güney Kore Wonu" },
  "₩": { code: "KRW", name: "Güney Kore Wonu" },
  inr: { code: "INR", name: "Hindistan Rupisi" },
  rupi: { code: "INR", name: "Hindistan Rupisi" },
  "₹": { code: "INR", name: "Hindistan Rupisi" },
  brl: { code: "BRL", name: "Brezilya Reali" },
  real: { code: "BRL", name: "Brezilya Reali" },
  pln: { code: "PLN", name: "Polonya Zlotisi" },
  zloti: { code: "PLN", name: "Polonya Zlotisi" },
  czk: { code: "CZK", name: "Çek Korunası" },
  bgn: { code: "BGN", name: "Bulgar Levası" },
  leva: { code: "BGN", name: "Bulgar Levası" },
  huf: { code: "HUF", name: "Macar Forinti" },
  forint: { code: "HUF", name: "Macar Forinti" },
  ron: { code: "RON", name: "Rumen Leyi" },
  ils: { code: "ILS", name: "İsrail Şekeli" },
  şekel: { code: "ILS", name: "İsrail Şekeli" },
  sekel: { code: "ILS", name: "İsrail Şekeli" },
  "₪": { code: "ILS", name: "İsrail Şekeli" },
  mxn: { code: "MXN", name: "Meksika Pezosu" },
  nzd: { code: "NZD", name: "Yeni Zelanda Doları" },
  sgd: { code: "SGD", name: "Singapur Doları" },
  hkd: { code: "HKD", name: "Hong Kong Doları" },
  zar: { code: "ZAR", name: "Güney Afrika Randı" },
  rand: { code: "ZAR", name: "Güney Afrika Randı" },
  thb: { code: "THB", name: "Tayland Bahtı" },
  baht: { code: "THB", name: "Tayland Bahtı" },
  "฿": { code: "THB", name: "Tayland Bahtı" },
  idr: { code: "IDR", name: "Endonezya Rupiahı" },
  myr: { code: "MYR", name: "Malezya Ringgiti" },
  php: { code: "PHP", name: "Filipinler Pezosu" },
};

let fxRatesCache = {
  timestamp: 0,
  dateStr: "",
  rates: { USD: 1, TRY: 48.27, EUR: 0.86, GBP: 0.74, JPY: 160.0, CHF: 0.81, AUD: 1.55, CAD: 1.38, CNY: 7.25 },
};

async function getFxRates(customFetch = fetch) {
  const now = Date.now();
  if (now - fxRatesCache.timestamp < 15 * 60 * 1000 && Object.keys(fxRatesCache.rates).length > 5) {
    return fxRatesCache;
  }
  try {
    const res = await customFetch("https://api.frankfurter.dev/v1/latest?base=USD", {
      signal: AbortSignal.timeout(3000),
    });
    if (res.ok) {
      const d = await res.json();
      fxRatesCache = {
        timestamp: now,
        dateStr: d.date || new Date().toISOString().slice(0, 10),
        rates: {
          USD: 1,
          ...d.rates,
        },
      };
    }
  } catch {
    // fallback cache
  }
  return fxRatesCache;
}

async function solveInstantQuery(query, customFetch = fetch) {
  const q = query.trim().toLowerCase();

  // 1. Döviz sorguları (örn: "50 dolar kaç tl", "100 euro kaç tl", "50 usd to try", "dolar kaç tl")
  const currencyMatch = q.match(/^(\d+(?:[.,]\d+)?\s*)?([a-z$€£₺¥]+)\s*(?:ka[cç]\s*([a-z$€£₺¥]+)|to\s*([a-z$€£₺¥]+)|([a-z$€£₺¥]+))$/i);
  if (currencyMatch) {
    const amountRaw = (currencyMatch[1] || "1").replace(",", ".");
    const fromSymbol = (currencyMatch[2] || "").toLowerCase();
    const toSymbol = (currencyMatch[3] || currencyMatch[4] || currencyMatch[5] || "tl").toLowerCase();

    const fromInfo = CURRENCY_CODES[fromSymbol];
    const toInfo = CURRENCY_CODES[toSymbol];

    if (fromInfo && toInfo && fromInfo.code !== toInfo.code) {
      const fxData = await getFxRates(customFetch);
      const amount = parseFloat(amountRaw) || 1;
      const fromRate = fxData.rates[fromInfo.code] || 1;
      const toRate = fxData.rates[toInfo.code] || 1;

      return {
        type: "currency",
        fromAmount: amount,
        fromCurrency: fromInfo.code,
        fromCurrencyName: fromInfo.name,
        toCurrency: toInfo.code,
        toCurrencyName: toInfo.name,
        fromRate,
        toRate,
        allRates: fxData.rates,
        date: fxData.dateStr || "Bugün",
      };
    }
  }

  // 2. Basit Matematik Hesaplamaları (örn: "125 * 8", "1500 / 12", "45 + 55")
  if (/^[\d\s.,+\-*/()^%]+$/.test(q) && /[+\-*/^%]/.test(q)) {
    try {
      const sanitized = q.replace(/,/g, ".").replace(/\^/g, "**");
      if (!/[a-zA-Z_$]/.test(sanitized)) {
        // eslint-disable-next-line no-new-func
        const result = Function(`'use strict'; return (${sanitized})`)();
        if (typeof result === "number") {
          let displayResult = "";
          if (isNaN(result)) {
            displayResult = "Tanımsız (0/0 belirsizliği)";
          } else if (!isFinite(result)) {
            displayResult = "Tanımsız (Sıfıra bölünemez)";
          } else {
            displayResult = result.toLocaleString("tr-TR", { maximumFractionDigits: 6 });
          }
          return {
            type: "calculator",
            expression: query.trim(),
            result: displayResult,
          };
        }
      }
    } catch {
      // ignore calculation error
    }
  }

  // 3. Birim Dönüştürücü (örn: "50 mil kaç km", "100 kg kaç pound", "100 fahrenheit kaç derece")
  const unitAnswer = solveUnitConversion(query);
  if (unitAnswer) {
    return unitAnswer;
  }

  // 5. TDK Sözlük & Tanım Çözücü (örn: "tabldot nedir", "pragmatik ne demek")
  const defAnswer = await solveTdkDefinition(query);
  if (defAnswer) {
    return defAnswer;
  }

  // 6. Canlı Kripto Takip Çözücü (örn: "btc kaç tl", "bitcoin kaç dolar", "eth kaç tl")
  const cryptoAnswer = await solveCryptoPrice(query);
  if (cryptoAnswer) {
    return cryptoAnswer;
  }

  return null;
}

// ── Görsel Fallback Yardımcısı (Vikipedi'de fotoğrafı olmayan ünlüler/kavramlar için) ──
async function fetchFallbackImage(query, searxUrl, customFetch) {
  try {
    const params = new URLSearchParams({
      q: query,
      categories: "images",
      format: "json",
    });
    const res = await customFetch(`${searxUrl.replace(/\/+$/, "")}/search?${params.toString()}`, {
      signal: AbortSignal.timeout(1500),
    });
    if (!res.ok) return null;
    const data = await res.json();
    const firstValid = (data.results || []).find((r) => r.img_src || r.thumbnail);
    return firstValid ? (firstValid.img_src || firstValid.thumbnail) : null;
  } catch {
    return null;
  }
}

// Wikipedia REST API Fallback (Kavram / Teknoloji aramalarında SearXNG infobox bulamazsa devreye girer)
async function fetchWikipediaFallback(query, customFetch = fetch) {
  if (!query || query.length < 2) return null;
  const qClean = query.trim().replace(/[?.,!]+$/, "");
  if (qClean.split(/\s+/).length > 3) return null;

  try {
    // 1. Önce doğrudan özet çekmeyi dene (hem girildiği gibi hem de baş harf büyük veya büyük harf varyantı ile)
    const variants = [qClean];
    if (qClean.length <= 6) variants.push(qClean.toUpperCase());
    const capitalized = qClean.charAt(0).toUpperCase() + qClean.slice(1);
    if (!variants.includes(capitalized)) variants.push(capitalized);

    for (const v of variants) {
      try {
        const wikiUrl = `https://tr.wikipedia.org/api/rest_v1/page/summary/${encodeURIComponent(v)}`;
        const res = await customFetch(wikiUrl, {
          headers: { "User-Agent": "Kepce/1.0 (bilgi@kepce.org)" },
          signal: AbortSignal.timeout(1800),
        });

        if (res.ok) {
          const data = await res.json();
          if (data && data.extract && data.type !== "disambiguation") {
            const raw = {
              title: data.title,
              content: data.extract,
              imgSrc: data.thumbnail?.source || "",
              urls: [{ title: "Vikipedi", url: data.content_urls?.desktop?.page || `https://tr.wikipedia.org/wiki/${encodeURIComponent(data.title)}` }],
              attributes: [],
            };
            const eType = classifyEntity(raw, query);
            const geo = TURKEY_GEO_MAP[normalizeTr(query)] || TURKEY_GEO_MAP[normalizeTr(data.title)] || null;

            return {
              ...raw,
              entityType: eType,
              placeInfo: geo,
              engine: "wikipedia_fallback",
            };
          }
        }
      } catch {}
    }

    // 2. Doğrudan başlık tutmadıysa Wikipedia Arama API'si ile en alakalı sayfayı bul
    const sUrl = `https://tr.wikipedia.org/w/api.php?action=query&list=search&srsearch=${encodeURIComponent(qClean)}&format=json&utf8=1`;
    const sRes = await customFetch(sUrl, {
      headers: { "User-Agent": "Kepce/1.0 (bilgi@kepce.org)" },
      signal: AbortSignal.timeout(2000),
    });
    if (sRes.ok) {
      const sData = await sRes.json();
      const top = sData?.query?.search?.[0];
      if (top && isRelevantInfobox(qClean, top.title)) {
        const sumUrl = `https://tr.wikipedia.org/api/rest_v1/page/summary/${encodeURIComponent(top.title)}`;
        const sumRes = await customFetch(sumUrl, {
          headers: { "User-Agent": "Kepce/1.0 (bilgi@kepce.org)" },
          signal: AbortSignal.timeout(2000),
        });
        if (sumRes.ok) {
          const sumData = await sumRes.json();
          if (sumData && sumData.extract) {
            const raw = {
              title: sumData.title,
              content: sumData.extract,
              imgSrc: sumData.thumbnail?.source || "",
              urls: [{ title: "Vikipedi", url: sumData.content_urls?.desktop?.page || `https://tr.wikipedia.org/wiki/${encodeURIComponent(sumData.title)}` }],
              attributes: [],
            };
            const eType = classifyEntity(raw, query);
            const geo = TURKEY_GEO_MAP[normalizeTr(query)] || TURKEY_GEO_MAP[normalizeTr(sumData.title)] || null;

            return {
              ...raw,
              entityType: eType,
              placeInfo: geo,
              engine: "wikipedia_fallback",
            };
          }
        }
      }
    }
  } catch {
    // gracefully ignore
  }

  return null;
}

// ── Spam ve Çöp Alan Adı Filtresi ───────────────────────────────────────
function isSpamResult(item) {
  if (!item || !item.url) return true;
  const url = item.url.toLowerCase();

  // 1. Şüpheli / bilinen sahte SEO spam TLD'leri
  const SPAM_TLDS = /\.(uy|mo|fk|buzz|top|tk|ml|ga|cf|gq|work|click|country|stream|link|xyz|quest|loan|men|date)(\/|$)/i;
  if (SPAM_TLDS.test(url)) return true;

  // 2. HTTPS olmayan rastgele çöp domainler (resmi .gov.tr, .edu.tr ve wikipedia hariç)
  if (url.startsWith("http://") && !url.includes("gov.tr") && !url.includes("edu.tr") && !url.includes("wikipedia.org")) {
    return true;
  }

  // 3. Başlıkta anlamsız şifreli SEO spam kelime kalıpları
  const title = (item.title || "").toLowerCase();
  if (/\b(hokpegus|pecokfe|galel|rotimer|nampe|hekapo|vesvebvoh|mubgur|cucbi|oziepa|jirwu)\b/i.test(title)) {
    return true;
  }

  return false;
}

// ── SearXNG Asenkron Veri Çekici (Streaming Helper) ──────────────────────
async function fetchSearxData({ effectiveQuery, searxUrl, searchParams, q, instantAnswer, customFetch }) {
  try {
    const res = await customFetch(`${searxUrl.replace(/\/+$/, "")}/search?${searchParams.toString()}`, {
      signal: AbortSignal.timeout(4500),
    });

    if (!res.ok) {
      return {
        results: [],
        infoboxes: [],
        suggestions: [],
        corrections: [],
        answer: instantAnswer,
        numberOfResults: 0,
        error: `Arama servisi yanıt vermedi (${res.status})`,
      };
    }

    const data = await res.json();
    const results = (data.results || [])
      .filter((item) => !isSpamResult(item))
      .map((item) => {
        // Çözünürlük ve en-boy oranını (aspect ratio) önceden çıkar (Layout Shift'i önle)
        let width = null;
        let height = null;
        let aspectRatio = null;
        if (item.resolution) {
          const m = String(item.resolution).match(/(\d+)\s*[xX*×]\s*(\d+)/);
          if (m) {
            width = parseInt(m[1], 10);
            height = parseInt(m[2], 10);
            if (width > 0 && height > 0) {
              aspectRatio = (width / height).toFixed(3);
            }
          }
        }

        // Hızlı ve güvenli CDN önizleme görseli (hotlink ve 403 blokajlarını önler)
        const thumb = item.thumbnail_src || item.thumbnail || item.img_src || "";
        const fullImg = item.img_src || item.thumbnail_src || "";

        return {
          title: item.title || "",
          url: cleanTrackingParams(item.url || ""),
          content: item.content || "",
          imgSrc: fullImg,
          thumbnailSrc: thumb,
          width,
          height,
          aspectRatio,
          publishedDate: item.publishedDate || item.pubdate || null,
          engine: item.engine || (item.engines && item.engines[0]) || "",
          parsedUrl: item.parsed_url || [],
        };
      });

    // Çok kelimeli sorgularda başlık ve açıklamada arama terimlerinin geçme yoğunluğuna göre akıllı sıralama
    const queryTokens = q.toLowerCase().split(/\s+/).filter((t) => t.length > 2);
    if (queryTokens.length > 1) {
      results.sort((a, b) => {
        const scoreResult = (item) => {
          let score = 0;
          const titleLower = (item.title || "").toLowerCase();
          const contentLower = (item.content || "").toLowerCase();
          for (const token of queryTokens) {
            if (titleLower.includes(token)) score += 3;
            if (contentLower.includes(token)) score += 1;
          }
          return score;
        };
        return scoreResult(b) - scoreResult(a);
      });
    }

    const ACRONYM_TITLES = {
      odtu: "Orta Doğu Teknik Üniversitesi (ODTÜ)",
      metu: "Orta Doğu Teknik Üniversitesi (ODTÜ)",
      itu: "İstanbul Teknik Üniversitesi (İTÜ)",
      boun: "Boğaziçi Üniversitesi",
      bogazici: "Boğaziçi Üniversitesi",
      bilkent: "İhsan Doğramacı Bilkent Üniversitesi",
      hacettepe: "Hacettepe Üniversitesi",
      ytu: "Yıldız Teknik Üniversitesi (YTÜ)",
      yildiz: "Yıldız Teknik Üniversitesi (YTÜ)",
    };

    const rawInfoboxes = (data.infoboxes || [])
      .map((box) => {
        let title = box.infobox || box.title || "";
        const qNorm = normalizeTr(q);
        if (ACRONYM_TITLES[qNorm]) {
          title = ACRONYM_TITLES[qNorm];
        }

        const cleanAttrs = (box.attributes || [])
          .map((a) => ({
            label: formatAttrLabel(a.label || ""),
            value: formatAttrValue(a.label || "", a.value || ""),
          }))
          .filter((a) => a.label && a.value);

        // 1. Eğer görsel yoksa Wikidata P18 resmini bul
        let foundImg = box.img_src || box.thumbnail || "";
        if (!foundImg && Array.isArray(box.urls)) {
          const p18 = box.urls.find((u) => u.title === "P18" || (u.url && u.url.includes("Special:FilePath")));
          if (p18 && p18.url) {
            foundImg = p18.url;
          }
        }

        // 2. Eğer koordinat varsa OpenStreetMap url'sinden çıkar
        let osmCoords = null;
        if (Array.isArray(box.urls)) {
          const osm = box.urls.find((u) => (u.url || "").includes("openstreetmap.org"));
          if (osm && osm.url) {
            const mLat = osm.url.match(/lat=([0-9.-]+)/);
            const mLon = osm.url.match(/lon=([0-9.-]+)/);
            if (mLat && mLon) {
              osmCoords = {
                lat: parseFloat(mLat[1]),
                lon: parseFloat(mLon[1]),
                name: title,
                country: "Türkiye",
              };
            }
          }
        }

        const rawBox = {
          title,
          id: box.id || "",
          content: box.content || "",
          imgSrc: foundImg,
          urls: cleanUrls(box.urls || (box.id ? [{ title: "Vikipedi", url: box.id }] : [])),
          attributes: cleanAttrs,
          engine: box.engine || (box.engines && box.engines[0]) || "",
        };

        return {
          ...rawBox,
          entityType: classifyEntity(rawBox, q),
          placeInfo: osmCoords,
        };
      })
      .filter((box) => isRelevantInfobox(q, box));

    let infoboxes = rawInfoboxes;

    // Eğer SearXNG'den infobox gelmediyse Wikipedia REST API Fallback'ini çalıştır (Docker, Kuantum, Kubernetes vb.)
    if (infoboxes.length === 0) {
      try {
        const wikiFallback = await fetchWikipediaFallback(q, customFetch);
        if (wikiFallback && isRelevantInfobox(q, wikiFallback)) {
          infoboxes = [wikiFallback];
        }
      } catch {
        // ignore
      }
    }

    // Fotoğrafı eksik bilgi kartları için alternatif arama motoru görsellerini kontrol et
    if (infoboxes.length > 0 && !infoboxes[0].imgSrc) {
      try {
        const fallbackTarget = infoboxes[0].title || effectiveQuery;
        const fallbackImg = await fetchFallbackImage(fallbackTarget, searxUrl, customFetch);
        if (fallbackImg) {
          infoboxes[0].imgSrc = fallbackImg;
        }
      } catch {
        // gracefully ignore
      }
    }

    // Hava durumu veya coğrafi yer detaylarını bağlama (Yalnızca gerçek yerler ve hava durumu için)
    const isWeatherQuery = q.toLowerCase().includes("hava durumu");
    if (isWeatherQuery || (infoboxes.length > 0 && infoboxes[0].entityType === "place")) {
      try {
        const firstBox = infoboxes[0];
        let locationQuery = firstBox?.title || q;
        if (isWeatherQuery) {
          locationQuery = q.replace(/hava\s*durumu/gi, "").trim() || "Ankara";
        }

        if (firstBox?.placeInfo?.lat && firstBox?.placeInfo?.lon) {
          // Zaten OpenStreetMap / Wikidata koordinatları mevcut
        } else if (firstBox?.entityType === "place" || isWeatherQuery) {
          const placeDetails = await fetchPlaceDetails(locationQuery, customFetch);
          if (placeDetails) {
            if (infoboxes.length > 0) {
              infoboxes[0].placeInfo = placeDetails;
              infoboxes[0].entityType = "place";
            } else if (isWeatherQuery) {
              infoboxes.unshift({
                title: placeDetails.name,
                content: `${placeDetails.name} için güncel hava durumu ve 3 günlük meteoroloji tahmini.`,
                imgSrc: "",
                urls: [],
                attributes: [],
                entityType: "place",
                placeInfo: placeDetails,
              });
            }
          }
        }
      } catch {
        // gracefully ignore
      }
    }

    let resolvedAnswer = instantAnswer;
    if (!resolvedAnswer && data.answers && data.answers.length > 0) {
      const rawAns = data.answers[0];
      let ansContent = typeof rawAns === "string" ? rawAns : (rawAns?.answer || rawAns?.content || "");
      let ansType = rawAns?.type || "generic";
      let ansTitle = rawAns?.title || "Anlık Yanıt";
      const isDateOrTimeAnswer = /\d{1,2}\s+[A-Za-zÇĞİÖŞÜçğıöşü]+\s+\d{4}|\d{2}:\d{2}:\d{2}/.test(ansContent);
      const isExplicitTimeQuery = /saat|time|tarih|date|bugün|gun|gün/.test(q.toLowerCase());
      if (ansContent && (!isDateOrTimeAnswer || isExplicitTimeQuery)) {
        resolvedAnswer = { type: ansType, title: ansTitle, content: ansContent };
      }
    }

    const suggestions = data.suggestions || [];
    const corrections = Array.isArray(data.corrections) ? data.corrections : [];
    const numberOfResults = data.number_of_results || results.length;

    return {
      results,
      infoboxes,
      suggestions,
      corrections,
      answer: resolvedAnswer,
      numberOfResults,
      error: null,
    };
  } catch {
    return {
      results: [],
      infoboxes: [],
      suggestions: [],
      corrections: [],
      answer: instantAnswer,
      numberOfResults: 0,
      error: "Arama servisi şu anda yanıt vermiyor.",
    };
  }
}

export async function load({ url, fetch }) {
  const q = (url.searchParams.get("q") || "").trim();
  const category = url.searchParams.get("kategori") || "general";
  const page = parseInt(url.searchParams.get("sayfa") || "1", 10);
  const language = url.searchParams.get("dil") || "tr";
  const timeRange = url.searchParams.get("zaman") || "";
  const safeSearch = url.searchParams.get("guvenli") || "1";
  const engines = url.searchParams.get("motorlar") || "";

  // Gelişmiş Arama Filtreleri
  const fileType = (url.searchParams.get("dosya") || "").trim().toLowerCase();
  const siteFilter = (url.searchParams.get("site") || "").trim().toLowerCase();
  const verbatim = url.searchParams.get("tam") === "1" || url.searchParams.get("verbatim") === "1";

  // Görsel Filtreleri
  const imgFormat = (url.searchParams.get("format") || "").trim().toLowerCase();
  const imgSize = (url.searchParams.get("boyut") || "").trim().toLowerCase();
  const imgColor = (url.searchParams.get("renk") || "").trim().toLowerCase();
  const imgLicense = (url.searchParams.get("lisans") || "").trim().toLowerCase();

  // Video Filtreleri
  const videoDuration = (url.searchParams.get("sure") || "").trim().toLowerCase();
  const videoQuality = (url.searchParams.get("kalite") || "").trim().toLowerCase();
  const videoPlatform = (url.searchParams.get("platform") || "").trim().toLowerCase();

  // Haber Filtreleri
  const newsSort = (url.searchParams.get("sirala") || "").trim().toLowerCase();

  // Kod & IT Filtreleri
  const codeLang = (url.searchParams.get("dil_prog") || "").trim().toLowerCase();
  const codePlatform = (url.searchParams.get("kaynak") || "").trim().toLowerCase();

  // Akademi Filtreleri
  const scholarAccess = (url.searchParams.get("erisim") || "").trim().toLowerCase();
  const scholarYear = (url.searchParams.get("yil") || "").trim().toLowerCase();

  const filterFields = {
    fileType,
    siteFilter,
    verbatim,
    imgFormat,
    imgSize,
    imgColor,
    imgLicense,
    videoDuration,
    videoQuality,
    videoPlatform,
    newsSort,
    codeLang,
    codePlatform,
    scholarAccess,
    scholarYear,
  };

  if (!q) {
    return {
      isHome: true,
      query: "",
      category: "general",
      page: 1,
      results: [],
      infoboxes: [],
      suggestions: [],
      answer: null,
      kepceCard: null,
      numberOfResults: 0,
      language: "tr",
      timeRange: "",
      safeSearch: "1",
      ...filterFields,
    };
  }

  // 1. !bang Kısayol Yönlendirmesi (!w, !yt, !gh, !so vb.)
  const bangUrl = resolveBang(q);
  if (bangUrl) {
    throw redirect(302, bangUrl);
  }

  // 2. Bellek İçi Önbellek Kontrolü
  const cacheKey = `${q}::${category}::${page}::${language}::${timeRange}::${safeSearch}::${fileType}::${siteFilter}::${verbatim}::${imgFormat}::${imgSize}::${imgColor}::${imgLicense}::${videoDuration}::${videoQuality}::${videoPlatform}::${newsSort}::${codeLang}::${codePlatform}::${scholarAccess}::${scholarYear}`;
  const cached = getCached(cacheKey);
  if (cached && !cached.error && cached.results?.length > 0) {
    return {
      ...cached,
      streamed: {
        searxData: Promise.resolve({
          results: cached.results,
          infoboxes: cached.infoboxes,
          suggestions: cached.suggestions,
          corrections: cached.corrections || [],
          answer: cached.answer,
          numberOfResults: cached.numberOfResults,
        }),
      },
    };
  }

  // 3. Anlık Çözücüler ve Kepçe Niyet Süzgeci (0-5ms içinde çözülür)
  const isGeneralCategory = category === "general" || !category;
  const instantAnswerPromise = isGeneralCategory ? solveInstantQuery(q, fetch) : Promise.resolve(null);
  let kepceCard = isGeneralCategory ? matchKepceIntent(q) : null;

  // Şehir menüsü niyetinde bugünün gerçek menü kalemlerini API'den çekip karta ekle
  if (kepceCard && kepceCard.type === "city_menu" && kepceCard.slug) {
    const today = istanbulToday();
    try {
      const payload = await apiGet(
        `/api/v1/menus?city=${encodeURIComponent(kepceCard.slug)}&date=${today}`,
        { timeout: 1000, fallback: null }
      );
      const menus = normalizeMenuList(payload);
      if (Array.isArray(menus) && menus.length > 0) {
        kepceCard.date = today;
        kepceCard.menus = menus;
      }
    } catch {
      // API yanıt vermezse sessizce temel kart yapısıyla devam et
    }
  }

  // Anlık yanıtı hemen bekle (yalnızca yerel matematik/döviz fonksiyonudur, <5ms)
  const instantAnswer = await instantAnswerPromise;

  // 4. SearXNG Sorgusunu Hazırla ve Filtreleri Akıllıca Uygula
  let effectiveQuery = q.startsWith("!") ? q.replace(/^!+/, "").trim() || q : q;

  // Tam eşleşme (Verbatim)
  if (verbatim && !effectiveQuery.startsWith('"')) {
    effectiveQuery = `"${effectiveQuery}"`;
  }

  // Dosya türü ve Alan Adı (Web & Akademi)
  if (fileType) {
    effectiveQuery += ` filetype:${fileType}`;
  }
  if (siteFilter) {
    effectiveQuery += ` site:${siteFilter}`;
  }

  // Görsel Filtreleri
  if (category === "images") {
    if (imgFormat === "transparent") {
      effectiveQuery += " transparent png";
    } else if (imgFormat === "gif") {
      effectiveQuery += " filetype:gif";
    } else if (imgFormat === "svg") {
      effectiveQuery += " filetype:svg";
    } else if (imgFormat === "jpeg") {
      effectiveQuery += " filetype:jpg";
    }

    if (imgSize === "large") {
      effectiveQuery += " wallpaper";
    } else if (imgSize === "icon") {
      effectiveQuery += " icon";
    }

    if (imgColor === "monochrome") {
      effectiveQuery += " black and white";
    } else if (imgColor === "transparent") {
      effectiveQuery += " transparent";
    }

    if (imgLicense === "cc") {
      effectiveQuery += " creative commons";
    } else if (imgLicense === "commercial") {
      effectiveQuery += " commercial use";
    }
  }

  // Video Filtreleri
  if (category === "videos") {
    if (videoQuality === "hd") {
      effectiveQuery += " HD 1080p";
    }
    if (videoPlatform) {
      effectiveQuery += ` site:${videoPlatform}.com`;
    }
  }

  // Kod Filtreleri
  if (category === "it") {
    if (codeLang) {
      effectiveQuery += ` language:${codeLang}`;
    }
    if (codePlatform) {
      effectiveQuery += ` site:${codePlatform}.com`;
    }
  }

  // Akademi Filtreleri
  if (category === "science") {
    if (scholarAccess === "open") {
      effectiveQuery += " filetype:pdf";
    }
    if (scholarYear) {
      effectiveQuery += ` ${scholarYear}`;
    }
  }

  const searxUrl = env.SEARXNG_URL || "http://localhost:8080";
  const searchParams = new URLSearchParams({
    q: effectiveQuery,
    format: "json",
    categories: category,
    pageno: String(Math.max(1, page)),
    language: language === "all" ? "" : language,
    safesearch: safeSearch,
  });

  if (timeRange) {
    searchParams.set("time_range", timeRange);
  }
  if (engines) {
    searchParams.set("engines", engines);
  }
  if (newsSort === "date") {
    searchParams.set("order", "date");
  }

  // 5. SearXNG Veri Akışı (Streaming Promise)
  const searxPromise = fetchSearxData({
    effectiveQuery,
    searxUrl,
    searchParams,
    q,
    instantAnswer,
    customFetch: fetch,
  }).then((data) => {
    // Yanıt başarılı ve sonuçlu ise önbelleğe kaydet (hataları asla önbelleğe alma)
    if (!data.error && (data.results?.length > 0 || data.answer || data.kepceCard)) {
      setCached(cacheKey, {
        isHome: false,
        query: q,
        category,
        page,
        results: data.results,
        infoboxes: data.infoboxes,
        suggestions: data.suggestions,
        corrections: data.corrections,
        answer: data.answer,
        kepceCard,
        numberOfResults: data.numberOfResults,
        language,
        timeRange,
        safeSearch,
        ...filterFields,
      });
    }
    return data;
  });

  // Sayfayı anında aç (0ms): instantAnswer ve kepceCard anında ekranda!
  return {
    isHome: false,
    query: q,
    category,
    page,
    language,
    timeRange,
    safeSearch,
    ...filterFields,
    answer: instantAnswer,
    kepceCard,
    results: [],
    infoboxes: [],
    suggestions: [],
    corrections: [],
    numberOfResults: 0,
    error: null,
    streamed: {
      searxData: searxPromise,
    },
  };
}

