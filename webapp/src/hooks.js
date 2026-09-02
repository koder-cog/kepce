/**
 * SvelteKit 2 Universal Reroute Hook
 * ara.kepce.org ve ara.localhost subdomain isteklerini izole /ara rotasına yönlendirir.
 */
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
