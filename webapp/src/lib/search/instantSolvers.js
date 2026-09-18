// Kepçe Ara — Birim Dönüştürücü, Dünya Saatleri, TDK Sözlük ve Kripto Takip Çözücü
import { CITY_MAP } from "@/utils/turkish.js";

// ── 1. Birim Dönüşüm Tablosu (Base Units) ───────────────────────────────────
const UNIT_CATEGORIES = {
  length: {
    name: "Uzunluk",
    base: "m",
    units: {
      km: { name: "Kilometre", factor: 1000, aliases: ["km", "kilometre", "kilometres"] },
      m: { name: "Metre", factor: 1, aliases: ["m", "metre", "meter", "meters"] },
      cm: { name: "Santimetre", factor: 0.01, aliases: ["cm", "santimetre", "centimeter", "santim", "sm"] },
      mm: { name: "Milimetre", factor: 0.001, aliases: ["mm", "milimetre", "millimeter", "milim"] },
      mi: { name: "Mil", factor: 1609.344, aliases: ["mil", "mile", "miles", "mi"] },
      in: { name: "İnç", factor: 0.0254, aliases: ["inç", "inc", "inch", "inches", "in"] },
      ft: { name: "Fit", factor: 0.3048, aliases: ["fit", "foot", "feet", "ft"] },
      yd: { name: "Yarda", factor: 0.9144, aliases: ["yarda", "yard", "yards", "yd"] },
    },
  },
  mass: {
    name: "Kütle / Ağırlık",
    base: "kg",
    units: {
      ton: { name: "Metrik Ton", factor: 1000, aliases: ["ton", "tonnes"] },
      kg: { name: "Kilogram", factor: 1, aliases: ["kg", "kilogram", "kilo", "kilograms"] },
      g: { name: "Gram", factor: 0.001, aliases: ["g", "gram", "grams", "gr"] },
      mg: { name: "Miligram", factor: 0.000001, aliases: ["mg", "miligram", "milligram"] },
      lb: { name: "Pound (Libre)", factor: 0.45359237, aliases: ["pound", "libre", "lb", "lbs"] },
      oz: { name: "Ons", factor: 0.028349523, aliases: ["ons", "oz", "ounce", "ounces"] },
    },
  },
  digital: {
    name: "Dijital Veri",
    base: "b",
    units: {
      b: { name: "Byte", factor: 1, aliases: ["b", "byte", "bayt"] },
      kb: { name: "Kilobyte", factor: 1024, aliases: ["kb", "kilobyte"] },
      mb: { name: "Megabyte", factor: 1024 * 1024, aliases: ["mb", "megabyte"] },
      gb: { name: "Gigabyte", factor: 1024 * 1024 * 1024, aliases: ["gb", "gigabyte"] },
      tb: { name: "Terabyte", factor: 1024 * 1024 * 1024 * 1024, aliases: ["tb", "terabyte"] },
    },
  },
};

export function levenshteinDistance(a, b) {
  const m = a.length;
  const n = b.length;
  const d = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));
  for (let i = 0; i <= m; i += 1) d[i][0] = i;
  for (let j = 0; j <= n; j += 1) d[0][j] = j;
  for (let i = 1; i <= m; i += 1) {
    for (let j = 1; j <= n; j += 1) {
      const cost = a[i - 1] === b[j - 1] ? 0 : 1;
      d[i][j] = Math.min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + cost);
    }
  }
  return d[m][n];
}

const KEYBOARD_ADJACENT = {
  k: ["j", "l", "i", "o", "m"],
  l: ["k", "o", "p", "ş"],
  c: ["x", "v", "d", "f"],
  s: ["a", "d", "w"],
};

