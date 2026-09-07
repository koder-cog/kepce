<script>
  import { icon } from "@/components/ui/icons.js";
  import { formatFullTurkishDate } from "@/utils/turkish.js";
  import { normalizeItems } from "@/utils/menu.js";

  let { card = null } = $props();

  let formattedDate = $derived(
    card?.date ? formatFullTurkishDate(card.date, true) : null
  );
</script>

{#if card}
  <aside class="c-kepce-direct-card" aria-label="Kepçe Menü Sonucu">
    <div class="c-kepce-direct-card__top">
      <h2 class="c-kepce-direct-card__title">
        <a href={card.href}>{card.title}</a>
      </h2>
      {#if formattedDate}
        <time class="c-kepce-direct-card__date">{formattedDate}</time>
      {/if}
    </div>

    {#if card.type === "city_menu" && card.menus && card.menus.length > 0}
      <!-- Doğrudan Menü Tabldot / Yemek Listesi Görünümü -->
      <div class="c-kepce-direct-card__meals">
        {#each card.menus as meal}
          {@const dishes = normalizeItems(meal)}
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

            {#if dishes.length > 0}
              <ul class="c-kepce-direct-card__dishes">
                {#each dishes as item}
                  <li class="c-kepce-direct-card__dish-item">
                    <span class="c-kepce-direct-card__dish-bullet">•</span>
                    <span class="c-kepce-direct-card__dish-name">{item.name}</span>
                    {#if item.dishes?.[0]?.weight}
                      <span class="c-kepce-direct-card__dish-portion">({item.dishes[0].weight})</span>
                    {/if}
                  </li>
                {/each}
              </ul>
            {:else}
              <p class="c-kepce-direct-card__dish-empty">Menü detayı bulunmuyor</p>
            {/if}
          </div>
        {/each}
      </div>
    {:else}
      <p class="c-kepce-direct-card__desc">
        {card.type === "city_menu"
          ? "Bu tarih için kayıtlı yemekhane menüsü bulunmuyor."
          : (card.description || "")}
      </p>
    {/if}

    <div class="c-kepce-direct-card__footer">
      <a href={card.href} class="c-kepce-direct-card__more-link">
        <span>{card.cta || "Detayları Kepçe'de incele"}</span>
        {@html icon("arrowRight", 14)}
      </a>
    </div>
  </aside>
{/if}
