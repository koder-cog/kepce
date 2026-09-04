<script>
  import { icon } from "@/components/ui/icons.js";

  let {
    basePath = "",
    query = "",
    category = "general",
    searchInput = "",
    searchInputEl = $bindable(null),
    isLoading = false,
    searchHistory = [],
    isHistoryOpen = false,
    suggestions = [],
    isSuggestionsOpen = false,
    selectedSuggestionIndex = -1,
    instantPreview = null,
    onSearch = () => {},
    onInput = () => {},
    onFocus = () => {},
    onKeydown = () => {},
    onBlur = () => {},
    onSelectSuggestion = () => {},
    onClearHistory = () => {},
    onRemoveHistoryItem = () => {},
    onOpenInfo = () => {},
    onOpenSettings = () => {},
  } = $props();
</script>

<header class="c-search-results__topbar">
  <div class="c-search-results__topbar-inner">
    <a
      href={basePath || "/"}
      class="c-search-results__brand"
      aria-label="Kepçe Ara"
      title="Kepçe Ara"
    >
      {@html icon("logoSmallExperimental", 36, "Kepçe Logosu")}
    </a>

    <div class="c-search-results__search-wrap">
      <form class="c-search-box" onsubmit={onSearch}>
        <input
          type="search"
          class="c-search-box__input"
          placeholder="Ara..."
          value={searchInput}
          oninput={onInput}
          onfocus={onFocus}
          onkeydown={onKeydown}
          onblur={onBlur}
          bind:this={searchInputEl}
          aria-label="Ara"
          autocomplete="off"
        />
        <button
          type="submit"
          class="c-search-box__submit"
          aria-label="Ara"
          title="Ara"
        >
          {@html icon("search", 20)}
        </button>

        {#if isLoading}
          <div class="c-search-pulse-bar" aria-hidden="true"></div>
        {/if}

        <!-- Yerel Arama Geçmişi (Arama çubuğu boş ve odaklıyken) -->
        {#if isHistoryOpen && searchHistory.length > 0 && !searchInput}
          <ul
            class="c-search-autocomplete c-search-history-panel"
            role="listbox"
          >
            <li class="c-search-history-header">
              <span>Son Aramalar</span>
              <button
                type="button"
                class="c-search-history-clear"
                onmousedown={onClearHistory}
              >
                Temizle
              </button>
            </li>
            {#each searchHistory as item}
              <li
                class="c-search-autocomplete__item c-search-history__item"
                onmousedown={() => onSelectSuggestion(item)}
                role="option"
                tabindex="-1"
                aria-selected="false"
              >
                <span class="c-search-autocomplete__icon">
                  {@html icon("clock", 16)}
                </span>
                <span class="c-search-history__text">{item}</span>
                <button
                  type="button"
                  class="c-search-history__remove"
                  onmousedown={(e) => onRemoveHistoryItem(item, e)}
                  aria-label="Sil"
                  title="Sil"
                >
                  ✕
                </button>
              </li>
            {/each}
          </ul>
        {/if}

        {#if (isSuggestionsOpen && suggestions.length > 0) || instantPreview}
          <ul class="c-search-autocomplete" role="listbox">
            {#if instantPreview}
              <li
                class="c-search-autocomplete__item is-instant-preview"
                onmousedown={() => onSearch()}
                role="option"
                aria-selected="false"
              >
                <span class="c-search-autocomplete__preview-badge"
                  >{instantPreview.badge}</span
                >
                <strong class="c-search-autocomplete__preview-text"
                  >{instantPreview.text}</strong
                >
              </li>
            {/if}
            {#each suggestions as item, idx}
              {#if item.isBang}
                <li
                  class="c-search-autocomplete__item is-bang"
                  class:is-selected={idx === selectedSuggestionIndex}
                  onmousedown={() => onSelectSuggestion(item)}
                  role="option"
                  aria-selected={idx === selectedSuggestionIndex}
                >
                  <span class="c-search-autocomplete__bang-prefix"
                    >{item.prefix}</span
                  >
                  <span class="c-search-autocomplete__bang-label"
                    >{item.label}</span
                  >
                </li>
              {:else}
                <li
                  class="c-search-autocomplete__item"
                  class:is-selected={idx === selectedSuggestionIndex}
                  onmousedown={() => onSelectSuggestion(item)}
                  role="option"
                  aria-selected={idx === selectedSuggestionIndex}
                >
                  <span class="c-search-autocomplete__icon">
                    {@html icon("search", 16)}
                  </span>
                  <span>{item.displayText || item}</span>
                </li>
              {/if}
            {/each}
          </ul>
        {/if}
      </form>
    </div>

    <div class="c-search-results__topbar-actions">
      <a
        href={basePath
          ? `${basePath}/rss?q=${encodeURIComponent(query)}&kategori=${category || "general"}`
          : `/ara/rss?q=${encodeURIComponent(query)}&kategori=${category || "general"}`}
        target="_blank"
        rel="noopener noreferrer"
        class="c-search-icon-btn"
        aria-label="RSS Linki"
        title="RSS Linki"
      >
        {@html icon("rss", 20)}
      </a>
      <button
        type="button"
        class="c-search-icon-btn"
        onclick={onOpenInfo}
        aria-label="Hakkında"
        title="Hakkında"
      >
        {@html icon("info", 22)}
      </button>
      <button
        type="button"
        class="c-search-icon-btn"
        onclick={onOpenSettings}
        aria-label="Ayarlar"
        title="Ayarlar"
      >
        {@html icon("settings", 22)}
      </button>
    </div>
  </div>
</header>
