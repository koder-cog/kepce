import { describe, it, expect } from 'vitest';
import {
  USERNAME_ADJECTIVES,
  USERNAME_NOUNS,
  capitalize,
  generateRandomUsername,
  getCryptoRandomInt,
  isValidUsername
} from './username.js';

describe('username Utility', () => {
  it('sıfat havuzu yalnızca küçük ASCII harfler içermeli ve uzunlukları 10 karakteri aşmamalı', () => {
    expect(USERNAME_ADJECTIVES.length).toBeGreaterThanOrEqual(40);
    for (const adj of USERNAME_ADJECTIVES) {
      expect(adj).toMatch(/^[a-z]+$/);
      expect(adj.length).toBeLessThanOrEqual(10);
      expect(adj.length).toBeGreaterThanOrEqual(2);
    }
  });

  it('isim havuzu yalnızca küçük ASCII harfler içermeli ve uzunlukları 10 karakteri aşmamalı', () => {
    expect(USERNAME_NOUNS.length).toBeGreaterThanOrEqual(40);
    for (const noun of USERNAME_NOUNS) {
      expect(noun).toMatch(/^[a-z]+$/);
      expect(noun.length).toBeLessThanOrEqual(10);
      expect(noun.length).toBeGreaterThanOrEqual(2);
    }
  });

  it('capitalize ilk harfi büyütmeli ve kalanı küçük tutmalı', () => {
    expect(capitalize('tuzlu')).toBe('Tuzlu');
    expect(capitalize('MERCIMEK')).toBe('Mercimek');
    expect(capitalize('')).toBe('');
  });

  it('tüm sıfat ve isim kombinasyonları 4 basamaklı sayı ile 32 karakter sınırını asla aşmamalı', () => {
    for (const adj of USERNAME_ADJECTIVES) {
      for (const noun of USERNAME_NOUNS) {
        // En uzun senaryo: Adj + Noun + 9999 (PascalCase, ayracsiz)
        const totalLen = adj.length + noun.length + 4;
        expect(totalLen).toBeLessThanOrEqual(32);
        expect(totalLen).toBeGreaterThanOrEqual(3);
      }
    }
  });

  it('getCryptoRandomInt belirtilen aralıkta değer üretmeli', () => {
    for (let i = 0; i < 100; i++) {
      const val = getCryptoRandomInt(1000, 9999);
      expect(val).toBeGreaterThanOrEqual(1000);
      expect(val).toBeLessThanOrEqual(9999);
    }
  });

  it('generateRandomUsername geçerli ve Reddit tarzı [Sıfat][İsim][4_sayı] biçiminde adlar üretmeli', () => {
    for (let i = 0; i < 300; i++) {
      const name = generateRandomUsername();
      expect(isValidUsername(name)).toBe(true);
      expect(name.length).toBeGreaterThanOrEqual(3);
      expect(name.length).toBeLessThanOrEqual(32);
      expect(name).toMatch(/^[A-Z][a-z]+[A-Z][a-z]+\d{4}$/);
    }
  });

  it('isValidUsername yeni desene göre harfle başlama, tekil ayraç ve sınır kurallarını doğrulamalı', () => {
    // Uzunluk sınırları
    expect(isValidUsername('ab')).toBe(false); // 2 karakter
    expect(isValidUsername('a'.repeat(32))).toBe(true); // 32 karakter
    expect(isValidUsername('a'.repeat(33))).toBe(false); // 33 karakter

    // Geçerli biçimler
    expect(isValidUsername('Kullanici_123')).toBe(true);
    expect(isValidUsername('TuzluMercimek7893')).toBe(true);
    expect(isValidUsername('gurme_yemek_123')).toBe(true);
    expect(isValidUsername('gurme-yemek-123')).toBe(true); // Tire ayracı geçerli
    expect(isValidUsername('nobetci-ogrenci_1')).toBe(true);

    // Başta / sonda ayraç ve rakam yasağı
    expect(isValidUsername('_gurme')).toBe(false);
    expect(isValidUsername('gurme_')).toBe(false);
    expect(isValidUsername('-gurme')).toBe(false);
    expect(isValidUsername('gurme-')).toBe(false);
    expect(isValidUsername('123gurme')).toBe(false); // Harfle başlamalı

    // Ardışık ayraç yasağı
    expect(isValidUsername('gurme__yemek')).toBe(false);
    expect(isValidUsername('gurme--yemek')).toBe(false);
    expect(isValidUsername('gurme_-yemek')).toBe(false);

    // Geçersiz karakterler
    expect(isValidUsername('türkçe_yemek_123')).toBe(false);
    expect(isValidUsername('TuzluMercimek_İ')).toBe(false);
    expect(isValidUsername('ali veli')).toBe(false);
    expect(isValidUsername('ali.veli')).toBe(false);
  });
});
