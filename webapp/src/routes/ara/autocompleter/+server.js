import { json } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { suggestUnitCorrection } from "$lib/search/instantSolvers.js";

async function fetchUpstreamSuggestions(q, motor, customFetch = fetch) {
  try {
    if (motor === "google") {
      const gRes = await customFetch(
        `https://suggestqueries.google.com/complete/search?client=firefox&q=${encodeURIComponent(q)}`,
        { signal: AbortSignal.timeout(2500) }
      );
      if (gRes.ok) {
        const gData = await gRes.json();
        if (Array.isArray(gData) && Array.isArray(gData[1])) {
          return gData[1];
        }
      }
    } else {
      // Varsayılan / DuckDuckGo tamamlayıcı yedeği
      const ddgRes = await customFetch(
        `https://duckduckgo.com/ac/?q=${encodeURIComponent(q)}&type=list`,
        { signal: AbortSignal.timeout(2500) }
      );
      if (ddgRes.ok) {
        const ddgData = await ddgRes.json();
        if (Array.isArray(ddgData) && Array.isArray(ddgData[1])) {
          return ddgData[1];
        }
      }
    }
  } catch {
    // Upstream ağ hatasını sessizce yut
  }
  return [];
}

export async function GET({ url, fetch }) {
  const q = (url.searchParams.get("q") || "").trim();
  const motor = (url.searchParams.get("motor") || "").trim();
  if (!q || q.length < 2) {
    return json([]);
  }

  let suggestions = [];

  // 1. Önce yerel SearXNG servisinden çekmeyi dene
  const searxUrl = env.SEARXNG_URL || "http://localhost:8080";
  try {
    const completerQuery = motor && motor !== "off" ? `&completer=${encodeURIComponent(motor)}` : "";
    const res = await fetch(
      `${searxUrl.replace(/\/+$/, "")}/autocompleter?q=${encodeURIComponent(q)}${completerQuery}`,
      {
        signal: AbortSignal.timeout(2000),
      }
    );

    if (res.ok) {
      const data = await res.json();
      if (Array.isArray(data) && Array.isArray(data[1])) {
        suggestions = data[1];
      } else if (Array.isArray(data)) {
        suggestions = data.filter((item) => typeof item === "string");
      }
    }
  } catch {
    // SearXNG erişilemezse doğrudan upstream tamamlayıcıya yönel
  }

  // 2. SearXNG boş döndüyse veya çevrimdışıysa doğrudan sağlayıcıya danış
  if (suggestions.length === 0 && motor !== "off") {
    suggestions = await fetchUpstreamSuggestions(q, motor, fetch);
  }

  // 3. Birim veya hesaplama düzeltme önerisi varsa en başa ekle (örn: "50 g kaç lg" -> "50 g kaç kg")
  const unitCorrection = suggestUnitCorrection(q);
  if (unitCorrection && !suggestions.includes(unitCorrection.correctedQuery)) {
    suggestions = [unitCorrection.correctedQuery, ...suggestions];
  }

  return json(suggestions.slice(0, 6));
}
