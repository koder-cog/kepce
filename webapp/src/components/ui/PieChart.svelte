<script>
  /**
   * PieChart.svelte — Zero-dependency SVG Donut/Pie Chart
   * Props:
   *  - data: Array of { category: string, count: number, percentage: number, color: string }
   *  - title: Optional center title
   *  - size: number (default 180)
   */
  let { data = [], title = "Toplam", size = 180 } = $props();

  let hoveredIdx = $state(null);

  const radius = 70;
  const circumference = 2 * Math.PI * radius;

  // Compute slice offsets
  let slices = $derived.by(() => {
    let cumulativeAngle = 0;
    return data.map((item, idx) => {
      const percentage = item.percentage || 0;
      const strokeDasharray = `${(percentage / 100) * circumference} ${circumference}`;
      const strokeDashoffset = -((cumulativeAngle / 100) * circumference);
      cumulativeAngle += percentage;

      return {
        ...item,
        idx,
        strokeDasharray,
        strokeDashoffset,
      };
    });
  });

  let totalCount = $derived(data.reduce((acc, curr) => acc + (curr.count || 0), 0));

  let activeItem = $derived(
    hoveredIdx !== null ? slices[hoveredIdx] : null
  );
</script>

<div class="pie-chart-wrapper" style="--pie-size: {size}px;">
  <div class="pie-chart-visual">
    <svg
      viewBox="0 0 200 200"
      class="pie-chart-svg"
      role="img"
      aria-label="Kategori dağılım pasta grafiği"
    >
      <!-- Background Circle track -->
      <circle
        cx="100"
        cy="100"
        r={radius}
        class="pie-chart-track"
        fill="transparent"
        stroke-width="26"
      />

      <!-- Dynamic Slices -->
      <g transform="rotate(-90 100 100)">
        {#each slices as slice}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <circle
            cx="100"
            cy="100"
            r={radius}
            fill="transparent"
            stroke={slice.color}
            stroke-width={hoveredIdx === slice.idx ? "30" : "24"}
            stroke-dasharray={slice.strokeDasharray}
            stroke-dashoffset={slice.strokeDashoffset}
            class="pie-chart-slice"
            class:is-active={hoveredIdx === slice.idx}
            onmouseenter={() => (hoveredIdx = slice.idx)}
            onmouseleave={() => (hoveredIdx = null)}
            aria-label="{slice.category}: %{slice.percentage}"
          />
        {/each}
      </g>
    </svg>

    <!-- Center Label -->
    <div class="pie-chart-center">
      {#if activeItem}
        <span class="pie-center-value">%{activeItem.percentage}</span>
        <span class="pie-center-label">{activeItem.category}</span>
      {:else}
        <span class="pie-center-value">{totalCount.toLocaleString("tr-TR")}</span>
        <span class="pie-center-label">{title}</span>
      {/if}
    </div>
  </div>

  <!-- Legend -->
  <div class="pie-chart-legend">
    {#each slices as slice}
      <button
        type="button"
        class="pie-legend-item"
        class:is-active={hoveredIdx === slice.idx}
        onmouseenter={() => (hoveredIdx = slice.idx)}
        onmouseleave={() => (hoveredIdx = null)}
      >
        <span class="pie-legend-dot" style="background-color: {slice.color};"></span>
        <span class="pie-legend-name">{slice.category}</span>
        <span class="pie-legend-pct">%{slice.percentage}</span>
      </button>
    {/each}
  </div>
</div>
