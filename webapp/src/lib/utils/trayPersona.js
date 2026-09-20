/**
 * KYK Tepsi Simülatörü - Dinamik Tepsi Kulpları (Tray Personas)
 * Tepsideki ürün kompozisyonunu, besin gruplarını ve fiyat dağılımını analiz ederek
 * tepsiye esprili unvanlar atayan saf kural motoru.
 */

function normalizeText(text) {
  if (!text) return "";
  return text.toLocaleLowerCase("tr-TR");
}

function isMeatItem(item) {
  const name = normalizeText(item.name);
  // Etsiz veya mercimekli ürünler et sayılmaz
  if (name.includes("etsiz") || name.includes("mercimek")) return false;

  const cat = item.category || "";
  if (cat === "ana_yemek_etli" || cat === "ana_yemek_tavuk") return true;

  const meatKeywords = [
    "kavurma",
    "köfte",
    "kofte",
    "tavuk",
    "döner",
    "doner",
    "sucuk",
    "kıymalı",
    "kiymali",
    "kuşbaşılı",
    "kusbasili",
    "salam",
    "etli",
    "dana",
    "kuzu",
  ];
  if (meatKeywords.some((k) => name.includes(k))) return true;
  return /\bet\b/i.test(name);
}

function isEggItem(item) {
  const name = normalizeText(item.name);
  return name.includes("yumurta") || name.includes("menemen") || name.includes("omlet");
}

function isDairyItem(item) {
  const name = normalizeText(item.name);
  const dairyKeywords = [
    "yoğurt",
    "yogurt",
    "cacık",
    "cacik",
    "ayran",
    "süt",
    "sut",
    "peynir",
    "kaşar",
    "kasar",
    "labne",
  ];
  return dairyKeywords.some((k) => name.includes(k));
}

function isCarbItem(item) {
  const cat = item.category || "";
  if (["pilav_makarna", "pide_hamur", "ekmek"].includes(cat)) return true;
  const name = normalizeText(item.name);
  const carbKeywords = [
    "pilav",
    "makarna",
    "pide",
    "börek",
    "borek",
    "poğaça",
    "pogaca",
    "açma",
    "acma",
    "simit",
    "tost",
    "patso",
    "sandviç",
    "sandvic",
    "lahmacun",
    "gözleme",
    "gozleme",
    "krep",
    "pankek",
    "kumpir",
    "pizza",
    "ekmek",
  ];
  return carbKeywords.some((k) => name.includes(k));
}

function isSweetItem(item) {
  const cat = item.category || "";
  if (["tatli", "tatlilar", "meyve", "meyveler"].includes(cat)) return true;
  const name = normalizeText(item.name);
  const sweetKeywords = [
    "tatlı",
    "tatli",
    "baklava",
    "pasta",
    "kek",
    "sütlaç",
    "sutlac",
    "çikolata",
    "cikolata",
    "kruvasan",
    "revani",
    "kemalpaşa",
    "kemalpasa",
    "kadayıf",
    "kadayif",
  ];
  return sweetKeywords.some((k) => name.includes(k));
}

function isLiquidItem(item) {
  const cat = item.category || "";
  if (["corba", "icecek", "icecekler"].includes(cat)) return true;
  const name = normalizeText(item.name);
  const liquidKeywords = [
    "çorba",
    "corba",
    "su",
    "çay",
    "cay",
    "kahve",
    "kola",
    "fanta",
    "gazoz",
    "soda",
    "limonata",
    "ayran",
  ];
  return liquidKeywords.some((k) => name.includes(k));
}

function isSolidBuffetItem(item) {
  const name = normalizeText(item.name);
  return (
    name.includes("tost") ||
    name.includes("patso") ||
    name.includes("sandviç") ||
    name.includes("sandvic") ||
    name.includes("kumru") ||
    name.includes("hamburger")
  );
}

