/**
 * Backend (Rust) ile birebir aynı hash zinciri doğrulama algoritması.
 * 
 * Backend payload formatı:
 *   "{YYYY-MM-DD}:{city_id}:{meal_type}:{dish_id_1,dish_id_2,...}:{previous_hash|GENESIS}"
 * 
 * Bu fonksiyon, backend'in ImmutableStore::compute_menu_hash fonksiyonunun
 * JavaScript karşılığıdır.
 */

/**
 * Bir menü kaydının hash zincirindeki hash'ini doğrular.
 * @param {string} expectedHash - Veritabanındaki (merkle_root) beklenen hash
 * @param {string} serveDate - ISO tarih formatı "YYYY-MM-DD"
 * @param {number} cityId - Şehir ID'si
 * @param {string} mealType - "breakfast", "lunch" veya "dinner"
 * @param {number[]} sortedDishIds - Sıralı yemek alias ID'leri
 * @param {string|null} previousHash - Zincirdeki önceki menünün hash'i (null ise GENESIS)
 * @returns {Promise<boolean>} Hash eşleşiyorsa true
 */
export async function verifyMenuHash(expectedHash, serveDate, cityId, mealType, sortedDishIds, previousHash) {
  const prev = previousHash || 'GENESIS';
  const dishesStr = sortedDishIds.join(',');
  
  const payload = `${serveDate}:${cityId}:${mealType}:${dishesStr}:${prev}`;
  
  const encoder = new TextEncoder();
  const data = encoder.encode(payload);
  const hashBuffer = await window.crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const calculatedHash = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  
  return calculatedHash === expectedHash;
}

/**
 * Bir menü kaydının hash'ini hesaplar (doğrulama yapmadan).
 * @returns {Promise<string>} Hesaplanan SHA-256 hash
 */
export async function computeMenuHash(serveDate, cityId, mealType, sortedDishIds, previousHash) {
  const prev = previousHash || 'GENESIS';
  const dishesStr = sortedDishIds.join(',');
  
  const payload = `${serveDate}:${cityId}:${mealType}:${dishesStr}:${prev}`;
  
  const encoder = new TextEncoder();
  const data = encoder.encode(payload);
  const hashBuffer = await window.crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}
