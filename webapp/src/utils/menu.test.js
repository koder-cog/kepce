import { describe, it, expect } from "vitest";
import { isPlaceholderDishText, normalizeItems } from "./menu.js";

describe("isPlaceholderDishText", () => {
  it("duyuru metinlerini yakalar", () => {
    expect(isPlaceholderDishText("Veri yok. Menüye sahipseniz bize yazın")).toBe(true);
    expect(isPlaceholderDishText("Lütfen mail atabilirsiniz")).toBe(true);
    expect(
      isPlaceholderDishText(
        "14 Eylül İtibarıyla Yeni Dönem Listeleri Girilmeye Başlanacak. Herkese Yeni Dönemde Başarılar Dileriz"
      )
    ).toBe(true);
    expect(isPlaceholderDishText("Yeni dönemde tüm öğrencilere başarılar dileriz")).toBe(true);
    expect(isPlaceholderDishText("Duyuru: Ramazan ayı boyunca yemek saatleri...")).toBe(true);
    expect(
      isPlaceholderDishText(
        "Listeler Elimize Ulaşır Ulaşmaz Siteye Eklenecektir. Elinizde Liste Mevcutsa Üye Olarak Bize İletebilir, Ödüllerden Yararlanabilirsiniz"
      )
    ).toBe(true);
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

describe("normalizeItems", () => {
  it("master_data üzerindeki diyet ve duygu istatistiklerini doğru aktarır", () => {
    const mockMenu = {
      items: [
        {
          order_index: 0,
          master_data: {
            dish_id: 101,
            name: "Zeytinyağlı Pırasa",
            is_celiac: false,
            is_vegan: true,
            is_vegetarian: true,
            estimated_calories: 140,
            total_votes: 15,
            positive_votes: 3,
            negative_votes: 12,
            dislike_ratio: 0.8,
            like_ratio: 0.2
          },
          amount: "150 gr",
          price: "25 TL",
          calories: 140,
          is_alternative: false
        }
      ]
    };

    const normalized = normalizeItems(mockMenu);
    expect(normalized).toHaveLength(1);
    const dish = normalized[0].dishes[0];
    expect(dish.name).toBe("Zeytinyağlı Pırasa");
    expect(dish.is_vegan).toBe(true);
    expect(dish.is_celiac).toBe(false);
    expect(dish.total_votes).toBe(15);
    expect(dish.dislike_ratio).toBe(0.8);
    expect(dish.like_ratio).toBe(0.2);
  });

  it("bitişik artı içeren yemek isimlerini arayüzde ferahlatır", () => {
    const mockMenu = {
      items: [
        {
          order_index: 0,
          raw_name: "Bal+tereyağ",
          is_alternative: false
        }
      ]
    };
    const normalized = normalizeItems(mockMenu);
    expect(normalized[0].name).toBe("Bal + tereyağ");
    expect(normalized[0].dishes[0].name).toBe("Bal + tereyağ");
  });

  it("backend tarafından dönen temiz name ve calories alanlarını doğrudan kullanır", () => {
    const mockMenu = {
      items: [
        {
          order_index: 0,
          name: "Mercimek Çorbası",
          calories: 220,
          is_alternative: false
        }
      ]
    };
    const normalized = normalizeItems(mockMenu);
    expect(normalized[0].name).toBe("Mercimek Çorbası");
    expect(normalized[0].dishes[0].name).toBe("Mercimek Çorbası");
    expect(normalized[0].dishes[0].calories).toBe(220);
  });
});

