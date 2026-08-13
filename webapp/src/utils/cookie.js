export function getCookie(name) {
  if (typeof document === 'undefined') return null;
  const value = `; ${document.cookie}`;
  const parts = value.split(`; ${name}=`);
  if (parts.length === 2) return parts.pop().split(';').shift();
  return null;
}

export function clearLoggedCookie() {
  if (typeof document === 'undefined') return;
  document.cookie = "kepce_logged_in=; Path=/; Max-Age=0; SameSite=Strict" + 
    (window.location.protocol === "https:" ? "; Secure" : "");
}
