<script>
  import { icon } from '$lib/components/ui/icons.js';
  import { sanitizeText } from '$lib/utils/sanitize.js';

  import { normalizeItems, groupItems } from '$lib/domain/menu/menu.js';

  let { menu } = $props();

  let isBreakfast = $derived(menu.meal_type === 'breakfast');
  let label = $derived(isBreakfast ? 'Kahvaltı' : 'Akşam yemeği');
  let ratingSum = $derived(menu.rating_sum || 0);
  let voteCount = $derived(menu.vote_count || 0);

  let voteMeta = $derived.by(() => {
    if (voteCount === 0) {
      return {
        css: 'archive-row__votes--soluk',
        icon: 'votedNone',
        text: 'Oylamaya tenezzül edilmedi'
      };
    }
    if (ratingSum > 0) {
      return {
        css: 'archive-row__votes--positive',
        icon: 'votedUpMore',
        text: 'Sevildi'
      };
    }
    if (ratingSum < 0) {
      return {
        css: 'archive-row__votes--negative',
        icon: 'votedDownMore',
        text: 'Nefret edildi'
      };
    }
    return {
      css: 'archive-row__votes--disclaimer',
      icon: 'votedNone',
      text: 'İnsanlar ikiye bölündü'
    };
  });

  let rawItems = $derived(normalizeItems(menu));
  let items = $derived(groupItems(rawItems));

  // Tekil menü detay bağlantısı: öğünün yemek, besin ve yorum detaylarına doğrudan
  // /menu/[id] üzerinden gidilir; id yoksa geriye dönük şehir gününe düşülür.
  let menuUrl = $derived.by(() => {
    if (menu?.id) return `/menu/${menu.id}`;
    const slug = menu?.city_slug;
    const day = menu?.date ?? menu?.serve_date;
    return slug && day ? `/${slug}?gun=${day}` : '/';
  });

  let dishesText = $derived(
    items
      .map((item) => {
        const dishes = item.dishes && item.dishes.length > 0 ? item.dishes : [{ name: item.name }];
        return dishes.map((d) => sanitizeText(d.name)).join(' / ');
      })
      .join(', ')
  );
</script>

<div class="archive-row" id="archive-row-{menu.id}">
  <div class="archive-row__meal-type">
    <span class="archive-row__meal-badge">{label}</span>
  </div>
  <div class="archive-row__stats">
    <div class="archive-row__votes {voteMeta.css}" data-tooltip={voteMeta.text}>
      {@html icon(voteMeta.icon, 14)}
      <span>{ratingSum}</span>
    </div>
  </div>
  <div class="archive-row__content">
    <span class="archive-row__items">{dishesText}</span>
  </div>
  <a href={menuUrl} class="archive-row__btn" data-link>İncele</a>
</div>