export function findClosestUnit(typoStr, categoryKey, excludeCode) {
  const s = typoStr.toLowerCase().trim();
  const cat = UNIT_CATEGORIES[categoryKey];
  if (!cat) return null;

  let best = null;
  let bestScore = Infinity;

  for (const [uKey, u] of Object.entries(cat.units)) {
    if (uKey === excludeCode) continue;
    for (const alias of u.aliases) {
      const dist = levenshteinDistance(s, alias);
      const maxAllowed = Math.max(1, Math.floor(alias.length / 2));
      if (dist <= maxAllowed) {
        let score = dist * 10;
        if (s.length === alias.length && dist === 1) {
          for (let i = 0; i < s.length; i += 1) {
            if (s[i] !== alias[i]) {
              const adj = KEYBOARD_ADJACENT[s[i]];
              if (adj && adj.includes(alias[i])) {
                score -= 3;
              }
            }
          }
        }
        if (uKey === cat.base) score -= 1;

        if (score < bestScore) {
          bestScore = score;
          best = { category: categoryKey, code: uKey, ...u };
        }
      }
    }
  }
  return best;
}

export function findUnit(str) {
  const s = str.toLowerCase().trim();
  for (const [catKey, cat] of Object.entries(UNIT_CATEGORIES)) {
    for (const [uKey, u] of Object.entries(cat.units)) {
      if (u.aliases.includes(s) || uKey === s) {
        return { category: catKey, code: uKey, ...u };
      }
    }
  }
  return null;
}

export function solveUnitConversion(query) {
  const q = query.trim().toLowerCase();

  const m = q.match(/^(\d+(?:[.,]\d+)?\s*)?([a-zğüşıöç°]+)\s*(?:ka[cç]\s*([a-zğüşıöç°]+)|to\s*([a-zğüşıöç°]+)|in\s*([a-zğüşıöç°]+))$/i);
  if (!m) return null;

  const amount = parseFloat((m[1] || "1").replace(",", ".")) || 1;
  const fromRaw = (m[2] || "").trim();
  const toRaw = (m[3] || m[4] || m[5] || "").trim();

  // Sıcaklık Özel Kontrolü
  const tempAliases = {
    c: ["c", "°c", "celsius", "santigrat", "derece"],
    f: ["f", "°f", "fahrenheit", "fahrenhayt"],
    k: ["k", "kelvin"],
  };

  const isTempFrom = Object.entries(tempAliases).find(([_, aliases]) => aliases.includes(fromRaw));
  const isTempTo = Object.entries(tempAliases).find(([_, aliases]) => aliases.includes(toRaw));

  if (isTempFrom && isTempTo && isTempFrom[0] !== isTempTo[0]) {
    const fromCode = isTempFrom[0];
    const toCode = isTempTo[0];
    let res = amount;

    let inC = amount;
    if (fromCode === "f") inC = (amount - 32) * (5 / 9);
    else if (fromCode === "k") inC = amount - 273.15;

    if (toCode === "c") res = inC;
    else if (toCode === "f") res = inC * (9 / 5) + 32;
    else if (toCode === "k") res = inC + 273.15;

    const names = { c: "Santigrat (°C)", f: "Fahrenheit (°F)", k: "Kelvin (K)" };

    return {
      type: "unit",
      categoryName: "Sıcaklık",
      fromAmount: amount,
      fromUnit: fromCode.toUpperCase(),
      fromUnitName: names[fromCode],
      toAmount: parseFloat(res.toFixed(4)),
      toUnit: toCode.toUpperCase(),
      toUnitName: names[toCode],
      formula: `${amount} ${names[fromCode]} = ${parseFloat(res.toFixed(4))} ${names[toCode]}`,
    };
  }

  // Standart Birimler
  const fromUnit = findUnit(fromRaw);
  const toUnit = findUnit(toRaw);

  if (fromUnit && toUnit && fromUnit.category === toUnit.category) {
    const inBase = amount * fromUnit.factor;
    const result = inBase / toUnit.factor;
    const catName = UNIT_CATEGORIES[fromUnit.category].name;

    return {
      type: "unit",
      categoryName: catName,
      fromAmount: amount,
      fromUnit: fromUnit.code,
      fromUnitName: fromUnit.name,
      toAmount: parseFloat(result.toFixed(6)),
      toUnit: toUnit.code,
      toUnitName: toUnit.name,
      formula: `${amount} ${fromUnit.name} = ${parseFloat(result.toFixed(6))} ${toUnit.name}`,
    };
  }

  return null;
}

