# Erişim ve Kimlik Doğrulama

Kepçe API uç noktalarına anonim olarak veya geliştirici anahtarıyla (`X-API-Key`) erişebilirsiniz.

## 1. Anonim Erişim

Genel okuma uç noktalarının tamamı API anahtarı gerektirmeden herkese açıktır.

Anonim isteklerde IP adresi başına dakikada en fazla 240, saniyede ise ani yük koruması (SpikeArrest) kapsamında en fazla 10 istek yapılabilir.

> [!NOTE]
> Saniyede 10 istek gönderen bir betik 24. saniyede dakikalık 240 isteklik kotayı tüketerek `429 Too Many Requests` hatası alır. Düzenli veri çeken istemcilerin istek aralarına en az 250 milisaniye bekleme süresi koyması önerilir.

## 2. Geliştirici Erişimi (`X-API-Key`)

Üniversite botları, mobil widget'lar ve kampüs bilgi panoları geliştirenlerin API anahtarı kullanması tavsiye edilir.

Standart ücretsiz geliştirici seviyesinde günlük 2.500 istek hakkı tanınır. Bu kota kişisel projeler, Discord/Telegram botları ve kampüs içi araçlar için yeterlidir. Kurumsal ve ticari kullanım seviyesinde ise günlük 100.000 istek hakkı sağlanır.

Günlük kotalar her gece 00:00 UTC anında sıfırlanır. Anlık tüketim ve kalan kota `/gelistirici` panelinden izlenebilir.

### API Anahtarı Alma Adımları
1. [kepce.org](https://kepce.org) üzerinde oturum açın.
2. `/gelistirici` sayfasına geçerek yeni bir API anahtarı üretin.
3. Üretilen anahtarı güvenli bir ortamda saklayın, istemci taraflı açık kodlarda paylaşmayın.

### İstek Başlığı Tanımı
API anahtarınızı HTTP isteklerinizin başlığına `X-API-Key` alanıyla ekleyin:

```bash
curl -X GET "https://kepce.org/api/v1/menus/today/istanbul" \
  -H "X-API-Key: kepce_live_ornekanahtar123456789" \
  -H "Accept: application/json"
```

JavaScript Fetch örneği:
```javascript
const res = await fetch("https://kepce.org/api/v1/menus/today/istanbul", {
  headers: {
    "X-API-Key": process.env.KEPCE_API_KEY,
    "Accept": "application/json"
  }
});
const data = await res.json();
```

## 3. Kullanıcı İşlemleri (`Bearer JWT`)

Menüye oy verme ve yorum gönderme gibi kullanıcı bazlı işlemler için oturum jetonu gerekir:

```http
POST /api/v1/menus/1420/vote
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json
```

Kullanıcı veya IP adresi başına dakikada en fazla 10 oy verilebilir ve en fazla 5 yorum gönderilebilir.

## 4. Hız Sınırları ve Kota Aşımı

Tanımlanan istek kotası aşıldığında API `429 Too Many Requests` durumuyla döner:

```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json
Retry-After: 30

{
  "error": "İstek kotası aşıldı. Lütfen bir süre sonra tekrar deneyiniz."
}
```

İstemciler `Retry-After` başlığında belirtilen saniye kadar bekledikten sonra isteği yinelemelidir. Hata durumlarında uygulamanızın kilitlenmemesi için üstel geri çekilme (exponential backoff) algoritması uygulamanız önerilir.
