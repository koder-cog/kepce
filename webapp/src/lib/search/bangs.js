// Kepçe Ara — 50+ Hızlı Bang / Kısayol Yönlendirme Motoru

export const BANG_DEFINITIONS = [
  // Ansiklopedi & Bilgi
  { prefix: "!w", name: "Vikipedi (TR)", url: "https://tr.wikipedia.org/wiki/Special:Search?search=" },
  { prefix: "!wen", name: "Wikipedia (EN)", url: "https://en.wikipedia.org/wiki/Special:Search?search=" },
  { prefix: "!tdk", name: "TDK Güncel Türkçe Sözlük", url: "https://sozluk.gov.tr/?ara=" },
  { prefix: "!arch", name: "ArchWiki", url: "https://wiki.archlinux.org/index.php?search=" },
  { prefix: "!wikt", name: "Vikisözlük", url: "https://tr.wiktionary.org/wiki/Special:Search?search=" },

  // Geliştirici & Kod
  { prefix: "!gh", name: "GitHub", url: "https://github.com/search?q=" },
  { prefix: "!gl", name: "GitLab", url: "https://gitlab.com/search?search=" },
  { prefix: "!so", name: "Stack Overflow", url: "https://stackoverflow.com/search?q=" },
  { prefix: "!npm", name: "npm Packages", url: "https://www.npmjs.com/search?q=" },
  { prefix: "!crates", name: "Crates.io (Rust)", url: "https://crates.io/search?q=" },
  { prefix: "!pypi", name: "PyPI (Python)", url: "https://pypi.org/search/?q=" },
  { prefix: "!mdn", name: "MDN Web Docs", url: "https://developer.mozilla.org/search?q=" },
  { prefix: "!hf", name: "Hugging Face", url: "https://huggingface.co/models?search=" },
  { prefix: "!pkg", name: "Go Packages", url: "https://pkg.go.dev/search?q=" },
  { prefix: "!docker", name: "Docker Hub", url: "https://hub.docker.com/search?q=" },

  // Multimedya & Video & Müzik
  { prefix: "!yt", name: "YouTube", url: "https://www.youtube.com/results?search_query=" },
  { prefix: "!ytm", name: "YouTube Music", url: "https://music.youtube.com/search?q=" },
  { prefix: "!sp", name: "Spotify", url: "https://open.spotify.com/search/" },
  { prefix: "!genius", name: "Genius (Şarkı Sözleri)", url: "https://genius.com/search?q=" },
  { prefix: "!sc", name: "SoundCloud", url: "https://soundcloud.com/search?q=" },
  { prefix: "!twitch", name: "Twitch", url: "https://www.twitch.tv/search?term=" },
  { prefix: "!steam", name: "Steam", url: "https://store.steampowered.com/search/?term=" },
  { prefix: "!imdb", name: "IMDb", url: "https://www.imdb.com/find?q=" },
  { prefix: "!lb", name: "Letterboxd", url: "https://letterboxd.com/search/" },
  { prefix: "!goodreads", name: "Goodreads", url: "https://www.goodreads.com/search?q=" },
  { prefix: "!unsplash", name: "Unsplash", url: "https://unsplash.com/s/photos/" },
  { prefix: "!vimeo", name: "Vimeo", url: "https://vimeo.com/search?q=" },
  { prefix: "!pin", name: "Pinterest", url: "https://www.pinterest.com/search/pins/?q=" },

  // Sosyal Medya & Topluluk
  { prefix: "!r", name: "Reddit", url: "https://www.reddit.com/search/?q=" },
  { prefix: "!eksi", name: "Ekşi Sözlük", url: "https://eksisozluk.com/?q=" },
  { prefix: "!x", name: "X (Twitter)", url: "https://twitter.com/search?q=" },
  { prefix: "!hn", name: "Hacker News", url: "https://hn.algolia.com/?q=" },

  // Arama Motorları
  { prefix: "!g", name: "Google", url: "https://www.google.com/search?q=" },
  { prefix: "!ddg", name: "DuckDuckGo", url: "https://duckduckgo.com/?q=" },
  { prefix: "!brave", name: "Brave Search", url: "https://search.brave.com/search?q=" },
  { prefix: "!bing", name: "Bing", url: "https://www.bing.com/search?q=" },
  { prefix: "!yandex", name: "Yandex", url: "https://yandex.com.tr/search/?text=" },

  // Haritalar & Coğrafya
  { prefix: "!m", name: "OpenStreetMap", url: "https://www.openstreetmap.org/search?query=" },
  { prefix: "!maps", name: "Google Haritalar", url: "https://www.google.com/maps/search/" },
  { prefix: "!gmaps", name: "Google Haritalar", url: "https://www.google.com/maps/search/" },

  // Alışveriş
  { prefix: "!trendyol", name: "Trendyol", url: "https://www.trendyol.com/sr?q=" },
  { prefix: "!hepsiburada", name: "Hepsiburada", url: "https://www.hepsiburada.com/ara?q=" },
  { prefix: "!amazon", name: "Amazon TR", url: "https://www.amazon.com.tr/s?k=" },
  { prefix: "!sahibinden", name: "Sahibinden", url: "https://www.sahibinden.com/kelime-ile-arama?query_text=" },

  // Çeviri & Araçlar
  { prefix: "!tr", name: "Google Çeviri (TR)", url: "https://translate.google.com/?sl=auto&tl=tr&text=" },
  { prefix: "!tren", name: "Google Çeviri (EN)", url: "https://translate.google.com/?sl=auto&tl=en&text=" },
  { prefix: "!chatgpt", name: "ChatGPT", url: "https://chatgpt.com/?q=" },
  { prefix: "!claude", name: "Claude", url: "https://claude.ai/new?q=" },
  { prefix: "!wayback", name: "Wayback Machine", url: "https://web.archive.org/web/*/" }
];

export function resolveBang(query) {
  if (!query || !query.startsWith("!")) return null;

  const trimmed = query.trim();
  const parts = trimmed.split(/\s+/);
  const prefix = parts[0].toLowerCase();
  const rawTerm = parts.slice(1).join(" ").trim();

  // 1. Yerel tanımlı hızlı bang'ler (0ms direkt yönlendirme)
  const match = BANG_DEFINITIONS.find((b) => b.prefix === prefix);
  if (match) {
    if (!rawTerm) {
      // Sorgu terimsizse doğrudan ilgili servisin ana sayfasına git
      try {
        const u = new URL(match.url);
        return u.origin;
      } catch {
        return match.url;
      }
    }
    return match.url + encodeURIComponent(rawTerm);
  }

  // 2. Yerel listede olmayan tüm !bang kalıplarını DuckDuckGo'nun 13.000+ global havuzuna devret
  if (/^![a-z0-9_]+$/i.test(prefix)) {
    return `https://duckduckgo.com/?q=${encodeURIComponent(trimmed)}`;
  }

  return null;
}
