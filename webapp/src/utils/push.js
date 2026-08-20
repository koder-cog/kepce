import { API_BASE } from '@/api/client.js';

function urlBase64ToUint8Array(base64String) {
  const padding = '='.repeat((4 - (base64String.length % 4)) % 4);
  const base64 = (base64String + padding).replace(/-/g, '+').replace(/_/g, '/');
  const rawData = window.atob(base64);
  const outputArray = new Uint8Array(rawData.length);
  for (let i = 0; i < rawData.length; ++i) {
    outputArray[i] = rawData.charCodeAt(i);
  }
  return outputArray;
}

export function isPushSupported() {
  return typeof window !== 'undefined' && 'serviceWorker' in navigator && 'PushManager' in window && 'Notification' in window;
}

export async function registerServiceWorker() {
  if (!isPushSupported()) return null;
  try {
    const reg = await navigator.serviceWorker.register('/sw.js', { scope: '/' });
    return reg;
  } catch (err) {
    console.error('Service worker kayıt hatası:', err);
    return null;
  }
}

export async function getPushSubscription() {
  if (!isPushSupported()) return null;
  try {
    const reg = await navigator.serviceWorker.ready;
    return await reg.pushManager.getSubscription();
  } catch (err) {
    console.error('Push aboneliği alma hatası:', err);
    return null;
  }
}

export async function subscribeToPush(options = {}) {
  if (!isPushSupported()) {
    throw new Error('Tarayıcınız Web Push bildirimlerini desteklemiyor.');
  }

  // 1. İzin İste
  const permission = await Notification.requestPermission();
  if (permission !== 'granted') {
    throw new Error('Bildirim izni verilmedi.');
  }

  // 2. Service Worker Hazırla
  let reg = await navigator.serviceWorker.getRegistration();
  if (!reg) {
    reg = await registerServiceWorker();
  }
  await navigator.serviceWorker.ready;

  // 3. Backend'den VAPID Public Key al
  const keyRes = await fetch(`${API_BASE}/api/v1/public/push/vapid-public-key`);
  if (!keyRes.ok) {
    throw new Error('VAPID sunucu anahtarı alınamadı.');
  }
  const { public_key } = await keyRes.json();

  // 4. PushManager ile Abone Ol
  const convertedKey = urlBase64ToUint8Array(public_key);
  let sub = await reg.pushManager.getSubscription();
  if (!sub) {
    sub = await reg.pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey: convertedKey
    });
  }

  const subJson = sub.toJSON();

  // 5. Backend'e Kaydet
  const payload = {
    endpoint: sub.endpoint,
    keys: {
      p256dh: subJson.keys.p256dh,
      auth: subJson.keys.auth
    },
    city_id: options.cityId || null,
    notif_breakfast_enabled: options.breakfastEnabled ?? true,
    notif_breakfast_time: options.breakfastTime || '07:30',
    notif_dinner_enabled: options.dinnerEnabled ?? true,
    notif_dinner_time: options.dinnerTime || '17:00',
    user_agent: navigator.userAgent
  };

  const saveRes = await fetch(`${API_BASE}/api/v1/public/push/subscribe`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify(payload)
  });

  if (!saveRes.ok) {
    const errData = await saveRes.json().catch(() => ({}));
    throw new Error(errData.message || 'Bildirim aboneliği sunucuya kaydedilemedi.');
  }

  return await saveRes.json();
}

export async function unsubscribeFromPush() {
  if (!isPushSupported()) return;

  try {
    const reg = await navigator.serviceWorker.ready;
    const sub = await reg.pushManager.getSubscription();
    if (sub) {
      // Backend'den sil
      await fetch(`${API_BASE}/api/v1/public/push/unsubscribe`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ endpoint: sub.endpoint })
      }).catch(() => {});

      // Tarayıcıdan sil
      await sub.unsubscribe();
    }
  } catch (err) {
    console.error('Abonelikten çıkma hatası:', err);
  }
}

export async function sendTestPush() {
  if (!isPushSupported()) {
    throw new Error('Web Push desteklenmiyor.');
  }

  const reg = await navigator.serviceWorker.ready;
  const sub = await reg.pushManager.getSubscription();
  if (!sub) {
    throw new Error('Aktif bildirim aboneliği bulunamadı. Lütfen önce bildirimleri açın.');
  }

  const res = await fetch(`${API_BASE}/api/v1/public/push/test`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify({ endpoint: sub.endpoint })
  });

  if (!res.ok) {
    const errData = await res.json().catch(() => ({}));
    throw new Error(errData.message || 'Test bildirimi gönderilemedi.');
  }

  return await res.json();
}
