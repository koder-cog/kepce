<script>
    import { onMount } from "svelte";
    import { slide } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import { icon } from "./icons.js";

    let {
        id = "default-banner",
        text = "Duyuru metni",
        ctaText = "",
        ctaHref = "#",
        theme = "accent-primary", // or 'accent-disclaimer'
        onCtaClick = null,
    } = $props();

    let isVisible = $state(false);

    onMount(() => {
        const dismissed = localStorage.getItem(`banner-dismissed-${id}`);
        if (!dismissed) {
            isVisible = true;
        }
    });

    function dismiss() {
        isVisible = false;
        localStorage.setItem(`banner-dismissed-${id}`, "true");
    }
</script>

{#if isVisible}
    <div class="banner banner--{theme}" transition:slide|local={{ duration: 300, easing: cubicOut }}>
        <div class="banner__content">
            <span class="banner__text">{text}</span>
            <div class="banner__actions">
                {#if ctaText}
                    {#if onCtaClick}
                        <button class="banner__cta" onclick={onCtaClick}>{ctaText}</button>
                    {:else}
                        <a href={ctaHref} class="banner__cta" data-link>{ctaText}</a>
                    {/if}
                {/if}
                <button
                    class="banner__close"
                    onclick={dismiss}
                    aria-label="Kapat"
                >
                    {@html icon("close", 16)}
                </button>
            </div>
        </div>
    </div>
{/if}

