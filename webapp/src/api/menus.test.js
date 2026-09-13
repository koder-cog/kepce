import { describe, it, expect } from "vitest";
import { normalizeMenu } from "./menus.js";

describe("normalizeMenu", () => {
  it("standart menüyü ve içerisindeki alternatifleri doğru normalleştirir", () => {
    const rawData = {
      id: 1,
      city_slug: "istanbul",
      serve_date: "2026-09-13",
      meal_type: "dinner",
      source_type: "kepce",
      items: [
        {
          order_index: 0,
          raw_name: "Ezogelin Çorbası",
          is_alternative: false,
        },
      ],
      alternatives: [
        {
          id: 99,
          source_type: "kykyemek",
          meal_type: "dinner",
          items: [
            {
              order_index: 0,
              raw_name: "Yayla Çorbası",
              is_alternative: false,
            },
          ],
        },
      ],
    };

    const normalized = normalizeMenu(rawData);
    expect(normalized.date).toBe("2026-09-13");
    expect(normalized.items).toHaveLength(1);
    expect(normalized.items[0].raw_name).toBe("Ezogelin Çorbası");

    expect(normalized.alternatives).toHaveLength(1);
    expect(normalized.alternatives[0].source_type).toBe("kykyemek");
    expect(normalized.alternatives[0].items).toHaveLength(1);
    expect(normalized.alternatives[0].items[0].raw_name).toBe("Yayla Çorbası");
  });

  it("alternatives dizisi olmadığında boş dizi döner", () => {
    const rawData = {
      id: 2,
      serve_date: "2026-09-13",
      meal_type: "breakfast",
      items: [],
    };

    const normalized = normalizeMenu(rawData);
    expect(normalized.alternatives).toEqual([]);
  });

  it("ana menüyle aynı veya kendi arasında mükerrer alternatifleri eler", () => {
    const rawData = {
      id: 3,
      serve_date: "2026-09-13",
      meal_type: "dinner",
      items: [
        { order_index: 0, raw_name: "Ezogelin Çorbası", is_alternative: false },
        { order_index: 1, raw_name: "Pirinç Pilavı", is_alternative: false },
      ],
      alternatives: [
        // Ana menüyle birebir aynı olan alternatif elenmeli
        {
          id: 101,
          source_type: "kykyemek",
          items: [
            { order_index: 0, raw_name: "Ezogelin Çorbası", is_alternative: false },
            { order_index: 1, raw_name: "Pirinç Pilavı", is_alternative: false },
          ],
        },
        // Farklı alternatif kabul edilmeli
        {
          id: 102,
          source_type: "kykmenum.com",
          items: [
            { order_index: 0, raw_name: "Mercimek Çorbası", is_alternative: false },
            { order_index: 1, raw_name: "Bulgur Pilavı", is_alternative: false },
          ],
        },
        // Yukarıdaki alternatifle aynı olan mükerrer kopya elenmeli
        {
          id: 103,
          source_type: "kykmenu.com.tr",
          items: [
            { order_index: 0, raw_name: "Mercimek Çorbası", is_alternative: false },
            { order_index: 1, raw_name: "Bulgur Pilavı", is_alternative: false },
          ],
        },
      ],
    };

    const normalized = normalizeMenu(rawData);
    expect(normalized.alternatives).toHaveLength(1);
    expect(normalized.alternatives[0].id).toBe(102);
  });
});
