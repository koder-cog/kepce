import { describe, it, expect } from "vitest";
import { evaluateTrayPersona } from "./trayPersona.js";

const mockItems = [
  { id: "yogurt", name: "Süzme Yoğurt", price: 20, category: "yan_yemek" },
  { id: "cacik", name: "Cacık", price: 15, category: "yan_yemek" },
  { id: "ayran", name: "Ayran", price: 10, category: "icecek" },
  { id: "sut", name: "Kutu Süt", price: 15, category: "icecek" },
  { id: "kofte", name: "Ekmek Arası Köfte", price: 94, category: "tost_sandvic" },
  { id: "tavuk", name: "Tavuk Sote", price: 65, category: "ana_yemek_tavuk" },
  { id: "kavurma", name: "Dana Kavurma", price: 85, category: "ana_yemek_etli" },
  { id: "haslanmis-yumurta", name: "Haşlanmış Yumurta", price: 11, category: "kahvaltilik" },
  { id: "omlet", name: "Kaşarlı Omlet", price: 24, category: "kahvaltilik" },
  { id: "mercimek-kofte", name: "Mercimekli Köfte", price: 40, category: "salata_meze" },
  { id: "etsiz-dolma", name: "Zeytinyağlı Dolma", price: 40, category: "salata_meze" },
  { id: "su", name: "Su (Pet Şişe)", price: 5, category: "icecek" },
  { id: "cay", name: "Çay", price: 5, category: "icecek" },
  { id: "corba", name: "Mercimek Çorbası", price: 20, category: "corba" },
  { id: "borek", name: "Ispanaklı Börek", price: 35, category: "pide_hamur" },
  { id: "pide", name: "Kaşarlı Pide", price: 80, category: "pide_hamur" },
  { id: "pilav", name: "Pirinç Pilavı", price: 25, category: "pilav_makarna" },
  { id: "makarna", name: "Fırın Makarna", price: 30, category: "pilav_makarna" },
  { id: "baklava", name: "Fıstıklı Baklava", price: 50, category: "tatli" },
  { id: "sutlac", name: "Fırın Sütlaç", price: 35, category: "tatli" },
  { id: "tost", name: "Kaşarlı Tost", price: 36, category: "tost_sandvic" },
  { id: "patso", name: "Patso Sandviç", price: 34, category: "tost_sandvic" },
];

describe("evaluateTrayPersona", () => {
  it("boş sepette unvan döndürmez", () => {
    const result = evaluateTrayPersona({}, mockItems, 105);
    expect(result.activePersona).toBeNull();
    expect(result.matchingPersonas).toHaveLength(0);
    expect(result.summaryText).toBe("");
  });

  it("yoğurt ve cacık bir aradaysa 'Laktoz Koması' tetiklenir", () => {
    const tray = { yogurt: 1, cacik: 1 };
    const result = evaluateTrayPersona(tray, mockItems, 105);
    expect(result.activePersona?.id).toBe("laktoz_komasi");
    expect(result.activePersona?.title).toBe("Laktoz Koması");
  });

  it("sıfır et ile tam 105 TL bütçe yakalandığında 'Otobur Dehası' tetiklenir", () => {
    // 40 + 40 + 20 + 5 = 105 TL (etsiz dolma, mercimek köfte, çorba, su)
    const tray = {
      "etsiz-dolma": 1,
      "mercimek-kofte": 1,
      corba: 1,
      su: 1,
    };
    const result = evaluateTrayPersona(tray, mockItems, 105);
    expect(result.activePersona?.id).toBe("otobur_dehasi");
    expect(result.activePersona?.title).toBe("Otobur Dehası");
  });

  it("tek bir ürün bütçenin %75'ini aşıyorsa 'Tek Kurşunluk Rus Ruleti' tetiklenir", () => {
    // 94 TL ekmek arası köfte (105'in %75'i = 78.75)
    const tray = { kofte: 1, su: 1 };
    const result = evaluateTrayPersona(tray, mockItems, 105);
    expect(result.activePersona?.id).toBe("tek_kursun");
    expect(result.activePersona?.title).toBe("Tek Kurşunluk Rus Ruleti");
  });

  it("ürünlerin %80'i hamur ve pilavdan oluşuyorsa 'Karbonhidrat İntiharı' tetiklenir", () => {
    const tray = {
      borek: 1,
      pilav: 1,
      makarna: 1,
      su: 1,
    };
    // borek, pilav, makarna carb (3/4 = %75, yetmez)
    // bir carb daha ekleyelim: 4/5 = %80
    tray.pide = 1;
    const result = evaluateTrayPersona(tray, mockItems, 105);
    expect(result.activePersona?.id).toBe("karbonhidrat_intihari");
  });

  it("harcamanın %70'i et ve yumurtadan oluşuyorsa 'Protein Baronu' tetiklenir", () => {
    const tray = {
      tavuk: 1, // 65 TL
      omlet: 1, // 24 TL
      su: 1,    // 5 TL
    };
    // Toplam 94 TL, et/yumurta 89 TL (89/94 = %94)
    const result = evaluateTrayPersona(tray, mockItems, 105);
    expect(result.activePersona?.id).toBe("protein_baronu");
  });

  it("harcamanın %75'i tatlıdan oluşuyorsa 'Şeker Koması' tetiklenir", () => {
    const tray = {
      baklava: 1, // 50 TL
      sutlac: 1,  // 35 TL
      su: 1,      // 5 TL
    };
    // Toplam 90 TL, tatlı 85 TL (%94)
    const result = evaluateTrayPersona(tray, mockItems, 105);
    expect(result.activePersona?.id).toBe("seker_komasi");
  });

  it("sadece büfe ürünlerinden oluşuyorsa 'Kantin Faresi' tetiklenir", () => {
    const tray = {
      tost: 1,
      patso: 1,
      ayran: 1,
    };
    const result = evaluateTrayPersona(tray, mockItems, 105);
    expect(result.activePersona?.id).toBe("kantin_faresi");
  });

  it("sadece çorba ve içecek varsa 'Sıvı Beslenmesi' tetiklenir", () => {
    const tray = {
      corba: 1,
      su: 2,
      cay: 1,
    };
    const result = evaluateTrayPersona(tray, mockItems, 105);
    expect(result.activePersona?.id).toBe("sivi_diyeti");
  });

  it("tek bir ürünle kotayı kapatmaya çalışıyorsa 'Tek Tabanca' tetiklenir", () => {
    const tray = { tavuk: 1 }; // 65 TL, 1 adet
    const result = evaluateTrayPersona(tray, mockItems, 105);
    expect(result.activePersona?.id).toBe("tek_tabanca");
  });

  it("ürün özetini ürün sayısına göre doğru formatlar", () => {
    // 2 ürün: tek satır
    const res2 = evaluateTrayPersona({ su: 1, cay: 1 }, mockItems, 105);
    expect(res2.summaryText).toBe("Su (Pet Şişe) · Çay");

    // 4 ürün: 2 satır
    const res4 = evaluateTrayPersona(
      { su: 1, cay: 1, corba: 1, tost: 1 },
      mockItems,
      105
    );
    expect(res4.summaryText).toContain("\n");

    // 6 ürün: 2. satırda sayaç
    const res6 = evaluateTrayPersona(
      { su: 1, cay: 1, corba: 1, tost: 1, borek: 1, baklava: 1 },
      mockItems,
      105
    );
    expect(res6.summaryText).toContain("+2 ürün daha");
  });
});
