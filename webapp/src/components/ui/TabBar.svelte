<script>
  import { onMount, tick } from "svelte";
  import { dev, building } from "$app/environment";

  let {
    tabs = [], // Array of { id, label, href?, icon?, badge? }
    activeId = $bindable(), // The id of the currently active tab
    class: className = "",
    onChange = null,
  } = $props();

  const validateTabs = (tbs) => {
    for (const tab of tbs) {
      if (!tab.label || !tab.icon) {
        console.warn(
          `[Kepçe Uyarı] TabBar içinde "${tab.id}" sekmesi için ikon veya yazı eksik!`,
        );
      }
    }
  };

  if (building) {
    // svelte-ignore state_referenced_locally
    validateTabs(tabs);
  }

  $effect(() => {
    if (dev) {
      validateTabs(tabs);
    }
  });

  let containerNode = $state(null);
  let indicatorWidth = $state(0);
  let indicatorLeft = $state(0);
  let isReady = $state(false);

  function updateIndicator() {
    if (!containerNode || !activeId) return;
    requestAnimationFrame(() => {
      if (!containerNode) return;
      const activeBtn = containerNode.querySelector(
        `.c-tab[data-id="${activeId}"]`,
      );
      if (activeBtn) {
        indicatorWidth = activeBtn.offsetWidth;
        indicatorLeft = activeBtn.offsetLeft;
        activeBtn.scrollIntoView({
          behavior: "smooth",
          block: "nearest",
          inline: "nearest",
        });
      }
    });
  }

  $effect(() => {
    activeId; // Track activeId
    if (isReady) {
      updateIndicator();
    }
  });

  onMount(() => {
    updateIndicator();
    setTimeout(() => {
      isReady = true;
    }, 50);

    const ro = new ResizeObserver(() => {
      updateIndicator();
    });

    const tabsRow = containerNode?.querySelector(".c-tabs");
    if (tabsRow) ro.observe(tabsRow);
    return () => ro.disconnect();
  });
</script>

<div
  class="c-tabs-container {className}"
  bind:this={containerNode}
  class:is-ready={isReady}
>
  <div class="c-tabs">
    <div
      class="c-tabs__indicator"
      style="--indicator-width: {indicatorWidth}px; --indicator-transform: translateX({indicatorLeft}px);"
    ></div>

    {#each tabs as tab}
      {#if tab.href}
        <a
          href={tab.href}
          data-sveltekit-noscroll="true"
          data-id={tab.id}
          class="c-tab"
          class:c-tab--active={activeId === tab.id}
        >
          {#if tab.icon}
            <span class="c-tab__icon" class:c-tab__icon--responsive={tab.label}
              >{@html tab.icon}</span
            >
          {/if}
          {#if tab.label}
            <span class="c-tab__label" class:c-tab__label--responsive={tab.icon}
              >{tab.label}</span
            >
          {/if}
        </a>
      {:else}
        <button
          type="button"
          data-id={tab.id}
          class="c-tab"
          class:c-tab--active={activeId === tab.id}
          onclick={() => {
            activeId = tab.id;
            if (onChange) onChange(tab.id);
            if (tab.onClick) tab.onClick(tab.id);
          }}
        >
          {#if tab.icon}
            <span class="c-tab__icon" class:c-tab__icon--responsive={tab.label}
              >{@html tab.icon}</span
            >
          {/if}
          {#if tab.label}
            <span class="c-tab__label" class:c-tab__label--responsive={tab.icon}
              >{tab.label}</span
            >
          {/if}
          {#if tab.badge}
            <span class="notification-tab__badge">{tab.badge}</span>
          {/if}
        </button>
      {/if}
    {/each}
  </div>
</div>
