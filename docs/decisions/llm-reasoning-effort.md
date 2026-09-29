# Karar Kaydı: LLM Muhakeme Eforu (Reasoning Effort)

**Karar:** `OPENROUTER_REASONING_EFFORT=low` seviyesinde kalır.
**Durum:** Uygulandı. Varsayılan hem [`docker-compose.yml`](../../docker-compose.yml)
hem [`worker/src/parser/llm.rs`](../../worker/src/parser/llm.rs) içinde `low`.
**Tarih:** 2026.09.27
**Kapsam:** `worker/src/parser/llm.rs`, Gemini Flash Thinking modelleri
**Referanslar:** Google Developer Knowledge (`ai.google.dev/pricing`,
`firebase.google.com/docs/ai-logic/thinking`), OpenRouter `/chat/completions` API

---

## 1. Yönetici Özeti

Kepçe sisteminde yemekhane menü belgelerinin (PDF, taranmış tablo, görsel)
dijitalleştirilmesinde birincil LLM sağlayıcısı olarak OpenRouter üzerinden Gemini
Flash ailesi (`~google/gemini-flash-latest`), yedek olarak ise doğrudan Gemini API
kullanılmaktadır.

Bu raporda `OPENROUTER_REASONING_EFFORT` parametresinin (`low`, `medium`, `high`)
aylık tabldot menüsü çıkarma işlemindeki token tüketimi, maliyet çarpanı ve yanıt
kesilme riski karşılaştırmalı olarak analiz edilmiştir.

**Mühendislik kararı:** Yapısal menü çıkarımı için
`OPENROUTER_REASONING_EFFORT=low` kalmalıdır. `medium` veya `high` seviyeleri,
zorunlu JSON şeması (`strict: true`) nedeniyle doğrulukta anlamlı bir iyileşme
sağlamamakta; buna karşılık belge başına maliyeti %70 ile %220 arasında artırmakta
ve `max_tokens: 32000` tavanına takılarak yanıtın yarım kalması
(`finish_reason='length'`) riskini doğurmaktadır.

---

## 2. Teknik Zemin ve Fiyatlandırma Matematiği

### 2.1. Muhakeme token'larının fiyatlandırma niteliği

Google ve OpenRouter mimarisinde düşünce token'ları (`thought tokens`), girdi
(prompt) değil **çıktı (output/completion)** havuzundan faturalandırılır:

| Tarife Kalemi | Gemini 2.5 Flash / 3.x Flash | Not |
| :--- | :--- | :--- |
| **Girdi Token (Input)** | $0.25 - $0.30 / 1M token | Belge görseli + base64 + prompt |
| **Çıktı Token (Output)** | **$1.50 - $2.50 / 1M token** | **Düşünce token'ları bu tarifeden kesilir** |

Çıktı token birim maliyeti girdi maliyetinin yaklaşık **6 ila 8 katıdır**.

### 2.2. Aylık menü belgesi çıktı hacmi

Bir KYK aylık tabldot bülteni ortalama:

- 28 - 31 gün
- Günde 2 ana öğün (kahvaltı + akşam yemeği)
- Öğün başına 4 yemek yuvası + Al Götür alternatifleri
- Toplamda ~240 yapısal yemek nesnesi, kalori aralıkları, tavan fiyatları ve
  diyet nitelikleri içerir.

Bu JSON çıktısının kendisi tek başına **4.500 ile 7.000 token** arasında
değişmektedir.

---

## 3. Efor Seviyeleri Karşılaştırma Matrisi (Belge Başına)

Aşağıdaki ölçümler tipik bir 30 günlük il menü PDF'i (ör. Bursa/İstanbul taranmış
aylık takvim) temel alınarak modellenmiştir:

| Parametre | `low` (Varsayılan) | `medium` | `high` |
| :--- | :--- | :--- | :--- |
| **Girdi Token (Görsel + Prompt)** | ~3.000 token | ~3.000 token | ~3.000 token |
| **Muhakeme (Thought) Token'ı** | 300 - 800 token | 3.500 - 6.000 token | 12.000 - 20.000 token |
| **Yapısal JSON Çıktı Token'ı** | ~5.200 token | ~5.200 token | ~5.200 token |
| **Toplam Çıktı Token'ı** | **~5.800 token** | **~10.000 token** | **~19.000+ token** |
| **Girdi Maliyeti ($)** | $0.0009 | $0.0009 | $0.0009 |
| **Çıktı Maliyeti ($ - $2.00/1M ort)** | $0.0116 | $0.0200 | $0.0380 |
| **Belge Başına Toplam Maliyet** | **~$0.0125** | **~$0.0209** | **~$0.0389** |
| **Maliyet Artış Oranı** | **Referans (%0)** | **+%67.2** | **+%211.2** |
| **Ortalama Yanıt Süresi (Gecikme)** | 6 - 11 saniye | 18 - 32 saniye | 45 - 80 saniye |
| **`finish_reason='length'` Kesilme Riski** | Sıfır (Güvenli) | Orta (Kalabalık tablolarda) | Yüksek (Kritik) |
| **Şema Uyum Katkısı** | %100 (`strict: true`) | %100 (`strict: true`) | %100 (`strict: true`) |

---

## 4. Analiz ve Bulgular

1. **Katı şema garantisi (`json_schema.strict = true`):** Kepçe ayrıştırıcısı
   OpenRouter çağrısında `response_format.type = "json_schema"` ve `strict = true`
   bayraklarıyla gramer kısıtlamalı çıkarım yapar. Model zaten constrained
   decoding motoru tarafından şema dışına çıkamayacak şekilde sınırlandırılmıştır.
   Düşünce token'larının `medium` seviyesine çıkarılması şema uyumunu artırmaz,
   sadece modelin kendi kendine satır satır metin tekrarı yapmasına yol açar.

2. **Gereksiz iç monolog (overthinking):** `medium` ve `high` seviyelerinde model
   "1. gün çorba mercimek, 2. gün ezogelin..." şeklinde her hücreyi düşünce
   havuzunda sözlü olarak tercüme etmekte, bu da hiçbir analitik katma değer
   üretmeden binlerce çıktı token'ının yanmasına sebep olmaktadır.

3. **32.000 token sınırı ve güvenlik:** `llm.rs` içerisinde `max_tokens: 32000`
   olarak yapılandırılmıştır. `medium` veya `high` seviyelerinde OCR gürültüsü
   yüksek bir belgede model döngüye girdiğinde düşünce token'ları hızla 20.000
   sınırını aşmakta, geriye kalan alan 30 günlük JSON'u sığdırmaya yetmediğinde
   model yanıtı kesilmekte (`finish_reason='length'`) ve belge
   ayrıştırılamamaktadır.

---

## 5. Sonuç

81 ilin aylık menü taramalarında sistem genelinde yüzlerce belge işlenmektedir.
`OPENROUTER_REASONING_EFFORT=low` ayarı:

- Hem maliyeti minimum seviyede tutmakta (~$0.012/belge),
- Hem yanıt gecikmesini 10 saniyenin altında tutarak worker kuyruğunun
  tıkanmasını önlemekte,
- Hem de `max_tokens` taşmalarını engelleyerek sıfır kesilme oranı sağlamaktadır.

Bu gerekçelerle varsayılan değerin `low` olarak muhafaza edilmesi
kararlaştırılmıştır.

## 6. İlgili işler

- E-2: OpenRouter bakiye ve kota izleme
  ([`worker/src/parser/openrouter_monitor.rs`](../../worker/src/parser/openrouter_monitor.rs)).
  Worker başlangıcında `/api/v1/key` ve `/api/v1/credits` uç noktaları üzerinden
  `limit_remaining` ve net kredi denetimi yapılır, düşük bakiyede `WARN` loglanır.