export function suggestUnitCorrection(query) {
  if (!query) return null;
  const q = query.trim().toLowerCase();
  const m = q.match(/^(\d+(?:[.,]\d+)?\s*)?([a-zğüşıöç°]+)\s*(ka[cç]|to|in)\s*([a-zğüşıöç°]+)$/i);
  if (!m) return null;

  const amountStr = m[1] || "";
  const fromRaw = (m[2] || "").trim();
  const op = m[3];
  const toRaw = (m[4] || "").trim();

  // Halihazırda geçerli bir birim dönüşümüyse düzeltmeye gerek yok
  if (solveUnitConversion(query)) return null;

  const fromUnit = findUnit(fromRaw);
  const toUnit = findUnit(toRaw);

  // 1. Durum: Kaynak birim geçerli, hedef birimde yazım hatası var (örn: "50 g kaç lg" -> "50 g kaç kg")
  if (fromUnit && !toUnit) {
    const closest = findClosestUnit(toRaw, fromUnit.category, fromUnit.code);
    if (closest) {
      const correctedQuery = `${amountStr}${fromRaw} ${op} ${closest.code}`.trim();
      const solved = solveUnitConversion(correctedQuery);
      return {
        originalQuery: query,
        correctedQuery,
        typo: toRaw,
        suggested: closest.code,
        suggestedName: closest.name,
        solved,
      };
    }
  }

  // 2. Durum: Hedef birim geçerli, kaynak birimde yazım hatası var (örn: "50 lg kaç g" -> "50 kg kaç g")
  if (!fromUnit && toUnit) {
    const closest = findClosestUnit(fromRaw, toUnit.category, toUnit.code);
    if (closest) {
      const correctedQuery = `${amountStr}${closest.code} ${op} ${toRaw}`.trim();
      const solved = solveUnitConversion(correctedQuery);
      return {
        originalQuery: query,
        correctedQuery,
        typo: fromRaw,
        suggested: closest.code,
        suggestedName: closest.name,
        solved,
      };
    }
  }

  return null;
}

// ── 2. Dünya Saatleri & Zaman Dilimleri (Dinamik IANA + Yerel 81 İl) ────────
const IANA_ZONES = typeof Intl !== "undefined" && typeof Intl.supportedValuesOf === "function"
  ? Intl.supportedValuesOf("timeZone")
  : [];

const DYNAMIC_IANA_MAP = new Map();
for (const tz of IANA_ZONES) {
  const parts = tz.split("/");
  const cityName = parts[parts.length - 1].replace(/_/g, " ").toLowerCase();
  DYNAMIC_IANA_MAP.set(cityName, tz);
}

