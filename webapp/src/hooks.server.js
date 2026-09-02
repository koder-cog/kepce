import { redirect } from "@sveltejs/kit";

export function reroute({ url }) {
  const hostname = url.hostname.toLowerCase();
  const isAraSubdomain = hostname.startsWith("ara.") || hostname === "ara.localhost";

  if (isAraSubdomain) {
    if (url.pathname === "/" || url.pathname === "") {
      return "/ara";
    }
    if (
      url.pathname.startsWith("/ara") ||
      url.pathname === "/opensearch.xml" ||
      url.pathname === "/favicon.ico" ||
      url.pathname === "/robots.txt" ||
      url.pathname.startsWith("/_app")
    ) {
      return url.pathname;
    }
    return `/ara${url.pathname}`;
  }
}

export async function handle({ event, resolve }) {
  const hostname = event.url.hostname.toLowerCase();
  const isAraSubdomain = hostname.startsWith("ara.") || hostname === "ara.localhost";

  // Arama motorunda katı No-Referrer ve Güvenlik Başlıkları İzolasyonu (A4.1)
  if (isAraSubdomain || event.url.pathname.startsWith("/ara")) {
    event.setHeaders({
      "Referrer-Policy": "no-referrer",
      "X-Content-Type-Options": "nosniff",
      "X-Frame-Options": "DENY",
    });
  }

  // Production ortamında ana domainden kepce.org/ara isteklerini ara.kepce.org'a yönlendir (A4.2)
  if (
    !isAraSubdomain &&
    (event.url.pathname === "/ara" || event.url.pathname.startsWith("/ara/")) &&
    hostname.includes("kepce.org")
  ) {
    const cleanPath = event.url.pathname.replace(/^\/ara/, "") || "/";
    throw redirect(302, `https://ara.kepce.org${cleanPath}${event.url.search}`);
  }

  return resolve(event);
}
