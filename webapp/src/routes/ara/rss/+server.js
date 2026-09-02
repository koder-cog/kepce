import { env } from "$env/dynamic/private";

function escapeXml(unsafe) {
  return String(unsafe || "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&apos;");
}

export async function GET({ url, fetch }) {
  const q = (url.searchParams.get("q") || "").trim();
  const category = url.searchParams.get("kategori") || "general";

  if (!q) {
    return new Response("Missing search query ?q=", { status: 400 });
  }

  const searxUrl = env.SEARXNG_URL || "http://localhost:8080";
  const searchParams = new URLSearchParams({
    q,
    format: "json",
    categories: category,
  });

  let items = [];
  try {
    const res = await fetch(`${searxUrl.replace(/\/+$/, "")}/search?${searchParams.toString()}`, {
      signal: AbortSignal.timeout(5000),
    });
    if (res.ok) {
      const data = await res.json();
      items = data.results || [];
    }
  } catch {
    items = [];
  }

  const host = url.origin;
  const rssXml = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>Kepçe Ara - ${escapeXml(q)}</title>
    <link>${host}/ara?q=${encodeURIComponent(q)}</link>
    <description>${escapeXml(q)} için Kepçe Ara sonuçları</description>
    <language>tr</language>
    <atom:link href="${host}${url.pathname}?${url.searchParams.toString()}" rel="self" type="application/rss+xml"/>
    ${items
      .map(
        (it) => `
    <item>
      <title>${escapeXml(it.title || "")}</title>
      <link>${escapeXml(it.url || "")}</link>
      <guid isPermaLink="true">${escapeXml(it.url || "")}</guid>
      <description>${escapeXml(it.content || "")}</description>
      ${it.publishedDate ? `<pubDate>${new Date(it.publishedDate).toUTCString()}</pubDate>` : ""}
    </item>`
      )
      .join("\n")}
  </channel>
</rss>`;

  return new Response(rssXml, {
    headers: {
      "Content-Type": "application/rss+xml; charset=utf-8",
      "Cache-Control": "public, max-age=300",
    },
  });
}
