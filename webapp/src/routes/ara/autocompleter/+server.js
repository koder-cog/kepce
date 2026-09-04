import { json } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";

export async function GET({ url, fetch }) {
  const q = (url.searchParams.get("q") || "").trim();
  const motor = (url.searchParams.get("motor") || "").trim();
  if (!q || q.length < 2) {
    return json([]);
  }

  const searxUrl = env.SEARXNG_URL || "http://localhost:8080";
  try {
    const completerQuery = motor && motor !== "off" ? `&completer=${encodeURIComponent(motor)}` : "";
    const res = await fetch(
      `${searxUrl.replace(/\/+$/, "")}/autocompleter?q=${encodeURIComponent(q)}${completerQuery}`,
      {
        signal: AbortSignal.timeout(3000),
      }
    );

    if (!res.ok) {
      return json([]);
    }

    const data = await res.json();
    // SearXNG standard autocomplete response: [query, [suggestions...], ...]
    if (Array.isArray(data) && Array.isArray(data[1])) {
      return json(data[1].slice(0, 6));
    } else if (Array.isArray(data)) {
      return json(
        data.filter((item) => typeof item === "string").slice(0, 6)
      );
    }
    return json([]);
  } catch (err) {
    return json([]);
  }
}
