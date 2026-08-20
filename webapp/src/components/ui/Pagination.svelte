<script>
  import { icon } from "./icons.js";

  let {
    page = 1,
    totalPages = 1,
    totalItems = null,
    compact = false,
    onPageChange = () => {},
  } = $props();

  let currentPage = $derived(Math.max(1, Math.min(page, totalPages)));

  // Sliding window pagination array
  let pageNumbers = $derived.by(() => {
    if (totalPages <= 7) {
      return Array.from({ length: totalPages }, (_, i) => i + 1);
    }

    if (currentPage <= 4) {
      return [1, 2, 3, 4, 5, "...", totalPages];
    }

    if (currentPage >= totalPages - 3) {
      return [
        1,
        "...",
        totalPages - 4,
        totalPages - 3,
        totalPages - 2,
        totalPages - 1,
        totalPages,
      ];
    }

    return [
      1,
      "...",
      currentPage - 1,
      currentPage,
      currentPage + 1,
      "...",
      totalPages,
    ];
  });

  function goTo(target) {
    if (target < 1 || target > totalPages || target === currentPage) return;
    onPageChange(target);
  }
</script>

{#if totalPages > 1 || (compact && totalItems !== null)}
  {#if compact}
    <div class="pagination pagination--compact" role="navigation" aria-label="Kompakt sayfalama">
      {#if totalItems !== null}
        <span class="pagination__info">Toplam {totalItems}</span>
      {/if}
      <button
        class="pagination__btn pagination__btn--compact"
        disabled={currentPage <= 1}
        onclick={() => goTo(currentPage - 1)}
        title="Önceki Sayfa"
        aria-label="Önceki Sayfa"
      >
        {@html icon("chevronLeft", 16)}
      </button>
      <span class="pagination__info u-mx-xs">{currentPage} / {totalPages}</span>
      <button
        class="pagination__btn pagination__btn--compact"
        disabled={currentPage >= totalPages}
        onclick={() => goTo(currentPage + 1)}
        title="Sonraki Sayfa"
        aria-label="Sonraki Sayfa"
      >
        {@html icon("chevronRight", 16)}
      </button>
    </div>
  {:else}
    <nav class="pagination" aria-label="Sayfalama">
      <button
        class="pagination__btn pagination__btn--nav"
        disabled={currentPage <= 1}
        onclick={() => goTo(currentPage - 1)}
        aria-label="Önceki sayfa"
      >
        {@html icon("chevronLeft", 18)}
        <span class="pagination__btn-text">Önceki</span>
      </button>

      <span class="pagination__mobile-label">
        Sayfa {currentPage} / {totalPages}
      </span>

      <ul class="pagination__list">
        {#each pageNumbers as item, idx}
          {#if item === "..."}
            <li class="pagination__item">
              <span class="pagination__ellipsis">…</span>
            </li>
          {:else}
            <li class="pagination__item">
              <button
                class="pagination__btn {item === currentPage ? 'is-active' : ''}"
                aria-current={item === currentPage ? "page" : undefined}
                aria-label={`Sayfa ${item}`}
                onclick={() => goTo(item)}
              >
                {item}
              </button>
            </li>
          {/if}
        {/each}
      </ul>

      <button
        class="pagination__btn pagination__btn--nav"
        disabled={currentPage >= totalPages}
        onclick={() => goTo(currentPage + 1)}
        aria-label="Sonraki sayfa"
      >
        <span class="pagination__btn-text">Sonraki</span>
        {@html icon("chevronRight", 18)}
      </button>
    </nav>
  {/if}
{/if}
