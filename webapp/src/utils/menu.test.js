import { describe, it, expect } from "vitest";
import { isPlaceholderDishText } from "./menu.js";

describe("isPlaceholderDishText", () => {
  it("duyuru metinlerini yakalar", () => {
    expect(isPlaceholderDishText("Veri yok. Menüye sahipseniz bize yazın")).toBe(true);
    expect(isPlaceholderDishText("Lütfen mail atabilirsiniz")).toBe(true);
  });

  it("site navigasyon kalıntılarını yakalar (kykmenu scrape çöpleri)", () => {
    expect(isPlaceholderDishText("- Kahvaltı Yemek Listesi")).toBe(true);
    expect(isPlaceholderDishText("- Akşam Yemeği Yemek Listesi")).toBe(true);
    expect(isPlaceholderDishText("←İstanbul KYK Menüsü")).toBe(true);
    expect(isPlaceholderDishText("←Adana KYK Menüsü")).toBe(true);
    expect(isPlaceholderDishText("Gün Menüsü")).toBe(true);
    expect(isPlaceholderDishText("KahvaltıAkşam")).toBe(true);
  });

  it("ok karakteriyle başlayan satırları yakalar", () => {
    expect(isPlaceholderDishText("→ Kayseri KYK Menüsü")).toBe(true);
    expect(isPlaceholderDishText("← Trabzon")).toBe(true);
  });

  it("gerçek yemek isimlerini temiz sayar", () => {
    expect(isPlaceholderDishText("Beyaz Peynir")).toBe(false);
    expect(isPlaceholderDishText("Haşlanmış Yumurta")).toBe(false);
    expect(isPlaceholderDishText("Mevsim Sebzeleri Söğüş")).toBe(false);
    expect(isPlaceholderDishText("Mercimek Çorbası")).toBe(false);
    expect(isPlaceholderDishText(123)).toBe(false);
    expect(isPlaceholderDishText(undefined)).toBe(false);
  });
});
