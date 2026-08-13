export function parseUserAgent(uaString) {
  if (!uaString) return { os: 'Bilinmeyen Cihaz', browser: 'Bilinmeyen Tarayıcı', icon: 'monitor' };

  let os = 'Bilinmeyen OS';
  let browser = 'Bilinmeyen Tarayıcı';
  let icon = 'monitor'; // varsayılan masaüstü ikonu (lucide)

  // OS Tespiti
  if (uaString.includes('Windows')) os = 'Windows';
  else if (uaString.includes('Mac OS X')) os = 'macOS';
  else if (uaString.includes('Android')) { os = 'Android'; icon = 'smartphone'; }
  else if (uaString.includes('iPhone')) { os = 'iOS'; icon = 'smartphone'; }
  else if (uaString.includes('iPad')) { os = 'iPadOS'; icon = 'tablet'; }
  else if (uaString.includes('Linux')) os = 'Linux';

  // Tarayıcı Tespiti
  if (uaString.includes('Firefox') && !uaString.includes('Seamonkey')) browser = 'Firefox';
  else if (uaString.includes('OPR') || uaString.includes('Opera')) browser = 'Opera';
  else if (uaString.includes('Edg')) browser = 'Edge';
  else if (uaString.includes('Chrome')) browser = 'Chrome';
  else if (uaString.includes('Safari')) browser = 'Safari';

  return { os, browser, icon };
}
