<script>
    import { onMount } from "svelte";
    import { slide } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import { icon } from "./icons.js";

    let isOffline = $state(false);
    let justConnected = $state(false);
    let autoCloseTimer = null;

    onMount(() => {
        if (typeof window !== "undefined" && typeof navigator !== "undefined") {
            isOffline = !navigator.onLine;

            const handleOffline = () => {
                justConnected = false;
                if (autoCloseTimer) clearTimeout(autoCloseTimer);
                isOffline = true;
            };

            const handleOnline = () => {
                if (isOffline) {
                    isOffline = false;
                    justConnected = true;
                    if (autoCloseTimer) clearTimeout(autoCloseTimer);
                    // Kısaltılmış ekran süresi (1500 ms)
                    autoCloseTimer = setTimeout(() => {
                        justConnected = false;
                    }, 1500);
                }
            };

            window.addEventListener("offline", handleOffline);
            window.addEventListener("online", handleOnline);

            return () => {
                window.removeEventListener("offline", handleOffline);
                window.removeEventListener("online", handleOnline);
                if (autoCloseTimer) clearTimeout(autoCloseTimer);
            };
        }
    });
</script>

{#if isOffline}
    <div 
        class="banner banner--offline" 
        role="status" 
        aria-live="polite"
        transition:slide|local={{ duration: 200, easing: cubicOut }}
    >
        <div class="banner__content-simple">
            <span class="banner__icon">{@html icon("wifiOff", 16)}</span>
            <span class="banner__text-simple">Çevrimdışı Mod</span>
        </div>
    </div>
{:else if justConnected}
    <div 
        class="banner banner--online" 
        role="status" 
        aria-live="polite"
        transition:slide|local={{ duration: 200, easing: cubicOut }}
    >
        <div class="banner__content-simple">
            <span class="banner__text-simple">İnternete bağlandı</span>
        </div>
    </div>
{/if}
