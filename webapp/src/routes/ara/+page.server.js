import { env } from "$env/dynamic/private";

// Open-Meteo ve Coğrafi Konum Servisi (Nominatim)
async function fetchPlaceDetails(query) {
  try {
    const geoUrl = `https://nominatim.openstreetmap.org/search?q=${encodeURIComponent(query)}&format=json&limit=1&addressdetails=1`;
    const geoRes = await fetch(geoUrl, {
      headers: { "User-Agent": "Kepce/1.0 (bilgi@kepce.org)" },
      signal: AbortSignal.timeout(3000),
    });

    if (!geoRes.ok) return null;
    const geoData = await geoRes.json();
    if (!geoData || geoData.length === 0) return null;

    const top = geoData[0];
    const lat = parseFloat(top.lat);
    const lon = parseFloat(top.lon);
    const displayName = top.name || top.display_name.split(",")[0];
    const country = top.address?.country || "";

    // Open-Meteo ile güncel ve 3 günlük hava tahmini
    const weatherUrl = `https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&current=temperature_2m,weather_code&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto&forecast_days=3`;
    const wRes = await fetch(weatherUrl, { signal: AbortSignal.timeout(3000) });

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

// Sorgu-Başlık Uyum Filtresi (Relevance Gate)
function isRelevantInfobox(query, title) {
  if (!query || !title) return false;
  const qNorm = normalizeTr(query);
  const tNorm = normalizeTr(title);

  if (tNorm.includes(qNorm) || qNorm.includes(tNorm)) return true;

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

  // Alan için km² ekleme
  const labelLower = (label || "").toLowerCase();
  if ((labelLower.includes("alan") || labelLower.includes("yüzölçüm")) && /^\d+(?:[.,]\d+)?$/.test(str)) {
    return `${str} km²`;
  }

  return str;
}

// Varlık Tipi Sınıflandırıcısı
function classifyEntity(infobox, query) {
  const text = `${infobox.title} ${infobox.content || ""}`.toLowerCase();
  const labels = (infobox.attributes || []).map((a) => a.label.toLowerCase());

  // 1. Kişi / Biyografi (Person) Kontrolü
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
  ];
  if (personKeywords.some((k) => text.includes(k))) {
    return "person";
  }

  // 2. Kurum / Üniversite (Organization) Kontrolü
  const orgLabels = [
    "rektör",
    "genel merkez",
    "ceo",
    "kuruluş tarihi",
    "yönetim kurulu başkanı",
    "çalışan sayısı",
  ];
  if (labels.some((l) => orgLabels.includes(l))) {
    return "organization";
  }
  if (
    text.includes("üniversite") ||
    text.includes("enstitü") ||
    text.includes("vakıf") ||
    text.includes("kurumu") ||
    text.includes("şirketi") ||
    text.includes("kulübü")
  ) {
    return "organization";
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
};

let fxRatesCache = {
  timestamp: 0,
  dateStr: "",
  rates: { USD: 1, TRY: 48.27, EUR: 0.86, GBP: 0.74, JPY: 160.0, CHF: 0.81 },
};

async function getFxRates() {
  const now = Date.now();
  if (now - fxRatesCache.timestamp < 15 * 60 * 1000 && Object.keys(fxRatesCache.rates).length > 2) {
    return fxRatesCache;
  }
  try {
    const res = await fetch("https://api.frankfurter.dev/v1/latest?base=USD&symbols=TRY,EUR,GBP,JPY,CHF", {
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

async function solveInstantQuery(query) {
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
      const fxData = await getFxRates();
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
        if (typeof result === "number" && !isNaN(result) && isFinite(result)) {
          return {
            type: "calculator",
            expression: query.trim(),
            result: result.toLocaleString("tr-TR", { maximumFractionDigits: 6 }),
          };
        }
      }
    } catch {
      // ignore calculation error
    }
  }

  return null;
}

export async function load({ url, fetch }) {
  const q = (url.searchParams.get("q") || "").trim();
  const category = url.searchParams.get("kategori") || "general";
  const page = parseInt(url.searchParams.get("sayfa") || "1", 10);
  const language = url.searchParams.get("dil") || "tr";
  const timeRange = url.searchParams.get("zaman") || "";
  const safeSearch = url.searchParams.get("guvenli") || "1";
  const engines = url.searchParams.get("motorlar") || "";

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
      numberOfResults: 0,
      language: "tr",
      timeRange: "",
      safeSearch: "1",
    };
  }

  // Anlık Yanıt Çözücü (Döviz, Hesap Makinesi)
  const instantAnswer = await solveInstantQuery(q);

  const searxUrl = env.SEARXNG_URL || "http://localhost:8080";
  const searchParams = new URLSearchParams({
    q,
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

  try {
    const res = await fetch(`${searxUrl.replace(/\/+$/, "")}/search?${searchParams.toString()}`, {
      signal: AbortSignal.timeout(8000),
    });

    if (!res.ok) {
      return {
        isHome: false,
        query: q,
        category,
        page,
        results: [],
        infoboxes: [],
        suggestions: [],
        answer: instantAnswer,
        numberOfResults: 0,
        language,
        timeRange,
        safeSearch,
        error: instantAnswer ? null : `Arama servisi yanıt vermedi (${res.status})`,
      };
    }

    const data = await res.json();

    const results = (data.results || []).map((item) => ({
      title: item.title || "",
      url: item.url || "",
      content: item.content || "",
      imgSrc: item.img_src || item.thumbnail || "",
      thumbnail: item.thumbnail || "",
      publishedDate: item.publishedDate || item.pubdate || null,
      engine: item.engine || (item.engines && item.engines[0]) || "",
      parsedUrl: item.parsed_url || [],
    }));

    const rawInfoboxes = (data.infoboxes || [])
      .map((box) => {
        const title = box.infobox || box.title || "";
        const cleanAttrs = (box.attributes || [])
          .map((a) => ({
            label: formatAttrLabel(a.label || ""),
            value: formatAttrValue(a.label || "", a.value || ""),
          }))
          .filter((a) => a.label && a.value);

        const rawBox = {
          title,
          id: box.id || "",
          content: box.content || "",
          imgSrc: box.img_src || box.thumbnail || "",
          urls: cleanUrls(box.urls || (box.id ? [{ title: "Vikipedi", url: box.id }] : [])),
          attributes: cleanAttrs,
          engine: box.engine || (box.engines && box.engines[0]) || "",
        };

        const entityType = classifyEntity(rawBox, q);

        return {
          ...rawBox,
          entityType,
          placeInfo: null,
        };
      })
      .filter((box) => isRelevantInfobox(q, box.title));

    const infoboxes = rawInfoboxes;

    // Hava durumu veya coğrafi yer detaylarını bağlama
    const isWeatherQuery = q.toLowerCase().includes("hava durumu");
    if (isWeatherQuery || (infoboxes.length > 0 && (infoboxes[0].entityType === "place" || infoboxes[0].entityType === "organization"))) {
      try {
        let locationQuery = infoboxes.length > 0 ? (infoboxes[0].title || q) : q;
        if (isWeatherQuery) {
          // "Ankara hava durumu" -> "Ankara"
          locationQuery = q.replace(/hava\s*durumu/gi, "").trim() || "Ankara";
        }
        const placeDetails = await fetchPlaceDetails(locationQuery);
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
      } catch {
        // gracefully ignore
      }
    }

    const suggestions = data.suggestions || [];
    const numberOfResults = data.number_of_results || results.length;

    return {
      isHome: false,
      query: q,
      category,
      page,
      results,
      infoboxes,
      suggestions,
      answer: instantAnswer,
      numberOfResults,
      language,
      timeRange,
      safeSearch,
      error: null,
    };
  } catch (err) {
    return {
      isHome: false,
      query: q,
      category,
      page,
      results: [],
      infoboxes: [],
      suggestions: [],
      answer: instantAnswer,
      numberOfResults: 0,
      language,
      timeRange,
      safeSearch,
      error: instantAnswer ? null : "Arama servisine şu anda ulaşılamıyor.",
    };
  }
}
