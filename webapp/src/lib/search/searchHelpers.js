// Pure search helper functions & constants for Kepçe Ara

export const CATEGORIES = [
  { id: "general", label: "Web" },
  { id: "images", label: "Görseller" },
  { id: "videos", label: "Videolar" },
  { id: "news", label: "Haberler" },
  { id: "it", label: "Kod" },
  { id: "science", label: "Akademi" },
  { id: "map", label: "Haritalar" },
];

export const REGION_OPTIONS = [
  { value: "all", label: "Tüm bölgeler" },
  { value: "tr", label: "Türkiye" },
  { value: "de-DE", label: "Almanya" },
  { value: "en-US", label: "Amerika Birleşik Devletleri" },
  { value: "az", label: "Azerbaycan" },
  { value: "en-GB", label: "Birleşik Krallık" },
  { value: "pt-BR", label: "Brezilya" },
  { value: "zh-CN", label: "Çin" },
  { value: "fr-FR", label: "Fransa" },
  { value: "ko-KR", label: "Güney Kore" },
  { value: "nl-NL", label: "Hollanda" },
  { value: "es-ES", label: "İspanya" },
  { value: "sv-SE", label: "İsveç" },
  { value: "it-IT", label: "İtalya" },
  { value: "ja-JP", label: "Japonya" },
  { value: "en-CA", label: "Kanada" },
  { value: "pl-PL", label: "Polonya" },
  { value: "ru-RU", label: "Rusya" },
  { value: "ar-SA", label: "Suudi Arabistan" },
  { value: "el-GR", label: "Yunanistan" },
];

export function getDomain(rawUrl) {
  try {
    const u = new URL(rawUrl);
    return u.hostname.replace(/^www\./, "");
  } catch {
    return "";
  }
}

export function getFaviconUrl(rawUrl, resolver = "google") {
  if (resolver === "off" || resolver === "none") return null;
  const domain = getDomain(rawUrl);
  if (!domain) return null;
  if (resolver === "google") {
    return `https://www.google.com/s2/favicons?domain=${domain}&sz=32`;
  }
  return `https://icons.duckduckgo.com/ip3/${domain}.ico`;
}

export function formatUrlBreadcrumb(rawUrl) {
  try {
    const u = new URL(rawUrl);
    const parts = u.pathname.split("/").filter(Boolean);
    if (parts.length === 0) return u.origin;
    return `${u.origin} › ${parts.join(" › ")}`;
  } catch {
    return rawUrl;
  }
}

export function formatDateSnippet(dateStr) {
  if (!dateStr) return "";
  try {
    const d = new Date(dateStr);
    if (isNaN(d.getTime())) return "";
    return d.toLocaleDateString("tr-TR", {
      day: "numeric",
      month: "short",
      year: "numeric",
    });
  } catch {
    return "";
  }
}

