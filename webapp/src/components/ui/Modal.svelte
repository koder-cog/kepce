<script>
    import { onMount, tick } from "svelte";
    import { icon } from "./icons.js";
    import { enhanceSelects } from "../../lib/dom/dropdown_enhancer.js";

    let { options = {}, onClose, controller, children, footer } = $props();

    let isOpen = $state(false);
    let isClosing = $state(false);
    let modalContainer = $state(null);
    // #66-67: Focus restoration - modal kapanınca odak, modalı açan
    // elemana geri verilir.
    let previouslyFocused = null;
    let pushedState = false;

    onMount(() => {
        previouslyFocused = document.activeElement;
        // Controller'a kendi DOM elementimizi ve close metodumuzu atıyoruz
        if (controller) {
            controller.close = close;
            controller.modalElement = modalContainer;
        }

        // Double requestAnimationFrame ensures the initial (closed) state is
        // painted before we flip isOpen, so the entrance transition plays
        // smoothly instead of the modal appearing abruptly.
        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                isOpen = true;
                document.body.style.overflow = "hidden";

                if (modalContainer) {
                    enhanceSelects(modalContainer);
                    const firstInput = modalContainer.querySelector(
                        'input:not([type="hidden"])',
                    );
                    const firstSelect = modalContainer.querySelector("select");
                    const firstTextarea =
                        modalContainer.querySelector("textarea");
                    const lastBtn = modalContainer.querySelector(
                        ".c-modal__footer .btn:last-child",
                    );
                    (
                        firstInput ||
                        firstSelect ||
                        firstTextarea ||
                        lastBtn
                    )?.focus();

                    history.pushState({ kepceModal: true }, "");
                    pushedState = true;
                }
            });
        });
    });

    export function close(fromPopState = false) {
        if (isClosing) return;
        isClosing = true;
        document.body.style.overflow = "";

        if (!fromPopState && pushedState && history.state?.kepceModal) {
            history.back();
        }
        pushedState = false;
        setTimeout(() => {
            if (onClose) onClose();
            // #66: Odağı tetikleyen elemana geri ver (DOM'da hâlâ varsa)
            if (previouslyFocused && document.contains(previouslyFocused)) {
                previouslyFocused.focus();
            }
        }, 350);
    }

    function handleKeydown(e) {
        if (e.key === "Escape" && !options.disableEscape) {
            close();
            return;
        }
        // #66: Focus trap - Tab ile modal dışına çıkılamaz
        if (e.key === "Tab" && modalContainer) {
            const focusables = modalContainer.querySelectorAll(
                'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
            );
            if (focusables.length === 0) return;
            const first = focusables[0];
            const last = focusables[focusables.length - 1];
            if (e.shiftKey && document.activeElement === first) {
                e.preventDefault();
                last.focus();
            } else if (!e.shiftKey && document.activeElement === last) {
                e.preventDefault();
                first.focus();
            }
        }
    }

    function handlePopState(e) {
        if (isOpen && !isClosing) {
            close(true);
        }
    }
</script>

<svelte:window onkeydown={handleKeydown} onpopstate={handlePopState} />

<div
    class="c-modal c-modal--{options.variant || 'standard'} {isOpen &&
    !isClosing
        ? 'c-modal--open'
        : ''}"
    role="dialog"
    aria-modal="true"
    bind:this={modalContainer}
>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
        class="c-modal__backdrop"
        onclick={() => !options.disableEscape && close()}
    ></div>

    <div class="c-modal__surface">
        <div class="c-modal__header">
            <h2 class="c-modal__title">{options.title}</h2>
            {#if options.iconHtml}
                <div
                    class="c-modal__icon {options.iconColor === 'danger' ||
                    options.iconColor === 'negative'
                        ? 'c-modal__icon--danger'
                        : ''}"
                >
                    {@html options.iconHtml}
                </div>
            {/if}
        </div>

        <div class="c-modal__content">
            {#if children}
                {@render children()}
            {:else if options.contentHtml}
                {@html options.contentHtml}
            {/if}
            {#if options.variant === "lightbox"}
                <button
                    class="c-lightbox-close"
                    aria-label="Kapat"
                    onclick={close}
                >
                    {@html icon("close", 20)}
                </button>
            {/if}
        </div>

        {#if footer}
            <div class="c-modal__footer">
                {@render footer()}
            </div>
        {:else if options.buttons && options.buttons.length > 0}
            <div class="c-modal__footer">
                {#each options.buttons as btnConfig}
                    <button
                        class="btn btn--{btnConfig.variant || 'secondary'}"
                        onclick={async (e) => {
                            e.preventDefault();
                            let preventClose = false;
                            if (btnConfig.onClick) {
                                const result =
                                    await btnConfig.onClick(modalContainer);
                                if (result === false) preventClose = true;
                            }
                            if (!preventClose) close();
                        }}
                    >
                        {btnConfig.label}
                    </button>
                {/each}
            </div>
        {/if}
    </div>
</div>
