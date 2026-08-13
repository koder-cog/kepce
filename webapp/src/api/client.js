// `HOST_BASE` SSR sırasında `window` tanımsız olduğundan hata fırlatır;
// statik SvelteKit derlemesi için sorun olmasa da gelecekteki olası
// kullanımlar (örn. meta etiketleri) için güvenli bir fallback sunuyoruz.
const _origin = typeof window !== 'undefined' ? window.location.origin : '';

export const API_BASE = '/api/v1';
export const HOST_BASE = _origin;

let isRefreshing = false;
let refreshSubscribers = [];

function subscribeTokenRefresh(cb) {
  refreshSubscribers.push(cb);
}

function onRefreshed() {
  refreshSubscribers.forEach((cb) => cb());
  refreshSubscribers = [];
}

function onRefreshFailed(error) {
  refreshSubscribers.forEach((cb) => cb(error || new Error('Oturum yenileme başarısız')));
  refreshSubscribers = [];
}

export function buildQuery(params) {
  const q = new URLSearchParams();
  for (const [key, val] of Object.entries(params)) {
    if (val !== undefined && val !== null && val !== '') {
      q.append(key, val);
    }
  }
  const qs = q.toString();
  return qs ? `?${qs}` : '';
}

export async function request(path, options = {}) {
  let url = `${API_BASE}${path}`;
  const headers = { ...options.headers };

  if (!(typeof FormData !== 'undefined' && options.body instanceof FormData) && !headers['Content-Type']) {
    headers['Content-Type'] = 'application/json';
  }

  // Tüm GET'lere cache buster eklemek CDN ve tarayıcı önbelleğini tamamen
  // devre dışı bırakıyordu; bu, hem API yükünü hem yanıt süresini
  // artırıyordu. Sadece açıkça `noCache: true` istenen isteklerde veya
  // mutasyon sonrası güncel veriye ihtiyaç duyulan rotalarda buster
  // ekliyoruz. Önceki davranış, `Cache-Control: no-store` ayarlansa bile
  // yine de her istekte 304/200 yarışı çıkarıyordu.
  if (
    (!options.method || options.method.toUpperCase() === 'GET') &&
    options.noCache === true
  ) {
    const separator = url.includes('?') ? '&' : '?';
    url += `${separator}_t=${Date.now()}`;
  }

  const fetchOptions = {
    ...options,
    headers,
    credentials: 'include'
  };

  let res;
  try {
    res = await fetch(url, fetchOptions);
  } catch (err) {
    throw new Error('İnternetin çekmiyor ya da sunucu bayılmış, mutfağın kapısına git de aşçı abla ne diyor bir bak.');
  }

  const hasLoggedInCookie = typeof document !== 'undefined' && document.cookie.includes('kepce_logged_in');

if (res.status === 401 && hasLoggedInCookie && path !== '/auth/refresh' && path !== '/auth/login' && path !== '/auth/register') {
    if (!isRefreshing) {
      isRefreshing = true;
      try {
        await request('/auth/refresh', { method: 'POST' });
        isRefreshing = false;
        onRefreshed();
      } catch (err) {
        isRefreshing = false;
        onRefreshFailed(err);
        if (typeof document !== 'undefined') {
          document.cookie = "kepce_logged_in=; Path=/; Max-Age=0; SameSite=Strict" + 
            (window.location.protocol === "https:" ? "; Secure" : "");
        }
        throw new Error('Oturumunuz sonlanmış, lütfen tekrar giriş yapın.');
      }
    }

    return new Promise((resolve, reject) => {
      subscribeTokenRefresh((err) => {
        if (err) return reject(err);
        request(path, options)
          .then(resolve)
          .catch(reject);
      });
    });
  }

  if (!res.ok) {
    const bodyText = await res.text().catch(() => "");
    let body = {};
    try {
      body = JSON.parse(bodyText);
    } catch (e) {}
    
    let detail = body.error || body.message || body.detail || (bodyText.length > 0 ? bodyText : `Hata oluştu (Durum: ${res.status})`);
    if (typeof detail === 'object') {
      detail = JSON.stringify(detail);
    }
    const error = new Error(detail);
    error.status = res.status;
    throw error;
  }

  if (res.status === 204) return null;
  return res.json();
}