export function escapeHtml(str) {
  return String(str || "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

export function highlightQuery(text, query) {
  if (!text) return "";
  if (!query) return escapeHtml(text);

  const tokens = query
    .trim()
    .split(/\s+/)
    .filter((t) => t.length > 1 && !t.startsWith("!"));
  if (tokens.length === 0) return escapeHtml(text);

  const escapedTokens = tokens.map((t) =>
    t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"),
  );
  const regex = new RegExp(`(${escapedTokens.join("|")})`, "gi");

  const safe = escapeHtml(text);
  return safe.replace(regex, "<strong>$1</strong>");
}

export function getYoutubeId(url) {
  if (!url) return null;
  const m = url.match(
    /(?:youtube\.com\/(?:watch\?v=|embed\/)|youtu\.be\/)([a-zA-Z0-9_-]{11})/,
  );
  return m ? m[1] : null;
}

export function getYoutubeEmbedUrl(url) {
  const id = getYoutubeId(url);
  if (id) {
    return `https://www.youtube-nocookie.com/embed/${id}?autoplay=1`;
  }
  return null;
}

export function getYoutubeThumbnail(url) {
  const id = getYoutubeId(url);
  return id ? `https://i.ytimg.com/vi/${id}/hqdefault.jpg` : null;
}

/**
 * Vikipedi ve bilgi kartı metinlerindeki ansiklopedik telaffuz, IPA ve
 * doğum/ölüm tarih parantezlerini temizleyerek akıcı bir başlangıç sunar.
 */
export function cleanLeadParentheses(text) {
  if (!text) return "";
  let str = text.trim();

  // 1. Telaffuz ve IPA parantezleri (örn: (İngilizce telaffuz: [ˈpaɪθɑːn]), ([paʁi]))
  str = str.replace(/\s*\((?:[A-Za-zÇĞİÖŞÜçğıöşü\s]+telaffuz:\s*)?\[[^\]]+\]\)/gi, "");
  str = str.replace(/\s*\([A-Za-zÇĞİÖŞÜçğıöşü\s]+telaffuz:\s*[^)]+\)/gi, "");

  // 2. İlk cümledeki yabancı dil / orijinal ad parantezleri (örn: (İngilizce: London), (Japonca: 東京))
  str = str.replace(
    /^([^.!?\n]{1,80}?)\s*\((?:İngilizce|Fransızca|Almanca|Arapça|Farsça|Rusça|Yunanca|Latince|Japonca|Çince|Osmanlıca|İtalyanca|İspanyolca|Korece):\s*[^)]+\)/i,
    "$1"
  );

  // 3. İlk cümledeki doğum-ölüm tarih parantezleri (örn: (14 Mart 1879 – 18 Nisan 1955), (1881 – 1938))
  str = str.replace(
    /^([^.!?\n]{1,80}?)\s*\(\s*(?:d\.\s*|ö\.\s*)?(?:\d{1,2}\s+[A-Za-zÇĞİÖŞÜçğıöşü]+\s+)?\d{3,4}[^)]*(?:–|-)[^)]*\d{3,4}[^)]*\)/i,
    "$1"
  );
  str = str.replace(
    /^([^.!?\n]{1,80}?)\s*\(\s*(?:d\.\s*|ö\.\s*)\d{3,4}[^)]*\)/i,
    "$1"
  );

  // Noktalama ve boşluk düzeltmesi (örn: "Atatürk , Türk" -> "Atatürk, Türk")
  str = str
    .replace(/\s+([,.:;!?])/g, "$1")
    .replace(/([,.:;!?])\1+/g, "$1")
    .replace(/\s{2,}/g, " ")
    .trim();

  return str;
}

export function buildSearchUrl(params, isSubdomain = false) {
  const qs = params.toString();
  const prefix = isSubdomain ? "" : "/ara";
  return `${prefix}${qs ? `?${qs}` : ""}` || "/";
}

export function checkInstantPreview(val, preferences = {}) {
  const q = (val || "").trim().toLowerCase();
  if (!q) return null;

  if (
    preferences.pluginCalculator &&
    /^[\d\s.,+\-*/()^%]+$/.test(q) &&
    /[+\-*/^%]/.test(q)
  ) {
    try {
      const sanitized = q.replace(/,/g, ".").replace(/\^/g, "**");
      if (!/[a-zA-Z_$]/.test(sanitized)) {
        // eslint-disable-next-line no-new-func
        const result = Function(`'use strict'; return (${sanitized})`)();
        if (typeof result === "number") {
          let resText = "";
          if (isNaN(result)) {
            resText = "Tanımsız (0/0 belirsizliği)";
          } else if (!isFinite(result)) {
            resText = "Tanımsız (Sıfıra bölünemez)";
          } else {
            resText = result.toLocaleString("tr-TR", {
              maximumFractionDigits: 6,
            });
          }
          return {
            badge: "Hesaplama",
            text: `${q} = ${resText}`,
          };
        }
      }
    } catch {}
  }

  if (preferences.pluginCalculator || preferences.pluginUnitConverter) {
    const mCur = q.match(/^(\d+(?:[.,]\d+)?)\s*(dolar|usd|\$|euro|eur|€)/i);
    if (mCur) {
      const amt = parseFloat(mCur[1].replace(",", "."));
      const cur = mCur[2].toLowerCase();
      const rate = cur.includes("e") || cur.includes("€") ? 52.5 : 48.27;
      const total = amt * rate;
      return {
        badge: "Döviz Tahmini",
        text: `≈ ${total.toLocaleString("tr-TR", { maximumFractionDigits: 2 })} ₺`,
      };
    }
  }

  return null;
}

export function isAnswerPluginAllowed(ans, preferences = {}) {
  if (!ans) return false;
  if (ans.type === "calculator" && !preferences.pluginCalculator) return false;
  if (ans.type === "unit" && !preferences.pluginUnitConverter) return false;
  if (ans.type === "time" && !preferences.pluginTimezones) return false;
  return true;
}
