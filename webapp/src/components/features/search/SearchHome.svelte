<script>
  import { icon } from "@/components/ui/icons.js";

  let {
    isNavigating = false,
    searchInput = "",
    searchInputEl = $bindable(null),
    suggestions = [],
    isSuggestionsOpen = false,
    selectedSuggestionIndex = -1,
    randomShortcuts = [],
    basePath = "",
    onSearch = () => {},
    onInput = () => {},
    onKeydown = () => {},
    onBlur = () => {},
    onSelectSuggestion = () => {},
    onSelectBang = () => {},
    onOpenInfo = () => {},
    onOpenSettings = () => {},
  } = $props();
</script>

<div class="c-search-home" class:is-navigating={isNavigating}>
  <header class="c-search-home__header">
    <button
      type="button"
      class="c-search-icon-btn"
      onclick={onOpenInfo}
      aria-label="Hakkında"
      title="Hakkında"
    >
      {@html icon("info", 24)}
    </button>
    <button
      type="button"
      class="c-search-icon-btn"
      onclick={onOpenSettings}
      aria-label="Ayarlar"
      title="Ayarlar"
    >
      {@html icon("settings", 24)}
    </button>
  </header>

  <main class="c-search-home__center">
    <!-- Stabil Logo -->
    <div class="c-search-home__logo" aria-label="Kepçe">
      {@html icon("logoExperimental", null, "Kepçe")}
    </div>

    <form class="c-search-box" onsubmit={onSearch}>
      <input
        type="search"
        class="c-search-box__input"
        placeholder="İnterneti kurcala..."
        value={searchInput}
        oninput={onInput}
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

      {#if isSuggestionsOpen && suggestions.length > 0}
        <ul class="c-search-autocomplete" role="listbox">
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

    <!-- Hızlı Kısayol İpuçları (Dinamik Rastgele Havuz) -->
    <div class="c-search-home__shortcuts">
      {#each randomShortcuts as s}
        <button
          type="button"
          class="c-search-shortcut-pill"
          onclick={() => onSelectBang(s.prefix)}
        >
          <span class="pill-prefix">{s.prefix}</span>
          {s.label}
        </button>
      {/each}
    </div>
  </main>

  <!-- Minimalist Ana Sayfa Alt Şeridi -->
  <footer class="c-search-home-footer">
    <div class="c-search-home-footer__left">
      <a href={`${basePath}/ayarlar`} class="c-search-home-footer__link"
        >Ayarlar</a
      >
      <a href={`${basePath}/gizlilik`} class="c-search-home-footer__link"
        >Gizlilik</a
      >
      <a href={`${basePath}/kosullar`} class="c-search-home-footer__link"
        >Koşullar</a
      >
      <a href={`${basePath}/iletisim`} class="c-search-home-footer__link"
        >İletişim</a
      >
    </div>
    <div class="c-search-home-footer__right">
      <a
        href="https://github.com/searxng/searxng"
        target="_blank"
        rel="noopener noreferrer"
        class="c-search-home-footer__link">SearXNG</a
      >
      <a
        href="https://github.com/koder-cog/kepce"
        target="_blank"
        rel="noopener noreferrer"
        class="c-search-home-footer__link">Kaynak Kodu</a
      >
      <a
        href="https://www.gnu.org/licenses/agpl-3.0.html#license-text"
        target="_blank"
        rel="noopener noreferrer"
        class="c-search-home-footer__link c-search-home-footer__agpl">AGPLv3</a
      >
    </div>
  </footer>
</div>
