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

  const isAraRoute = isAraSubdomain || event.url.pathname.startsWith("/ara");

  // Arama motorunda katı No-Referrer ve Güvenlik Başlıkları İzolasyonu (A4.1)
  if (isAraRoute) {
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

  return resolve(event, {
    transformPageChunk: ({ html }) => {
      if (isAraRoute) {
        return html
          .replace(/<link rel="icon" href="\/favicon\.ico"[^>]*>/, '<link rel="icon" type="image/svg+xml" href="/favicon-ara.svg" />')
          .replace(/<link rel="icon" type="image\/svg\+xml" href="\/favicon\.svg"[^>]*>/, '')
          .replace(/<link rel="icon" type="image\/png"[^>]*>/g, '');
      }
      return html;
    }
  });
}
