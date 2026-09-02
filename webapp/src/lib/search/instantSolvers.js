// Kepçe Ara — Birim Dönüştürücü, Dünya Saatleri, TDK Sözlük ve Kripto Takip Çözücü

// ── 1. Birim Dönüşüm Tablosu (Base Units) ───────────────────────────────────
const UNIT_CATEGORIES = {
  length: {
    name: "Uzunluk",
    base: "m",
    units: {
      km: { name: "Kilometre", factor: 1000, aliases: ["km", "kilometre", "kilometres"] },
      m: { name: "Metre", factor: 1, aliases: ["m", "metre", "meter", "meters"] },
      cm: { name: "Santimetre", factor: 0.01, aliases: ["cm", "santimetre", "centimeter"] },
      mm: { name: "Milimetre", factor: 0.001, aliases: ["mm", "milimetre", "millimeter"] },
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

function findUnit(str) {
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

// ── 2. Dünya Saatleri & Zaman Dilimleri ──────────────────────────────────────
const WORLD_CITIES = {
  tokyo: { city: "Tokyo", country: "Japonya", timezone: "Asia/Tokyo" },
  londra: { city: "Londra", country: "Birleşik Krallık", timezone: "Europe/London" },
  london: { city: "Londra", country: "Birleşik Krallık", timezone: "Europe/London" },
  "new york": { city: "New York", country: "ABD", timezone: "America/New_York" },
  newyork: { city: "New York", country: "ABD", timezone: "America/New_York" },
  paris: { city: "Paris", country: "Fransa", timezone: "Europe/Paris" },
  berlin: { city: "Berlin", country: "Almanya", timezone: "Europe/Berlin" },
  moskova: { city: "Moskova", country: "Rusya", timezone: "Europe/Moscow" },
  moscow: { city: "Moskova", country: "Rusya", timezone: "Europe/Moscow" },
  pekin: { city: "Pekin", country: "Çin", timezone: "Asia/Shanghai" },
  beijing: { city: "Pekin", country: "Çin", timezone: "Asia/Shanghai" },
  sidney: { city: "Sidney", country: "Avustralya", timezone: "Australia/Sydney" },
  sydney: { city: "Sidney", country: "Avustralya", timezone: "Australia/Sydney" },
  dubai: { city: "Dubai", country: "BAE", timezone: "Asia/Dubai" },
  roma: { city: "Roma", country: "İtalya", timezone: "Europe/Rome" },
  rome: { city: "Roma", country: "İtalya", timezone: "Europe/Rome" },
  madrid: { city: "Madrid", country: "İspanya", timezone: "Europe/Madrid" },
  amsterdam: { city: "Amsterdam", country: "Hollanda", timezone: "Europe/Amsterdam" },
  toronto: { city: "Toronto", country: "Kanada", timezone: "America/Toronto" },
  "los angeles": { city: "Los Angeles", country: "ABD", timezone: "America/Los_Angeles" },
  baku: { city: "Bakü", country: "Azerbaycan", timezone: "Asia/Baku" },
  bakü: { city: "Bakü", country: "Azerbaycan", timezone: "Asia/Baku" },
  almanya: { city: "Berlin", country: "Almanya", timezone: "Europe/Berlin" },
  japonya: { city: "Tokyo", country: "Japonya", timezone: "Asia/Tokyo" },
  ingiltere: { city: "Londra", country: "Birleşik Krallık", timezone: "Europe/London" },
  fransa: { city: "Paris", country: "Fransa", timezone: "Europe/Paris" },
  rusya: { city: "Moskova", country: "Rusya", timezone: "Europe/Moscow" },
  abd: { city: "Washington D.C.", country: "ABD", timezone: "America/New_York" },
  amerika: { city: "Washington D.C.", country: "ABD", timezone: "America/New_York" },
};

export function solveWorldTime(query) {
  const q = query.trim().toLowerCase();

  const timeRegex = /^(?:([a-zğüşıöç\s]+?)(?:'d[ae]|'t[ae]|'n?da|'n?de|'de|'da)?\s*(?:saat\s*ka[cç]|saati|saat))$/i;
  const m = q.match(timeRegex);
  if (!m) return null;

  const placeRaw = m[1].trim();
  const info = WORLD_CITIES[placeRaw];
  if (!info) return null;

  try {
    const now = new Date();

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

    const targetHour = parseInt(new Intl.DateTimeFormat("en-US", { timeZone: info.timezone, hour: "numeric", hour12: false }).format(now), 10);
    const trHour = parseInt(new Intl.DateTimeFormat("en-US", { timeZone: "Europe/Istanbul", hour: "numeric", hour12: false }).format(now), 10);
    const diff = targetHour - trHour;

    let diffText = "Türkiye ile aynı saat diliminde";
    if (diff > 0) {
      diffText = `Türkiye'den ${diff} saat ileri`;
    } else if (diff < 0) {
      diffText = `Türkiye'den ${Math.abs(diff)} saat geri`;
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

// ── 3. TDK Sözlük & Tanım Çözücü ───────────────────────────────────────────
export async function solveTdkDefinition(query) {
  const q = query.trim().toLowerCase();
  const m = q.match(/^([a-zğüşıöç]+)\s*(?:nedir|ne\s*demek|anlam[ıi])$/i);
  if (!m) return null;

  const word = m[1].trim();
  if (word.length < 2) return null;

  try {
    const res = await fetch(`https://sozluk.gov.tr/gts?ara=${encodeURIComponent(word)}`, {
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
        word: first.madde || word,
        meanings,
        source: "Türk Dil Kurumu Güncel Türkçe Sözlük",
      };
    }
  } catch {
    // gracefully ignore
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
    // gracefully ignore
  }

  return null;
}
