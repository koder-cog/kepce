/**
 * SvelteKit 2 Universal Reroute Hook
 * ara.kepce.org ve ara.localhost subdomain isteklerini izole /ara rotasına yönlendirir.
 */
export function reroute({ url }) {
  const hostname = url.hostname.toLowerCase();
  const isAraSubdomain = hostname.startsWith("ara.") || hostname === "ara.localhost";

  if (isAraSubdomain) {
    if (url.pathname === "/") {
      return "/ara";
    }
    if (url.pathname === "/opensearch.xml") {
      return "/ara/opensearch.xml";
    }
    if (!url.pathname.startsWith("/ara")) {
      return `/ara${url.pathname}`;
    }
  }
}
