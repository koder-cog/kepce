# Güvenlik Politikası (Security Policy)

Kepçe projesi, öğrenci ve topluluk verilerinin güvenliğini, mahremiyetini ve kriptografik veri bütünlüğünü en yüksek öncelik olarak kabul eder.

## Desteklenen Sürümler

Aşağıdaki tabloda güvenlik güncellemeleri alan sürümler listelenmiştir:

| Sürüm | Destek Durumu |
| :--- | :--- |
| `v0.1.x` (main) | Destekleniyor |
| `< 0.1.0` | Desteklenmiyor |

---

## Güvenlik Açığı Bildirimi (Reporting a Vulnerability)

Eğer Kepçe altyapısında, API servislerinde, kimlik doğrulama katmanında veya veri madenciliği bileşenlerinde potansiyel bir güvenlik açığı tespit ettiyseniz:

1. **Lütfen Açığı Herkese Açık Olarak (Public Issue) Paylaşmayınız.**
2. Açık bildirimlerinizi doğrudan **`guvenlik@kepce.org`** veya **`yasal@kepce.org`** adresine e-posta yoluyla iletiniz.
3. Bildiriminizde lütfen aşağıdaki detaylara yer veriniz:
   - Etkilenen bileşen veya API rotası
   - Adım adım yeniden üretme (PoC) kılavuzu
   - Açığın yaratabileceği potansiyel etki (veri sızıntısı, yetki yükseltme, DoS vb.)
   - Varsa önerdiğiniz çözüm veya yama

---

## Yanıt ve Süreç Taahhüdü

* **İlk Yanıt:** Bildiriminiz en geç **48 saat** içerisinde incelenip tarafınıza teyit yanıtı verilir.
* **Değerlendirme ve Yama:** Güvenlik açığı doğrulandıktan sonra kritiklik derecesine göre önceliklendirilerek hızlı yama geliştirilir ve yayına alınır.
* **Açıklama (Coordinated Disclosure):** Düzeltme production ortamında yayınlandıktan sonra katkınız güvenlik teşekkür listesinde (Security Hall of Fame / Release Notes) memnuniyetle belirtilir.
