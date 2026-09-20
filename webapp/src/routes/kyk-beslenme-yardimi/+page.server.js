import pricingData from "@/lib/data/pricing/istanbul_2025_2026.json";
import { evaluateTrayPersona } from "@/lib/utils/trayPersona.js";

/** @type {import('./$types').PageServerLoad} */
export function load({ url }) {
  const sepetParam = url.searchParams.get("sepet");
  const mealParam = url.searchParams.get("ogun") === "breakfast" ? "breakfast" : "dinner";

  if (!sepetParam) {
    return {
      ogImage: "https://kepce.org/api/v1/public/og/page/rehber",
      seoTitle: "KYK Beslenme Yardımı | Kepçe",
      seoDescription:
        "KYK yurtlarında kahvaltı ve akşam yemeği için tanımlanan günlük beslenme yardımı ve yemekhane harcama kuralları.",
    };
  }

  const tray = {};
  const pairs = sepetParam.split(",");
  for (const pair of pairs) {
    const [id, qtyStr] = pair.split(":");
    const qty = parseInt(qtyStr, 10);
    if (id && qty > 0 && pricingData.items.some((i) => i.id === id)) {
      tray[id] = qty;
    }
  }

  const allowance = pricingData.defaultAllowances[mealParam] || 105;
  const { activePersona, summaryText } = evaluateTrayPersona(
    tray,
    pricingData.items,
    allowance
  );

  const totalPrice = Object.entries(tray).reduce((acc, [id, qty]) => {
    const item = pricingData.items.find((i) => i.id === id);
    return acc + (item ? item.price * qty : 0);
  }, 0);

  const mealLabel = mealParam === "breakfast" ? "Kahvaltı" : "Akşam Yemeği";
  const title = activePersona ? activePersona.title : "KYK Tepsisi";
  const sub1 = `${totalPrice.toFixed(0)} TL · ${mealLabel}`;

  const ogParams = new URLSearchParams({
    title,
    sub1,
  });
  if (summaryText) {
    ogParams.set("sub2", summaryText);
  }

  return {
    ogImage: `https://kepce.org/api/v1/public/og/tepsi?${ogParams.toString()}`,
    seoTitle: activePersona
      ? `${activePersona.title} (${totalPrice.toFixed(0)} TL) | Kepçe`
      : `KYK Tepsisi (${totalPrice.toFixed(0)} TL) | Kepçe`,
    seoDescription: summaryText
      ? `KYK yemekhanesi tepsi simülasyonu: ${summaryText.replace(/\n/g, ", ")} (${totalPrice.toFixed(0)} TL)`
      : "KYK yurtlarında yemekhane tepsi simülatörü ve tavan fiyat tarifesi.",
  };
}