// IANA standart İngilizce lokasyon adlarından farklı olan Türkçe şehir ve ülke eşleştirmeleri
const TURKISH_WORLD_ALIASES = {
  // Şehirler
  atina: { tz: "Europe/Athens", city: "Atina", country: "Yunanistan" },
  viyana: { tz: "Europe/Vienna", city: "Viyana", country: "Avusturya" },
  kahire: { tz: "Africa/Cairo", city: "Kahire", country: "Mısır" },
  roma: { tz: "Europe/Rome", city: "Roma", country: "İtalya" },
  londra: { tz: "Europe/London", city: "Londra", country: "Birleşik Krallık" },
  moskova: { tz: "Europe/Moscow", city: "Moskova", country: "Rusya" },
  pekin: { tz: "Asia/Shanghai", city: "Pekin", country: "Çin" },
  sidney: { tz: "Australia/Sydney", city: "Sidney", country: "Avustralya" },
  lizbon: { tz: "Europe/Lisbon", city: "Lizbon", country: "Portekiz" },
  brüksel: { tz: "Europe/Brussels", city: "Brüksel", country: "Belçika" },
  bruksel: { tz: "Europe/Brussels", city: "Brüksel", country: "Belçika" },
  varşova: { tz: "Europe/Warsaw", city: "Varşova", country: "Polonya" },
  varsova: { tz: "Europe/Warsaw", city: "Varşova", country: "Polonya" },
  prag: { tz: "Europe/Prague", city: "Prag", country: "Çekya" },
  tahran: { tz: "Asia/Tehran", city: "Tahran", country: "İran" },
  bağdat: { tz: "Asia/Baghdad", city: "Bağdat", country: "Irak" },
  bagdat: { tz: "Asia/Baghdad", city: "Bağdat", country: "Irak" },
  şam: { tz: "Asia/Damascus", city: "Şam", country: "Suriye" },
  sam: { tz: "Asia/Damascus", city: "Şam", country: "Suriye" },
  riyad: { tz: "Asia/Riyadh", city: "Riyad", country: "Suudi Arabistan" },
  kudüs: { tz: "Asia/Jerusalem", city: "Kudüs", country: "Filistin / İsrail" },
  kudus: { tz: "Asia/Jerusalem", city: "Kudüs", country: "Filistin / İsrail" },
  beyrut: { tz: "Asia/Beirut", city: "Beyrut", country: "Lübnan" },
  bakü: { tz: "Asia/Baku", city: "Bakü", country: "Azerbaycan" },
  baku: { tz: "Asia/Baku", city: "Bakü", country: "Azerbaycan" },
  tiflis: { tz: "Asia/Tbilisi", city: "Tiflis", country: "Gürcistan" },
  saraybosna: { tz: "Europe/Sarajevo", city: "Saraybosna", country: "Bosna-Hersek" },
  üsküp: { tz: "Europe/Skopje", city: "Üsküp", country: "Kuzey Makedonya" },
  uskup: { tz: "Europe/Skopje", city: "Üsküp", country: "Kuzey Makedonya" },
  belgrad: { tz: "Europe/Belgrade", city: "Belgrad", country: "Sırbistan" },
  bükreş: { tz: "Europe/Bucharest", city: "Bükreş", country: "Romanya" },
  bukres: { tz: "Europe/Bucharest", city: "Bükreş", country: "Romanya" },
  sofya: { tz: "Europe/Sofia", city: "Sofya", country: "Bulgaristan" },
  kopenhag: { tz: "Europe/Copenhagen", city: "Kopenhag", country: "Danimarka" },
  münih: { tz: "Europe/Berlin", city: "Münih", country: "Almanya" },
  munih: { tz: "Europe/Berlin", city: "Münih", country: "Almanya" },
  seul: { tz: "Asia/Seoul", city: "Seul", country: "Güney Kore" },
  tokyo: { tz: "Asia/Tokyo", city: "Tokyo", country: "Japonya" },
  berlin: { tz: "Europe/Berlin", city: "Berlin", country: "Almanya" },
  paris: { tz: "Europe/Paris", city: "Paris", country: "Fransa" },
  madrid: { tz: "Europe/Madrid", city: "Madrid", country: "İspanya" },
  amsterdam: { tz: "Europe/Amsterdam", city: "Amsterdam", country: "Hollanda" },
  "new york": { tz: "America/New_York", city: "New York", country: "ABD" },
  newyork: { tz: "America/New_York", city: "New York", country: "ABD" },
  "los angeles": { tz: "America/Los_Angeles", city: "Los Angeles", country: "ABD" },
  toronto: { tz: "America/Toronto", city: "Toronto", country: "Kanada" },
  dubai: { tz: "Asia/Dubai", city: "Dubai", country: "BAE" },
  şanghay: { tz: "Asia/Shanghai", city: "Şanghay", country: "Çin" },
  sanghay: { tz: "Asia/Shanghai", city: "Şanghay", country: "Çin" },

  // Popüler Ülkeler
  türkiye: { tz: "Europe/Istanbul", city: "İstanbul", country: "Türkiye" },
  turkiye: { tz: "Europe/Istanbul", city: "İstanbul", country: "Türkiye" },
  almanya: { tz: "Europe/Berlin", city: "Berlin", country: "Almanya" },
  ingiltere: { tz: "Europe/London", city: "Londra", country: "Birleşik Krallık" },
  fransa: { tz: "Europe/Paris", city: "Paris", country: "Fransa" },
  italya: { tz: "Europe/Rome", city: "Roma", country: "İtalya" },
  ispanya: { tz: "Europe/Madrid", city: "Madrid", country: "İspanya" },
  rusya: { tz: "Europe/Moscow", city: "Moskova", country: "Rusya" },
  japonya: { tz: "Asia/Tokyo", city: "Tokyo", country: "Japonya" },
  abd: { tz: "America/New_York", city: "New York", country: "ABD" },
  amerika: { tz: "America/New_York", city: "New York", country: "ABD" },
  çin: { tz: "Asia/Shanghai", city: "Pekin", country: "Çin" },
  cin: { tz: "Asia/Shanghai", city: "Pekin", country: "Çin" },
  mısır: { tz: "Africa/Cairo", city: "Kahire", country: "Mısır" },
  misir: { tz: "Africa/Cairo", city: "Kahire", country: "Mısır" },
  yunanistan: { tz: "Europe/Athens", city: "Atina", country: "Yunanistan" },
  avusturya: { tz: "Europe/Vienna", city: "Viyana", country: "Avusturya" },
  hollanda: { tz: "Europe/Amsterdam", city: "Amsterdam", country: "Hollanda" },
  isviçre: { tz: "Europe/Zurich", city: "Zürih", country: "İsviçre" },
  isvicre: { tz: "Europe/Zurich", city: "Zürih", country: "İsviçre" },
  azerbaycan: { tz: "Asia/Baku", city: "Bakü", country: "Azerbaycan" },
  güneykore: { tz: "Asia/Seoul", city: "Seul", country: "Güney Kore" },
  kore: { tz: "Asia/Seoul", city: "Seul", country: "Güney Kore" },
};

