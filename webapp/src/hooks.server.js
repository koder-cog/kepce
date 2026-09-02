import { redirect } from "@sveltejs/kit";

export async function handle({ event, resolve }) {
  const hostname = event.url.hostname.toLowerCase();
  const isAraSubdomain = hostname.startsWith("ara.") || hostname === "ara.localhost";

  // Production ortamında ana domainden kepce.org/ara isteklerini ara.kepce.org'a yönlendir
  if (!isAraSubdomain && (event.url.pathname === "/ara" || event.url.pathname.startsWith("/ara/")) && hostname.includes("kepce.org")) {
    const cleanPath = event.url.pathname.replace(/^\/ara/, "") || "/";
    throw redirect(302, `https://ara.kepce.org${cleanPath}${event.url.search}`);
  }

  return resolve(event);
}
