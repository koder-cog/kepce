<script>
  import { icon } from "@/components/ui/icons.js";

  let { card = null } = $props();
</script>

{#if card}
  <aside class="c-kepce-direct-card" aria-label="Kepçe Sonucu">
    <div class="c-kepce-direct-card__header">
      <div class="c-kepce-direct-card__badge-group">
        <span class="c-kepce-direct-card__brand">🥣 KEPÇE DOĞRUDAN SONUÇ</span>
        <span class="c-kepce-direct-card__badge">{card.badge}</span>
      </div>
      <span class="c-kepce-direct-card__source">kepce.org</span>
    </div>
    <div class="c-kepce-direct-card__body">
      <h2 class="c-kepce-direct-card__title">
        <a href={card.href}>{card.title}</a>
      </h2>
      <p class="c-kepce-direct-card__subtitle">
        {card.subtitle}
      </p>
      <p class="c-kepce-direct-card__desc">
        {card.description}
      </p>
      {#if card.type === "city_menu"}
        <div class="c-kepce-direct-card__features">
          <span class="c-kepce-direct-card__feature-chip"
            >🍲 4 Çeşit Tabldot Menü</span
          >
          <span class="c-kepce-direct-card__feature-chip"
            >⏰ Sabah & Akşam Saatleri</span
          >
          <span class="c-kepce-direct-card__feature-chip"
            >📊 Kalori & Fiyat Takibi</span
          >
        </div>
      {/if}
    </div>
    <div class="c-kepce-direct-card__footer">
      <a href={card.href} class="c-kepce-direct-card__btn">
        <span>{card.cta}</span>
        {@html icon("arrowRight", 14)}
      </a>
    </div>
  </aside>
{/if}
