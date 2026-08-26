# Kepçe Katkıda Bulunma Kılavuzu

Kepçe'ye katkıda bulunmak istediğiniz için teşekkür ederiz! Açık kaynaklı, şeffaf ve güvenilir bir topluluk inşa etmek için her türlü katkı değerlidir.

## Geliştirme Ortamı Kurulumu

### Gereksinimler
- **Rust:** `1.80+` (Cargo, Clippy, Rustfmt)
- **Node.js:** `20+` (npm)
- **Veritabanı:** PostgreSQL `15+` (veya Docker / Podman)

### Hızlı Başlangıç

1. Depoyu forklayın ve klonlayın:
   ```bash
   git clone https://github.com/<kullanici-adiniz>/kepce.git
   cd kepce
   ```
2. Örnek ortam dosyasını kopyalayın:
   ```bash
   cp .env.example .env
   ```
3. Yerel servisleri başlatın:
   ```bash
   ./manage.sh start
   ```

## Kod Standartları ve Prensipleri

### 1. Rust (Backend ve Worker)
- Kodlama standartları için `cargo fmt --all` ve `cargo clippy --workspace --all-targets` kullanılmalıdır.
- Güvenlik açıklarını ve panikleri önlemek adına production kodlarında kontrolsüz `unwrap()` yerine uygun `Result` / `Option` eşlemeleri (`?` operatörü) tercih edilmelidir.
- Tüm iş mantığı ve kritik ayrıştırıcılar için birim testleri (`cargo test --workspace`) yazılmalıdır.

### 2. SvelteKit ve Vanilla CSS (Frontend)
- Yeni arayüz geliştirirken `webapp/src/styles/` altındaki mevcut CSS sınıfları ve değişkenler (`main.css`) kullanılmalıdır.
- HTML elementlerinde inline statik `style="..."` tanımları kullanılmaz; CSS sınıfları ve CSS custom property pattern'i (`style="--var-name: {value}"`) kullanılır.
- Kod kalitesi ve tip kontrolü için `cd webapp && npm run check` çalıştırılmalıdır.

## Katkı ve PR Süreci

1. Değişikliğiniz için anlamlı bir dal (branch) açın:
   ```bash
   git checkout -b feature/yeni-ozellik
   # veya
   git checkout -b fix/hata-duzeltme
   ```
2. Conventional Commits standardına uygun, açıklayıcı commit mesajları yazın (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `build:`).
3. PR açmadan önce testlerin geçtiğinden emin olun:
   ```bash
   # Tüm kontrolleri (Clippy, Rust testleri, svelte-check, vitest, SSR smoke) tek seferde çalıştırmak için:
   bash scripts/ci-local.sh

   # Veya ayrı ayrı:
   cargo test --workspace
   cd webapp && npm run check && npm run test
   ```
4. PR açıklamanızda yapılan değişiklikleri, çözülen Issue numarasını ve test sonuçlarını özetleyin.

## Yasal Uyarı ve Lisans Taahhüdü

Gönderdiğiniz tüm katkılar projenin GNU Affero General Public License v3.0 lisansı altında lisanslanacaktır. PR göndererek kodunuzun bu şartlar altında dağıtılmasını kabul etmiş sayılırsınız.