function isBuffetItem(item) {
  const cat = item.category || "";
  if (["tost_sandvic", "icecek", "icecekler"].includes(cat)) return true;
  return isSolidBuffetItem(item) || isLiquidItem(item);
}

export const PERSONA_RULES = [
  {
    id: "laktoz_komasi",
    title: "Laktoz Koması",
    emoji: "🥛",
    description: "Sepette hem yoğurt hem cacık veya birden fazla süt ürünü var. Sindirim sistemine sabır diliyoruz.",
    priority: 90,
    predicate: (ctx) => {
      const dairyItems = ctx.trayItems.filter((i) => isDairyItem(i));
      if (dairyItems.length < 2) return false;
      const names = dairyItems.map((i) => normalizeText(i.name));
      const hasYogurt = names.some((n) => n.includes("yoğurt") || n.includes("yogurt"));
      const hasCacik = names.some((n) => n.includes("cacık") || n.includes("cacik"));
      if (hasYogurt && hasCacik) return true;
      return dairyItems.reduce((acc, i) => acc + i.qty, 0) >= 3;
    },
  },
  {
    id: "otobur_dehasi",
    title: "Otobur Dehası",
    emoji: "🥗",
    description: "Tek bir gram et veya tavuk olmadan bütçeyi kuruşu kuruşuna tam denkleştirme başarısı.",
    priority: 85,
    predicate: (ctx) => {
      if (ctx.totalPrice !== ctx.allowance || ctx.allowance <= 0) return false;
      if (ctx.trayItems.length < 2) return false;
      const hasMeat = ctx.trayItems.some((i) => isMeatItem(i));
      return !hasMeat;
    },
  },
  {
    id: "karbonhidrat_intihari",
    title: "Karbonhidrat İntiharı",
    emoji: "🥖",
    description: "Tepsinin tamamı un, hamur, pilav ve makarna türevlerinden oluşuyor.",
    priority: 82,
    predicate: (ctx) => {
      if (ctx.totalCount < 3) return false;
      const carbCount = ctx.trayItems
        .filter((i) => isCarbItem(i))
        .reduce((acc, i) => acc + i.qty, 0);
      return carbCount / ctx.totalCount >= 0.8;
    },
  },
  {
    id: "tek_kursun",
    title: "Tek Kurşunluk Rus Ruleti",
    emoji: "🎯",
    description: "Tek bir ürün bütçenin aslan payını yuttu, geriye neredeyse sadece su parası kaldı.",
    priority: 80,
    predicate: (ctx) => {
      if (ctx.allowance <= 0) return false;
      if (ctx.totalCount <= 1) return false;
      const maxSingleTotal = Math.max(...ctx.trayItems.map((i) => i.price));
      return maxSingleTotal >= ctx.allowance * 0.75;
    },
  },
  {
    id: "protein_baronu",
    title: "Protein Baronu",
    emoji: "🥩",
    description: "Bütçeyi son kuruşuna kadar et, tavuk ve yumurtaya yatıran sporcu tepsisi.",
    priority: 70,
    predicate: (ctx) => {
      if (ctx.totalPrice <= 0 || ctx.totalCount <= 1) return false;
      const proteinTotal = ctx.trayItems
        .filter((i) => isMeatItem(i) || isEggItem(i))
        .reduce((acc, i) => acc + i.itemTotal, 0);
      return proteinTotal / ctx.totalPrice >= 0.7;
    },
  },
  {
    id: "seker_komasi",
    title: "Şeker Koması",
    emoji: "🍰",
    description: "Akşam yemeğini tamamen tatlı ve şeker festivaline dönüştürme hali.",
    priority: 65,
    predicate: (ctx) => {
      if (ctx.trayItems.length < 2 || ctx.totalPrice <= 0) return false;
      const sweetTotal = ctx.trayItems
        .filter((i) => isSweetItem(i))
        .reduce((acc, i) => acc + i.itemTotal, 0);
      return sweetTotal / ctx.totalPrice >= 0.75;
    },
  },
  {
    id: "sivi_diyeti",
    title: "Sıvı Beslenmesi",
    emoji: "🥣",
    description: "Katı gıdaya küsüp sadece çorba ve içecekle günü kurtarma girişimi.",
    priority: 62,
    predicate: (ctx) => {
      if (ctx.trayItems.length < 2) return false;
      const allLiquid = ctx.trayItems.every((i) => isLiquidItem(i));
      return allLiquid;
    },
  },
  {
    id: "kantin_faresi",
    title: "Kantin Faresi",
    emoji: "🥪",
    description: "Yemekhane sırasına girmeyip büfeden ne bulduysa tepsiye dolduran öğrenci.",
    priority: 60,
    predicate: (ctx) => {
      if (ctx.trayItems.length < 2) return false;
      const hasSolidBuffet = ctx.trayItems.some((i) => isSolidBuffetItem(i));
      const allBuffet = ctx.trayItems.every((i) => isBuffetItem(i));
      return hasSolidBuffet && allBuffet;
    },
  },
  {
    id: "tek_tabanca",
    title: "Tek Tabanca",
    emoji: "🏹",
    description: "Yanına ekmek veya su bile almadan tek bir porsiyonla doymaya çalışan minimalist.",
    priority: 50,
    predicate: (ctx) => {
      return (
        ctx.trayItems.length === 1 &&
        ctx.totalCount === 1 &&
        ctx.totalPrice >= ctx.allowance * 0.5
      );
    },
  },
];

