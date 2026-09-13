# Veri Modelleri

Kepçe API yanıtlarında kullanılan temel veri modelleri ve alan tanımları.

## 1. Menü Modeli (`MenuResponseDto`)

Belirli bir şehir, tarih ve öğün için sunulan menünün ana veri yapısıdır.

| Alan Adı | Tip | Açıklama |
| :--- | :--- | :--- |
| `id` | `integer` | Menünün sistemdeki benzersiz kimlik numarası. |
| `city_name` | `string` | Şehir adı (örnek: `"İstanbul"`). |
| `city_slug` | `string` | Şehir kısa adı (örnek: `"istanbul"`). |
| `serve_date` | `string` | Servis tarihi (`YYYY-MM-DD`). |
| `meal_type` | `string` | Öğün türü: `"breakfast"`, `"lunch"` veya `"dinner"`. |
| `source_type` | `string` | Verinin temin edildiği kaynak tipi (`"ocr"`, `"table"` veya `"manual"`). |
| `status` | `string` | Moderasyon onay durumu: `"approved"`, `"pending"` veya `"rejected"`. |
| `bot_commentary` | `string \| null` | Menü hakkında moderasyon veya sistem notu. |
| `items` | `Array<MenuItemDto>` | Menüye dahil standart yemeklerin listesi. |
| `takeaways` | `Array<TakeawayMenuDto>` | Günün Al Götür paket alternatifleri listesi. |
| `comment_count` | `integer` | Menüye yapılan onaylı yorum sayısı. |
| `vote_count` | `integer` | Menü için kullanılan toplam oy sayısı. |
| `rating_sum` | `integer` | Pozitif ve negatif oyların toplam dengesi. |
| `my_vote` | `string \| null` | İstek sahibi oturum açmışsa kullandığı oy (`"positive"`, `"negative"` veya `null`). |
| `calorie_range_min` | `integer \| null` | Menü için bildirilen en düşük kalori değeri. |
| `calorie_range_max` | `integer \| null` | Menü için bildirilen en yüksek kalori değeri. |
| `calorie_range` | `string \| null` | Menü bültenindeki resmi kalori metni (örnek: `"600 - 800 kcal"`). |
| `calculated_calories` | `integer \| null` | Menüdeki yemeklerin porsiyon kalorilerinden hesaplanan toplam değer. |

## 2. Yemek Modeli (`MenuItemDto`)

Menü veya Al Götür paketi içindeki her bir yemek bileşeni.

| Alan Adı | Tip | Açıklama |
| :--- | :--- | :--- |
| `order_index` | `integer` | Menüdeki servis sırası indeksi (0 tabanlı). |
| `raw_name` | `string` | Kaynak listeden okunan ham yemek adı. |
| `is_alternative` | `boolean` | Ana yemek yerine seçilebilen alternatif seçenek olup olmadığı. |
| `amount` | `string \| null` | Gramaj veya porsiyon bilgisi (örnek: `"250 g"`, `"1 adet L boy"`). |
| `calories` | `integer \| null` | Yemeğe ait porsiyon kalori değeri (örnek: `180`). |
| `price` | `number \| null` | Resmi tavan fiyat listesindeki porsiyon tutarı (örnek: `35.0`). |
| `category` | `string \| null` | Yemek kategorisi (örnek: `"Çorba"`, `"Ana Yemek"`, `"Kahvaltılık Sıcak"`). |
| `master_data` | `DishMasterDataDto \| null` | Yemek merkezi yemek kataloğuyla eşleşmişse ek besin ve beğeni verisi. |

## 3. Onaylanmış Yemek Verisi (`DishMasterDataDto`)

Yemeğin merkezi veritabanı kaydıyla eşleştiği durumlarda döner.

| Alan Adı | Tip | Açıklama |
| :--- | :--- | :--- |
| `dish_id` | `integer` | Merkezi yemek kataloğundaki tekil kimlik. |
| `name` | `string` | Yemeğin standart ve düzeltilmiş adı (örnek: `"Mercimek Çorbası"`). |
| `is_celiac` | `boolean` | Glutensiz ve çölyak diyetine uygunluk durumu. |
| `is_vegan` | `boolean` | Vegan beslenmeye uygunluk durumu. |
| `is_vegetarian` | `boolean` | Vejetaryen beslenmeye uygunluk durumu. |
| `estimated_calories` | `integer \| null` | Standart porsiyon için tahmin edilen kalori. |
| `total_votes` | `integer` | Yemek için kullanılan toplam oy sayısı. |
| `positive_votes` | `integer` | Pozitif oy sayısı. |
| `negative_votes` | `integer` | Negatif oy sayısı. |
| `like_ratio` | `number \| null` | Beğeni oranı (0.0 ile 1.0 arasında). |
| `dislike_ratio` | `number \| null` | Beğenmeme oranı (0.0 ile 1.0 arasında). |

## 4. Al Götür Paket Modeli (`TakeawayMenuDto`)

Yurtlarda kahvaltı veya akşam yemeği yerine sunulan paket alternatifleri.

| Alan Adı | Tip | Açıklama |
| :--- | :--- | :--- |
| `name` | `string` | Paketin resmi adı (örnek: `"1. Soğuk Sandviç Paketi"`). |
| `items` | `Array<MenuItemDto>` | Paket içeriğindeki yiyecek ve içeceklerin listesi. |

## 5. Şehir Modeli (`CityResponseDto`)

| Alan Adı | Tip | Açıklama |
| :--- | :--- | :--- |
| `id` | `integer` | Şehir kimlik numarası (plaka kodu). |
| `name` | `string` | Şehrin Türkçe adı (örnek: `"İstanbul"`). |
| `slug` | `string` | URL ve sorgularda kullanılan kısa ad (`"istanbul"`). |
| `has_celiac` | `boolean` | Şehirde çölyak menüsü desteğinin bulunup bulunmadığı. |
