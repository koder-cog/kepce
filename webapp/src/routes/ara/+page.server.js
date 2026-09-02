import { env } from "$env/dynamic/private";

async function fetchPlaceDetails(placeName) {
  if (!placeName || placeName.length < 2) return null;

  let lat = null;
  let lon = null;
  let displayName = placeName;
  let country = "";

  try {
    // 1. Open-Meteo Geocoding (şehirler ve yerleşimler)
    const geoRes = await fetch(
      `https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(placeName)}&count=1&language=tr`,
      { signal: AbortSignal.timeout(1000) }
    );
    if (geoRes.ok) {
      const geoData = await geoRes.json();
      if (geoData.results && geoData.results.length > 0) {
        const place = geoData.results[0];
        lat = place.latitude;
        lon = place.longitude;
        displayName = place.name;
        country = place.country || "";
      }
    }

    // 2. Nominatim Fallback (göller, dağlar, üniversiteler, yapılar)
    if (!lat || !lon) {
      const nomRes = await fetch(
        `https://nominatim.openstreetmap.org/search?q=${encodeURIComponent(placeName)}&format=json&limit=1`,
        {
          headers: { "User-Agent": "KepceSearch/1.0" },
          signal: AbortSignal.timeout(1200),
        }
      );
      if (nomRes.ok) {
        const nomData = await nomRes.json();
        if (nomData && nomData.length > 0) {
          lat = parseFloat(nomData[0].lat);
          lon = parseFloat(nomData[0].lon);
          displayName = nomData[0].name || placeName;
          const parts = (nomData[0].display_name || "").split(", ");
          country = parts[parts.length - 1] || "";
        }
      }
    }

    if (!lat || !lon) return null;

    // 3. Open-Meteo Hava Durumu
    const weatherRes = await fetch(
      `https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&current=temperature_2m,weather_code&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto&forecast_days=3`,
      { signal: AbortSignal.timeout(1000) }
    );
    let weather = null;
    if (weatherRes.ok) {
      const wData = await weatherRes.json();
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

function classifyEntity(box, q) {
  const attributes = box.attributes || [];
  const labels = attributes.map((a) => (a.label || "").toLowerCase());
  const text = `${box.title || ""} ${box.content || ""} ${q || ""}`.toLowerCase();

  // 1. Kişi (Person) Kontrolü
  const personLabels = [
    "doğum tarihi",
    "ölüm tarihi",
    "vatandaşlığı",
    "mesleği",
    "eşi",
    "çocukları",
    "boyu",
    "ebeveynleri",
    "etkin yılları",
    "eğitimi",
  ];
  if (labels.some((l) => personLabels.includes(l))) {
    return "person";
  }
  const personKeywords = [
    "devlet adamı",
    "mareşal",
    "yazar",
    "şair",
    "fizikçi",
    "matematikçi",
    "müzisyen",
    "futbolcu",
    "oyuncu",
    "şarkıcı",
    "ressam",
    "bilim insanı",
    "politikacı",
    "filozof",
  ];
  if (
    personKeywords.some((k) => text.includes(k)) &&
    !text.includes("üniversite") &&
    !text.includes("şehir")
  ) {
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
      numberOfResults: 0,
      language: "tr",
      timeRange: "",
      safeSearch: "1",
    };
  }

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
        numberOfResults: 0,
        language,
        timeRange,
        safeSearch,
        error: `Arama servisi yanıt vermedi (${res.status})`,
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

    const infoboxes = (data.infoboxes || []).map((box) => {
      const cleanAttrs = (box.attributes || [])
        .map((a) => ({
          label: a.label || "",
          value: a.value || "",
        }))
        .filter((a) => a.label && a.value);

      const rawBox = {
        title: box.infobox || box.title || "",
        id: box.id || "",
        content: box.content || "",
        imgSrc: box.img_src || box.thumbnail || "",
        urls: box.urls || (box.id ? [{ title: "Vikipedi", url: box.id }] : []),
        attributes: cleanAttrs,
        engine: box.engine || (box.engines && box.engines[0]) || "",
      };

      const entityType = classifyEntity(rawBox, q);

      return {
        ...rawBox,
        entityType,
        placeInfo: null,
      };
    });

    if (
      infoboxes.length > 0 &&
      (infoboxes[0].entityType === "place" || infoboxes[0].entityType === "organization")
    ) {
      try {
        const placeDetails = await fetchPlaceDetails(infoboxes[0].title || q);
        if (placeDetails) {
          infoboxes[0].placeInfo = placeDetails;
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
      numberOfResults: 0,
      language,
      timeRange,
      safeSearch,
      error: "Arama servisine şu anda ulaşılamıyor.",
    };
  }
}
