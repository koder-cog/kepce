import { request, buildQuery } from './client.js';

/**
 * Backend'in farklı rotalar için farklı response şekilleri döndürmesi
 * muhtemel (menü kimliğiyle tekil çağrı, tarihe göre liste, aya göre liste).
 * Bu normalizer, hangi response gelirse gelsin MenuCard'ın beklediği
 * tek tipte bir menu objesine dönüştürür. Normalleştirilmiş obje
 * `items: [{ raw_name, master_data? }]` dizisi içerir; MenuCard
 * zaten bu şekli biliyor.
 */
function normalizeMenu(raw) {
  if (!raw || typeof raw !== 'object') return raw;

  // Bazı response'larda öğeler doğrudan `foods` veya `meals` altında
  // düz string listesi olarak gelebiliyor (kykyemek ham verisi gibi).
  const rawItemsText =
    raw.items_text ||
    raw.raw_items ||
    (Array.isArray(raw.foods) && raw.foods.every(f => typeof f === 'string') ? raw.foods : null) ||
    null;

  let items = raw.items;
  if (!Array.isArray(items) || items.length === 0) {
    if (Array.isArray(raw.dishes) && raw.dishes.length > 0) {
      // `dishes` → `items` dönüşümü
      items = raw.dishes.map((d, idx) => ({
        order_index: d.sort_order ?? idx,
        raw_name: d.name,
        is_alternative: !!d.is_alternative,
        master_data: d.id ? {
          dish_id: d.id,
          name: d.name,
          is_vegan: !!d.is_vegan,
          is_vegetarian: !!d.is_vegetarian,
          is_celiac: !!d.is_celiac,
          estimated_calories: d.estimated_calories,
        } : null,
      }));
    } else if (Array.isArray(raw.foods) && raw.foods.length > 0) {
      // String listesi → `items` dönüşümü (kykyemek scrape)
      items = raw.foods.map((name, idx) => ({
        order_index: idx,
        raw_name: name,
        is_alternative: false,
        master_data: null,
      }));
    } else if (Array.isArray(rawItemsText) && rawItemsText.length > 0) {
      // Yedek: düz string listesi
      items = rawItemsText.map((name, idx) => ({
        order_index: idx,
        raw_name: name,
        is_alternative: false,
        master_data: null,
      }));
    }
  }

  return { ...raw, items: items || [] };
}

function normalizeMenuList(payload) {
  if (Array.isArray(payload)) return payload.map(normalizeMenu);
  if (payload && Array.isArray(payload.menus)) return payload.menus.map(normalizeMenu);
  if (payload && Array.isArray(payload.results)) return payload.results.map(normalizeMenu);
  if (payload && Array.isArray(payload.data)) return payload.data.map(normalizeMenu);
  return payload;
}


export const menusApi = {
  // Kanonik yol /public/cities (kökteki duplicate route 308 ile yönlendirir)
  getCities: () => request('/public/cities'),
  detectCity: () => request('/public/cities/detect'),
  getTodayMenu: async (city, dietary_type = 'standard') => {
    const data = await request(`/menus${buildQuery({ city, date: 'today', dietary_type })}`);
    return Array.isArray(data) ? data.map(normalizeMenu) : normalizeMenu(data);
  },
  getMenusByDate: async (city, date, dietary_type = 'standard') => {
    const data = await request(`/menus${buildQuery({ city, date, dietary_type })}`);
    return normalizeMenuList(data);
  },
  getMonthlyMenus: async (city, year, month, dietary_type = 'standard') => {
    const data = await request(`/menus${buildQuery({ city, year, month, dietary_type })}`);
    return normalizeMenuList(data);
  },
  getArchiveYears: (city) => request(`/menus/archive/years${buildQuery({ city })}`),
  getMenu: async (menuId, dietary_type = 'standard') => {
    const data = await request(`/menus/${menuId}${buildQuery({ dietary_type })}`);
    return normalizeMenu(data);
  },
  getMenuDetail: async (menuId, dietary_type = 'standard') => {
    const data = await request(`/menus/${menuId}${buildQuery({ dietary_type })}`);
    return normalizeMenu(data);
  },
  voteMenu: (menuId, sentiment) => {
    return request(`/menus/${menuId}/vote`, {
      method: 'POST',
      body: JSON.stringify({ sentiment })
    });
  },
};