/**
 * Tepsi içeriğini değerlendirir.
 * @param {Record<string, number>} tray - { [itemId]: quantity }
 * @param {Array<Object>} allItems - Fiyat tarifesindeki tüm ürünler
 * @param {number} allowance - Günlük öğün kotası (TL)
 * @returns {{ activePersona: Object|null, matchingPersonas: Array<Object>, summaryText: string }}
 */
export function evaluateTrayPersona(tray, allItems, allowance) {
  const trayEntries = Object.entries(tray || {}).filter(([, qty]) => qty > 0);
  if (trayEntries.length === 0) {
    return { activePersona: null, matchingPersonas: [], summaryText: "" };
  }

  const trayItems = trayEntries
    .map(([id, qty]) => {
      const item = allItems.find((i) => i.id === id);
      if (!item) return null;
      return {
        ...item,
        qty,
        itemTotal: item.price * qty,
      };
    })
    .filter(Boolean);

  const totalPrice = trayItems.reduce((acc, i) => acc + i.itemTotal, 0);
  const totalCount = trayItems.reduce((acc, i) => acc + i.qty, 0);
  const difference = totalPrice - allowance;

  const context = {
    trayItems,
    totalPrice,
    totalCount,
    difference,
    allowance,
  };

  const matchingPersonas = PERSONA_RULES.filter((rule) => {
    try {
      return rule.predicate(context);
    } catch {
      return false;
    }
  }).sort((a, b) => b.priority - a.priority);

  const activePersona = matchingPersonas[0] || null;

  // Akıllı 2 satırlı ürün özeti
  const itemNames = trayItems.map((i) => (i.qty > 1 ? `${i.name} (x${i.qty})` : i.name));
  let summaryText = "";
  if (itemNames.length <= 3) {
    summaryText = itemNames.join(" · ");
  } else if (itemNames.length <= 5) {
    const half = Math.ceil(itemNames.length / 2);
    summaryText = `${itemNames.slice(0, half).join(" · ")}\n${itemNames.slice(half).join(" · ")}`;
  } else {
    // 6 veya daha fazla ürün varsa ikinci satıra sayaç
    const line1 = itemNames.slice(0, 3).join(" · ");
    const remaining = itemNames.length - 3;
    summaryText = `${line1}\n${itemNames[3]} · +${remaining - 1} ürün daha`;
  }

  return {
    activePersona,
    matchingPersonas,
    summaryText,
  };
}
