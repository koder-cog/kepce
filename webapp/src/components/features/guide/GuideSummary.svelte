<script>
  import '@/styles/pages/_guide.css';
  import { icon } from '@/components/ui/icons.js';

  /**
   * GuideSummary - Rehber sayfaları için "hap özet kutusu".
   *
   * Zaman çizelgesindeki bildirim banner'ı ile aynı görsel dili kullanır
   * (`status-card--main status-card--yavas status-card--notice`): yumuşak
   * uyarı tonlu arka plan, yuvarlak köşe, ikon + başlık. Google Featured
   * Snippet (sıfırıncı sıra) için sayfa tepesine yerleştirilir; `dl` yapısı
   * sayesinde arama motorları etiket-değer çiftlerini net çözer.
   *
   * @type {{
   *   title: string,
   *   rows?: Array<{ label: string, value: string, wide?: boolean }>,
   *   note?: string | null
   * }}
   */
  let { title, rows = [], note = null } = $props();
</script>

<aside
  class="status-card--main status-card--yavas status-card--notice guide-summary"
  aria-label={title}
>
  <div class="guide-summary__head">
    <span class="guide-summary__icon" aria-hidden="true">{@html icon('info', 20)}</span>
    <span class="guide-summary__title">{title}</span>
  </div>
  <dl class="guide-summary__grid">
    {#each rows as row}
      <div class="guide-summary__row {row.wide ? 'guide-summary__row--wide' : ''}">
        <dt class="guide-summary__label">{row.label}</dt>
        <dd class="guide-summary__value">{row.value}</dd>
      </div>
    {/each}
  </dl>
  {#if note}
    <p class="guide-summary__note">{note}</p>
  {/if}
</aside>
