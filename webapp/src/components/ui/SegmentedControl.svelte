<script>
    import { onMount } from "svelte";
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
    let isReady = $state(false);

    onMount(() => {
        requestAnimationFrame(() => {
            isReady = true;
        });
    });

    function selectOption(val, e) {
        value = val;
        if (onChange) onChange(val, e);
    }
</script>

<div
    {id}
    class="c-segmented-control {className}"
    data-variant={variant}
    class:is-ready={isReady}
    style="--active-index: {activeIndex}; --total-options: {totalOptions};"
>
    <div class="c-segmented-control__indicator"></div>

    {#each options as opt}
        <button
            type="button"
            class="c-segmented-control__btn"
            class:c-segmented-control__btn--active={value === opt.value}
            data-value={opt.value}
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
