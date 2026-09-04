<script>
  import { icon } from "@/components/ui/icons.js";
  import { formatFullTurkishDate } from "@/utils/turkish.js";

  let { card = null } = $props();

  let formattedDate = $derived(
    card?.date ? formatFullTurkishDate(card.date, true) : null
  );
</script>

{#if card}
  <aside class="c-kepce-direct-card" aria-label="Kepçe Sonucu">
    <div class="c-kepce-direct-card__header">
      <div class="c-kepce-direct-card__badge-group">
        <span class="c-kepce-direct-card__brand">🥣 KEPÇE DOĞRUDAN SONUÇ</span>
        <span class="c-kepce-direct-card__badge">{card.badge}</span>
        {#if formattedDate}
          <span class="c-kepce-direct-card__date-badge">{formattedDate}</span>
        {/if}
      </div>
      <span class="c-kepce-direct-card__source">kepce.org</span>
    </div>

    <div class="c-kepce-direct-card__body">
      <h2 class="c-kepce-direct-card__title">
        <a href={card.href}>{card.title}</a>
      </h2>
      {#if card.subtitle}
        <p class="c-kepce-direct-card__subtitle">
          {card.subtitle}
        </p>
      {/if}

      {#if card.type === "city_menu" && card.menus && card.menus.length > 0}
        <!-- Doğrudan Menü Tabldot / Yemek Listesi Görünümü -->
        <div class="c-kepce-direct-card__meals">
          {#each card.menus as meal}
            <div class="c-kepce-direct-card__meal-box">
              <div class="c-kepce-direct-card__meal-header">
                <span class="c-kepce-direct-card__meal-type">
                  {meal.meal_type === "breakfast" ? "Kahvaltı" : "Akşam Yemeği"}
                </span>
                {#if meal.calorie_range_min && meal.calorie_range_max}
                  <span class="c-kepce-direct-card__meal-cal">
                    {meal.calorie_range_min === meal.calorie_range_max
                      ? `~${meal.calorie_range_min} kcal`
                      : `${meal.calorie_range_min} - ${meal.calorie_range_max} kcal`}
                  </span>
                {:else if meal.total_calories || meal.calculated_calories}
                  <span class="c-kepce-direct-card__meal-cal">
                    ~{meal.total_calories || meal.calculated_calories} kcal
                  </span>
                {/if}
              </div>

              {#if meal.items && meal.items.length > 0}
                <ul class="c-kepce-direct-card__dishes">
                  {#each meal.items as item}
                    <li class="c-kepce-direct-card__dish-item">
                      <span class="c-kepce-direct-card__dish-bullet">🍲</span>
                      <span class="c-kepce-direct-card__dish-name">{item.name}</span>
                      {#if item.portion_amount}
                        <span class="c-kepce-direct-card__dish-portion">({item.portion_amount})</span>
                      {/if}
                    </li>
                  {/each}
                </ul>
              {/if}
            </div>
          {/each}
        </div>
      {:else}
        <p class="c-kepce-direct-card__desc">
          {card.description}
        </p>
        {#if card.type === "city_menu"}
          <div class="c-kepce-direct-card__features">
            <span class="c-kepce-direct-card__feature-chip">🍲 4 Çeşit Tabldot Menü</span>
            <span class="c-kepce-direct-card__feature-chip">⏰ Sabah & Akşam Saatleri</span>
            <span class="c-kepce-direct-card__feature-chip">📊 Kalori & Fiyat Takibi</span>
          </div>
        {/if}
      {/if}
    </div>

    <!-- Wikipedia Tarzı "Detayları Kepçe'de incele" Linki -->
    <div class="c-kepce-direct-card__footer">
      <a href={card.href} class="c-kepce-direct-card__more-link">
        <span>{card.cta || "Detayları Kepçe'de incele"}</span>
        {@html icon("arrowRight", 14)}
      </a>
    </div>
  </aside>
{/if}
