<script>
    import { icon } from '../ui/icons.js';
    import { sanitizeText } from '../../utils/sanitize.js';

    import { normalizeItems, groupItems } from '../../utils/menu.js';

    let { menu } = $props();

    let isBreakfast = $derived(menu.meal_type === 'breakfast');
    let label = $derived(isBreakfast ? 'Kahvaltı' : 'Akşam yemeği');
    let ratingSum = $derived(menu.rating_sum || 0);
    let voteCount = $derived(menu.vote_count || 0);

    let voteClass = $derived(
        voteCount === 0 ? 'archive-row__votes--soluk' :
        ratingSum > 0 ? 'archive-row__votes--positive' :
        ratingSum < 0 ? 'archive-row__votes--negative' : 'archive-row__votes--soluk'
    );

    let voteIcon = $derived(
        voteCount === 0 ? 'votedNone' :
        ratingSum > 0 ? 'votedUpMore' :
        ratingSum < 0 ? 'votedDownMore' : 'votedNone'
    );

    let rawItems = $derived(normalizeItems(menu));
    let items = $derived(groupItems(rawItems));
    
    let dishesText = $derived(
        items.map(item => {
            const dishes = item.dishes && item.dishes.length > 0 ? item.dishes : [{ name: item.name }];
            return dishes.map(d => sanitizeText(d.name)).join(' / ');
        }).join(', ')
    );
</script>

<div class="archive-row" id="archive-row-{menu.id}">
    <div class="archive-row__meal-type">
    <span class="archive-row__meal-badge">{label}</span>
    </div>
    <div class="archive-row__stats">
    <div class="archive-row__votes {voteClass}" data-tooltip="{voteCount} oy kullanıldı">
        {@html icon(voteIcon, 14)}
        <span>{ratingSum}</span>
    </div>
    </div>
    <div class="archive-row__content">
    <span class="archive-row__items">{dishesText}</span>
    </div>
    <a href="/menu/{menu.id}" class="archive-row__btn" data-link>İncele</a>
</div>