export function resolveTimezoneInfo(placeStr) {
  if (!placeStr) return null;
  const p = placeStr.trim().toLowerCase();
  const pNorm = p.replace(/['’]?(?:d[ae]|t[ae]|n?da|n?de)$/, "").trim();

  // 1. Türkiye genel veya 81 il kontrolü (CITY_MAP)
  if (pNorm === "türkiye" || pNorm === "turkiye") {
    return { city: "İstanbul", country: "Türkiye", timezone: "Europe/Istanbul" };
  }
  if (CITY_MAP[pNorm]) {
    return { city: CITY_MAP[pNorm], country: "Türkiye", timezone: "Europe/Istanbul" };
  }

  // 2. Türkçe şehir/ülke alias haritası
  if (TURKISH_WORLD_ALIASES[pNorm]) {
    const a = TURKISH_WORLD_ALIASES[pNorm];
    return {
      city: a.city,
      country: a.country,
      timezone: a.tz,
    };
  }

  // 3. Doğrudan 418 IANA lokasyon eşleşmesi (örn: "seoul", "vienna", "cairo", "chicago", "tokyo", "sydney")
  const directTz = DYNAMIC_IANA_MAP.get(pNorm);
  if (directTz) {
    const parts = directTz.split("/");
    const cityName = parts[parts.length - 1].replace(/_/g, " ");
    const regionName = parts[0];
    return {
      city: cityName,
      country: regionName,
      timezone: directTz,
    };
  }

  return null;
}

function formatTimeAnswer(info, now = new Date()) {
  try {
    const timeFormatter = new Intl.DateTimeFormat("tr-TR", {
      timeZone: info.timezone,
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: false,
    });

    const dateFormatter = new Intl.DateTimeFormat("tr-TR", {
      timeZone: info.timezone,
      weekday: "long",
      year: "numeric",
      month: "long",
      day: "numeric",
    });

    // Gece yarısı veya gün değişimlerinde saat farkını dakika üzerinden net hesapla
    const getTzOffsetMinutes = (tz) => {
      const utcDate = new Date(now.toLocaleString("en-US", { timeZone: "UTC" }));
      const tzDate = new Date(now.toLocaleString("en-US", { timeZone: tz }));
      return Math.round((tzDate.getTime() - utcDate.getTime()) / 60000);
    };

    const trOffset = getTzOffsetMinutes("Europe/Istanbul");
    const targetOffset = getTzOffsetMinutes(info.timezone);
    const diffMinutes = targetOffset - trOffset;
    const diffHours = diffMinutes / 60;

    let diffText = "Türkiye ile aynı saat diliminde";
    if (diffHours > 0) {
      diffText = `Türkiye'den ${diffHours % 1 === 0 ? diffHours : diffHours.toFixed(1)} saat ileri`;
    } else if (diffHours < 0) {
      const absDiff = Math.abs(diffHours);
      diffText = `Türkiye'den ${absDiff % 1 === 0 ? absDiff : absDiff.toFixed(1)} saat geri`;
    }

    return {
      type: "time",
      city: info.city,
      country: info.country,
      timezone: info.timezone,
      currentTime: timeFormatter.format(now),
      currentDate: dateFormatter.format(now),
      diffText,
    };
  } catch {
    return null;
  }
}

export function solveWorldTime(query) {
  if (!query) return null;
  const q = query.trim().toLowerCase().replace(/[?.,!]+$/, "");

  // 1. Genel saat sorgusu (Şehir veya ülke belirtilmemişse Türkiye yerel saati gösterilir)
  // "saat kaç", "saat kac", "şu an saat kaç", "suan saat kac", "saat"
  if (/^(?:(?:şu\s*an\s*)?saat\s*ka[cç]|şu\s*an\s*saat|saat)$/i.test(q)) {
    return formatTimeAnswer({ city: "İstanbul", country: "Türkiye", timezone: "Europe/Istanbul" });
  }

  // 2. Şehir / ülke saati: "tokyo saati", "tokyo'da saat kaç", "londra saat", "ankara saati", "trabzon saati"
  const m1 = q.match(/^(?:([a-zğüşıöç\s]+?)(?:'d[ae]|'t[ae]|'n?da|'n?de|'de|'da)?\s*(?:saat\s*ka[cç]|saati|saat))$/i);
  if (m1) {
    const placeRaw = m1[1].trim();
    const info = resolveTimezoneInfo(placeRaw);
    if (info) return formatTimeAnswer(info);
  }

  // 3. Ters sıralı yer sorgusu: "saat tokyo", "saat kaç londra", "saat kahire"
  const m2 = q.match(/^(?:saat\s*ka[cç]\s+|saat\s+)([a-zğüşıöç\s]+)$/i);
  if (m2) {
    const placeRaw = m2[1].trim();
    const info = resolveTimezoneInfo(placeRaw);
    if (info) return formatTimeAnswer(info);
  }

  return null;
}

const TDK_WORD_ALIASES = {
  tabldot: "tabildot",
  tabldöt: "tabildot",
  vejeteryan: "vejetaryen",
  orjinal: "orijinal",
  laboratuar: "laboratuvar",
  karnıbahar: "karnabahar",
  yalnış: "yanlış",
  yanlız: "yalnız",
  traş: "tıraş",
  egsoz: "egzoz",
  egzozt: "egzoz",
  makina: "makine",
};

// ── 3. TDK Sözlük & Tanım Çözücü ───────────────────────────────────────────
export async function solveTdkDefinition(query) {
  const q = query.trim().toLowerCase();
  // 1. Sonek kalıpları: "merak tanım", "merak tanımı", "merak nedir", "merak ne demek", "merak anlamı"
  let m = q.match(/^([a-zğüşıöç]+)\s*(?:nedir|ne\s*demek|tan[ıi]m[ıi]?|anlam[ıi]?|manas[ıi]|kelimesi\s*nedir)$/i);
  // 2. Önek kalıpları: "tanım merak", "tanımı merak", "anlamı merak"
  if (!m) {
    const mPre = q.match(/^(?:tan[ıi]m[ıi]?|anlam[ıi]?|manas[ıi])\s+([a-zğüşıöç]+)$/i);
    if (mPre) {
      m = [mPre[0], mPre[1]];
    }
  }
  if (!m) return null;

  const rawWord = m[1].trim();
  if (rawWord.length < 2) return null;

  const lookupWord = TDK_WORD_ALIASES[rawWord] || rawWord;

  try {
    const res = await fetch(`https://sozluk.gov.tr/gts?ara=${encodeURIComponent(lookupWord)}`, {
      signal: AbortSignal.timeout(3000),
    });
    if (!res.ok) return null;

    const data = await res.json();
    if (Array.isArray(data) && data.length > 0 && data[0].anlamlarListe && data[0].anlamlarListe.length > 0) {
      const first = data[0];
      const meanings = first.anlamlarListe.slice(0, 3).map((a, idx) => ({
        index: idx + 1,
        meaning: a.anlam,
        example: a.orneklerListe?.[0]?.ornek || null,
        author: a.orneklerListe?.[0]?.yazar?.[0]?.tam_adi || null,
      }));

      return {
        type: "definition",
        word: first.madde || lookupWord,
        meanings,
        source: "Türk Dil Kurumu Güncel Türkçe Sözlük",
      };
    }
  } catch {
    // TDK servis kesintilerinde veya timeout durumunda arama akışını kesmeden sessizce devam et.
  }

  return null;
}

// ── 4. Canlı Kripto Para Çözücü ───────────────────────────────────────────
const CRYPTO_COINS = {
  btc: { id: "bitcoin", name: "Bitcoin (BTC)", symbol: "BTC" },
  bitcoin: { id: "bitcoin", name: "Bitcoin (BTC)", symbol: "BTC" },
  eth: { id: "ethereum", name: "Ethereum (ETH)", symbol: "ETH" },
  ethereum: { id: "ethereum", name: "Ethereum (ETH)", symbol: "ETH" },
  sol: { id: "solana", name: "Solana (SOL)", symbol: "SOL" },
  solana: { id: "solana", name: "Solana (SOL)", symbol: "SOL" },
  xrp: { id: "ripple", name: "XRP (Ripple)", symbol: "XRP" },
  ripple: { id: "ripple", name: "XRP (Ripple)", symbol: "XRP" },
  doge: { id: "dogecoin", name: "Dogecoin (DOGE)", symbol: "DOGE" },
  dogecoin: { id: "dogecoin", name: "Dogecoin (DOGE)", symbol: "DOGE" },
  avax: { id: "avalanche-2", name: "Avalanche (AVAX)", symbol: "AVAX" },
};

let cryptoCache = {
  ts: 0,
  data: {},
};

export async function solveCryptoPrice(query) {
  const q = query.trim().toLowerCase();
  const m = q.match(/^([a-z0-9]+)\s*(?:ka[cç]\s*([a-z$€£₺]+)|fiyat[ıi]|kuru|to\s*([a-z$€£₺]+))$/i);
  if (!m) return null;

  const rawCoin = m[1];
  const rawVs = (m[2] || m[3] || "tl").toLowerCase();

  const coin = CRYPTO_COINS[rawCoin];
  if (!coin) return null;

  let vsCurrency = "try";
  let symbol = "₺";
  if (rawVs === "dolar" || rawVs === "usd" || rawVs === "$") {
    vsCurrency = "usd";
    symbol = "$";
  } else if (rawVs === "euro" || rawVs === "eur" || rawVs === "€") {
    vsCurrency = "eur";
    symbol = "€";
  }

  try {
    const now = Date.now();
    if (now - cryptoCache.ts > 60 * 1000 || !cryptoCache.data[coin.id]) {
      const res = await fetch(`https://api.coingecko.com/api/v3/simple/price?ids=bitcoin,ethereum,solana,ripple,dogecoin,avalanche-2&vs_currencies=try,usd,eur&include_24hr_change=true`, {
        signal: AbortSignal.timeout(3000),
      });
      if (res.ok) {
        cryptoCache = {
          ts: now,
          data: await res.json(),
        };
      }
    }

    const entry = cryptoCache.data[coin.id];
    if (entry && entry[vsCurrency]) {
      const price = entry[vsCurrency];
      const change = entry[`${vsCurrency}_24h_change`];

      return {
        type: "crypto",
        name: coin.name,
        symbol: coin.symbol,
        price,
        formattedPrice: `${price.toLocaleString("tr-TR", { maximumFractionDigits: 2 })} ${symbol}`,
        currency: vsCurrency.toUpperCase(),
        change24h: change ? parseFloat(change.toFixed(2)) : null,
      };
    }
  } catch {
    // CoinGecko API kesintisinde veya rate-limit durumunda arama akışını kesmeden sessizce devam et.
  }

  return null;
}
