<script>
    import { onMount, tick } from "svelte";
    import { dev, building } from "$app/environment";

    let {
        value = $bindable(),
        options = [],
        class: className = "",
        id = null,
        variant = "text",
        onChange,
        onHover,
    } = $props();

    const validateOptions = (opts, compId) => {
        for (const opt of opts) {
            const hasLabel = Boolean(opt.label || opt.tooltip);
            const hasIcon = Boolean(opt.icon);
            if (!hasLabel && !hasIcon) {
                console.warn(
                    `[Kepçe Uyarı] SegmentedControl (id: ${compId || "bilinmeyen"}) içinde "${opt.value}" değeri için hem ikon hem yazı eksik.`,
                );
            } else if (variant === "responsive" && (!hasLabel || !hasIcon)) {
                console.warn(
                    `[Kepçe Uyarı] SegmentedControl (id: ${compId || "bilinmeyen"}) responsive modda "${opt.value}" için ikon veya yazı eksik.`,
                );
            }
        }
    };

    if (building) {
        // svelte-ignore state_referenced_locally
        validateOptions(options, id);
    }

    $effect(() => {
        if (dev) {
            validateOptions(options, id);
        }
    });

    let activeIndex = $derived.by(() => {
        const idx = options.findIndex((opt) => opt.value === value);
        return idx >= 0 ? idx : 0;
    });
    let totalOptions = $derived(Math.max(1, options.length));

    let containerEl = $state(null);
    let indicatorLeft = $state(null);
    let indicatorWidth = $state(null);
    let isReady = $state(false);

    function updateIndicator() {
        if (!containerEl) return;
        const activeBtn =
            containerEl.querySelector(`[data-value="${value}"]`) ||
            containerEl.querySelectorAll(".c-segmented-control__btn")[activeIndex];
        if (activeBtn) {
            indicatorLeft = activeBtn.offsetLeft;
            indicatorWidth = activeBtn.offsetWidth;
        }
    }

    $effect(() => {
        // Değer veya seçenekler değiştiğinde indikatör pozisyonunu güncelle
        if (value !== undefined && options) {
            tick().then(() => {
                updateIndicator();
                if (!isReady && typeof window !== "undefined") {
                    requestAnimationFrame(() => {
                        isReady = true;
                    });
                }
            });
        }
    });

    onMount(() => {
        updateIndicator();
        let ro;
        if (typeof ResizeObserver !== "undefined" && containerEl) {
            ro = new ResizeObserver(() => {
                updateIndicator();
            });
            ro.observe(containerEl);
        }
        return () => {
            ro?.disconnect();
        };
    });

    function selectOption(val, e) {
        value = val;
        if (onChange) onChange(val, e);
    }

    function handleKeyDown(e) {
        if (!options.length) return;
        let newIndex = activeIndex;

        if (e.key === "ArrowRight" || e.key === "ArrowDown") {
            e.preventDefault();
            newIndex = (activeIndex + 1) % options.length;
        } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
            e.preventDefault();
            newIndex = (activeIndex - 1 + options.length) % options.length;
        } else if (e.key === "Home") {
            e.preventDefault();
            newIndex = 0;
        } else if (e.key === "End") {
            e.preventDefault();
            newIndex = options.length - 1;
        } else {
            return;
        }

        const nextOpt = options[newIndex];
        if (nextOpt) {
            selectOption(nextOpt.value, e);
            if (containerEl) {
                const btns = containerEl.querySelectorAll(".c-segmented-control__btn");
                btns[newIndex]?.focus();
            }
        }
    }

    let cssCustomProperties = $derived.by(() => {
        let styleStr = `--active-index: ${activeIndex}; --total-options: ${totalOptions};`;
        if (indicatorLeft !== null) {
            styleStr += ` --indicator-left: ${indicatorLeft}px;`;
        }
        if (indicatorWidth !== null) {
            styleStr += ` --indicator-width: ${indicatorWidth}px;`;
        }
        return styleStr;
    });
</script>

<div
    {id}
    bind:this={containerEl}
    class="c-segmented-control {className}"
    data-variant={variant}
    class:is-ready={isReady}
    role="radiogroup"
    aria-label="Seçenekler"
    tabindex="0"
    onkeydown={handleKeyDown}
    style={cssCustomProperties}
>
    <div class="c-segmented-control__indicator"></div>

    {#each options as opt}
        <button
            type="button"
            class="c-segmented-control__btn"
            class:c-segmented-control__btn--active={value === opt.value}
            data-value={opt.value}
            role="radio"
            aria-checked={value === opt.value}
            tabindex={value === opt.value ? 0 : -1}
            onclick={(e) => selectOption(opt.value, e)}
            onmouseenter={(e) => onHover?.(opt.value, e)}
            aria-label={opt.label || opt.tooltip || opt.value}
            title={variant === "icons" || variant === "responsive"
                ? opt.label || opt.tooltip
                : null}
        >
            {#if variant === "icons"}
                {#if opt.icon}
                    {@html opt.icon}
                {:else}
                    <span class="c-segmented-control__fallback-text"
                        >{opt.label}</span
                    >
                {/if}
            {:else if variant === "responsive"}
                {#if opt.icon}
                    <span class="c-segmented-control__icon"
                        >{@html opt.icon}</span
                    >
                {/if}
                {#if opt.label}
                    <span class="c-segmented-control__label">{opt.label}</span>
                {/if}
            {:else}
                <!-- Default (text) variant -->
                {#if opt.label}
                    <span>{opt.label}</span>
                {:else if opt.icon}
                    {@html opt.icon}
                {/if}
            {/if}
        </button>
    {/each}
</div>
