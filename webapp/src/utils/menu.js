/**
 * Kaynak sitelerden yutulmuş "menü yok" placeholder metinlerini tespit eder
 * (örn. "Veri yok. Menüye sahipseniz ... mail atabilirsiniz"). Bu satırlar
 * yemek değil duyurudur; UI'da empty-state'e düşmeli.
 */
export function isPlaceholderDishText(name) {
  if (typeof name !== "string") return false;
  // Site navigasyon/başlık kalıntıları da duyurudur, yemek değildir
  // (örn. "←İstanbul KYK Menüsü", "Kahvaltı Yemek Listesi", "Gün Menüsü").
  if (/^(?:[-–-•*\s]*)(?:[←→↞↠]\s*)/.test(name)) return true;
  return /veri yok|sahipseniz|mail atabilirsiniz|e-?posta|yemek listesi|kyk men[üu]s?[üu]?|^g[üu]n men[üu]s[üu]$|kahvalt[ıi]ak[şs]am/i.test(name);
}

/**
 * Normalize raw items from the API to handle the different response formats
 * (items, dishes, foods) and ensure a uniform format for the UI.
 * Placeholder duyuru satırları (isPlaceholderDishText) elenir.
 */
export function normalizeItems(menu) {
  if (Array.isArray(menu.items) && menu.items.length > 0) {
    return menu.items
      .filter(i => !isPlaceholderDishText(i.master_data ? i.master_data.name : (i.raw_name || i.name)))
      .map(i => ({
        sort_order: i.order_index,
        name: i.master_data ? i.master_data.name : (i.raw_name || i.name),
        id: i.master_data ? i.master_data.dish_id : null,
        is_alternative: i.is_alternative,
        // Favori durumu kaynak item'dan dish'e taşınır ki UI reaktif
        // olarak güncellensin (handleFavorite menu.items'i mutasyona uğratır).
        my_favorite: i.my_favorite || false,
        dishes: [{
          id: i.master_data ? i.master_data.dish_id : null,
          name: i.master_data ? i.master_data.name : (i.raw_name || i.name),
          is_vegan: i.master_data ? !!i.master_data.is_vegan : false,
          is_vegetarian: i.master_data ? !!i.master_data.is_vegetarian : false,
          is_celiac: i.master_data ? !!i.master_data.is_celiac : false,
          is_alternative: i.is_alternative,
          my_favorite: i.my_favorite || false,
          weight: i.amount || null,
          price: i.price || null,
          calories: i.calories || null
        }]
      }));
  }
  if (Array.isArray(menu.dishes) && menu.dishes.length > 0) {
    return menu.dishes
      .filter(d => !isPlaceholderDishText(typeof d === "string" ? d : d?.name))
      .map(d => ({ dishes: [d] }));
  }
  if (Array.isArray(menu.foods) && menu.foods.length > 0) {
    return menu.foods
      .filter(f => !isPlaceholderDishText(f.name || f))
      .map(f => ({
        name: f.name || f,
        dishes: [{ id: typeof f === 'object' ? f.id : null, name: f.name || f }]
      }));
  }
  return [];
}

/**
 * Group menu items by sort_order to support inline alternative choices.
 */
export function groupItems(items) {
  if (!items || !Array.isArray(items)) return [];

  const grouped = [];
  const map = new Map(); // sort_order -> index in grouped

  for (const item of items) {
    let dishes = [];
    if (item.dishes && item.dishes.length > 0) {
      dishes = [...item.dishes];
    } else {
      dishes = [{ id: item.id ? `raw-${item.id}` : undefined, name: item.name }];
    }

    const sortOrder = item.sort_order !== undefined && item.sort_order !== null
      ? item.sort_order
      : (dishes[0] && dishes[0].sort_order !== undefined && dishes[0].sort_order !== null ? dishes[0].sort_order : null);

    if (sortOrder !== null && sortOrder !== undefined) {
      if (map.has(sortOrder)) {
        const existingIdx = map.get(sortOrder);
        grouped[existingIdx].dishes.push(...dishes);
      } else {
        map.set(sortOrder, grouped.length);
        grouped.push({
          ...item,
          dishes
        });
      }
    } else {
      grouped.push({
        ...item,
        dishes
      });
    }
  }

  return grouped;
}
