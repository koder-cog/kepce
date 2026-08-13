<script module>
    let activeDropdownClose = null;
</script>

<script>
    import { icon } from '../ui/icons.js';
    import { animate, getDuration } from '../../lib/dom/motion.js';
    import { popover } from '../../lib/dom/popover.js';
    import { onMount, tick } from 'svelte';

    let {
        options = [],
        value = $bindable(),
        placeholder = 'Seçiniz',
        disabled = false,
        variant = 'primary', // primary | secondary | ghost
        actionItem = null,
        onActionClick = null,
        specialItem = null,
        onSpecialClick = null,
        onChange = null
    } = $props();

    // ── State ──────────────────────────────────────────────────
    let isOpen = $state(false);
    let triggerEl = $state(null);
    let menuEl = $state(null);
    let overlayEl = $state(null);
    let listEl = $state(null);
    let searchInputEl = $state(null);
    let highlightedIndex = $state(-1);
    let searchQuery = $state('');
    let isMobile = $state(false);

    let searchBuffer = '';
    let searchTimeout = null;
    let openTime = 0;
    let isProgrammaticScroll = false;
    let animation = null;
    let overlayAnimation = null;

    // ── Derived ────────────────────────────────────────────────
    let displayLabel = $derived(options.find(o => o.value === value)?.label || placeholder);
    let isLongList = $derived(options.length >= 10);
    let useModal = $derived(isMobile && isLongList);
    let filteredOptions = $derived(
        searchQuery
            ? options.filter(o => o.label.toLocaleLowerCase('tr-TR').includes(searchQuery.toLocaleLowerCase('tr-TR')))
            : options
    );

    // ── Global Scroll Lock ─────────────────────────────────────
    $effect(() => {
        if (isOpen && useModal) {
            document.documentElement.classList.add('dropdown-open');
        } else {
            document.documentElement.classList.remove('dropdown-open');
        }
    });

    // ── Lifecycle ──────────────────────────────────────────────
    function checkMobile() { isMobile = window.innerWidth <= 600; }

    onMount(() => {
        checkMobile();
        window.addEventListener('resize', checkMobile);

        // Nav menüsü açıldığında bu Dropdown'ı kapat
        const onNavMenuOpen = () => { if (isOpen) close(); };
        window.addEventListener('kepce:nav-menu-open', onNavMenuOpen);

        return () => {
            window.removeEventListener('resize', checkMobile);
            window.removeEventListener('kepce:nav-menu-open', onNavMenuOpen);
        };
    });

    // ── Portal ─────────────────────────────────────────────────
    function portal(node) {
        let parent = node.parentNode;
        let placeholder = document.createComment('portal');
        if (parent) parent.insertBefore(placeholder, node);
        document.body.appendChild(node);
        return {
            destroy() {
                if (placeholder.parentNode) {
                    placeholder.parentNode.insertBefore(node, placeholder);
                    placeholder.parentNode.removeChild(placeholder);
                } else if (node.parentNode) {
                    node.parentNode.removeChild(node);
                }
            }
        };
    }

    // ── Scroll helpers ─────────────────────────────────────────
    function scrollToHighlighted() {
        if (!listEl) return;
        const items = listEl.querySelectorAll('.c-menu__item:not(.c-menu__item--action):not(.c-menu__item--special)');
        const target = items[highlightedIndex];
        if (!target) return;
        isProgrammaticScroll = true;
        const listHeight = listEl.clientHeight;
        const itemTop = target.offsetTop;
        const itemHeight = target.offsetHeight;
        listEl.scrollTop = itemTop - (listHeight / 2) + (itemHeight / 2);
        setTimeout(() => { isProgrammaticScroll = false; }, 50);
    }

    // ── Close helpers ──────────────────────────────────────────
    function onOutsideClick(e) {
        if (!isOpen) return;
        if (triggerEl?.contains(e.target)) return;
        if (menuEl?.contains(e.target)) return;
        close();
    }

    function onScrollClose(e) {
        if (isProgrammaticScroll || !isOpen) return;
        if (Date.now() - openTime < 100) return;
        const path = e.composedPath?.() || [];
        if (path.some(el => el === listEl || el === menuEl || el === overlayEl)) return;
        close();
    }

    // ── Open / Close / Toggle ──────────────────────────────────
    async function open() {
        if (isOpen || disabled) return;
        if (activeDropdownClose && activeDropdownClose !== close) activeDropdownClose();
        activeDropdownClose = close;

        window.dispatchEvent(new CustomEvent('kepce:dropdown-open'));

        searchQuery = '';
        isOpen = true;
        openTime = Date.now();

        highlightedIndex = filteredOptions.findIndex(o => o.value === value);
        if (highlightedIndex === -1 && filteredOptions.length > 0) highlightedIndex = 0;

        await tick();
        scrollToHighlighted();

        if (isLongList && searchInputEl && !isMobile) searchInputEl.focus({ preventScroll: true });

        if (animation) animation.cancel();

        requestAnimationFrame(() => {
            if (!menuEl) return;

            if (useModal) {
                
                animation = animate(menuEl, [
                    { opacity: 0, transform: 'translate(-50%, -50%) scale(0.95)' },
                    { opacity: 1, transform: 'translate(-50%, -50%) scale(1)' }
                ], { duration: getDuration(450), easing: 'cubic-bezier(0.34, 1.56, 0.64, 1)' });

                if (overlayAnimation) overlayAnimation.cancel();
                if (overlayEl) {
                    overlayAnimation = animate(overlayEl,
                        [{ opacity: 0 }, { opacity: 1 }],
                        { duration: getDuration(350), easing: 'ease-out' }
                    );
                }
            } else {
                const isUp = menuEl.dataset.openingDirection === 'up';
                animation = animate(menuEl, [
                    { opacity: 0, transform: `scale(0.95) translateY(${isUp ? '8px' : '-8px'})` },
                    { opacity: 1, transform: 'scale(1) translateY(0)' }
                ], { duration: getDuration(450), easing: 'cubic-bezier(0.34, 1.56, 0.64, 1)' });
            }
        });
    }

    function close() {
        if (!isOpen) return;
        if (activeDropdownClose === close) activeDropdownClose = null;
        if (animation) animation.cancel();

        if (useModal && menuEl) {
            animation = animate(menuEl, [
                { opacity: 1, transform: 'translate(-50%, -50%) scale(1)' },
                { opacity: 0, transform: 'translate(-50%, -50%) scale(0.95)' }
            ], { duration: getDuration(220), easing: 'ease-in' });

            if (overlayAnimation && overlayEl) {
                overlayAnimation.cancel();
                overlayAnimation = animate(overlayEl,
                    [{ opacity: 1 }, { opacity: 0 }],
                    { duration: getDuration(200), easing: 'ease-in' }
                );
            }
            
        } else if (menuEl) {
            const isUp = menuEl.dataset.openingDirection === 'up';
            animation = animate(menuEl, [
                { opacity: 1, transform: 'scale(1) translateY(0)' },
                { opacity: 0, transform: `scale(0.96) translateY(${isUp ? '4px' : '-4px'})` }
            ], { duration: getDuration(180), easing: 'ease-in' });
        }

        if (animation) {
            animation.onfinish = () => { isOpen = false; };
        } else {
            isOpen = false;
        }
    }

    function toggle(e) {
        if (e) e.stopPropagation();
        isOpen ? close() : open();
    }

    // ── Selection ──────────────────────────────────────────────
    function selectOption(e, opt) {
        e.stopPropagation();
        if (opt.disabled) return;
        value = opt.value;
        if (onChange) onChange(value);
        close();
    }

    function handleActionClick(e) {
        e.stopPropagation();
        close();
        if (onActionClick) onActionClick();
    }

    function handleSpecialClick(e) {
        e.stopPropagation();
        close();
        if (onSpecialClick) onSpecialClick();
    }

    // ── Keyboard ───────────────────────────────────────────────
    function handleKeyDown(e) {
        if (disabled) return;
        const key = e.key;
        const maxIndex = filteredOptions.length - 1;

        if (!isOpen) {
            if (key === 'ArrowDown' || key === 'ArrowUp') {
                e.preventDefault();
                const diff = key === 'ArrowDown' ? 1 : -1;
                let nextIndex = filteredOptions.findIndex(o => o.value === value) + diff;
                while (nextIndex >= 0 && nextIndex <= maxIndex && filteredOptions[nextIndex].disabled) nextIndex += diff;
                if (nextIndex >= 0 && nextIndex <= maxIndex) {
                    value = filteredOptions[nextIndex].value;
                    if (onChange) onChange(value);
                }
                return;
            }
            if (key === 'Enter' || key === ' ' || (key === 'ArrowDown' && e.altKey)) {
                e.preventDefault();
                open();
                return;
            }
            return;
        }

        // ── Menü açıkken ──
        if (key === 'Escape') { e.preventDefault(); close(); return; }

        // Arama kutusundayken tuşlara izin ver, sadece ok/enter yakala
        if (document.activeElement === searchInputEl) {
            if (key === 'ArrowDown' || key === 'ArrowUp' || key === 'Enter') {
                e.preventDefault();
            } else {
                setTimeout(() => { highlightedIndex = 0; scrollToHighlighted(); }, 0);
                return;
            }
        }

        if (key === 'Tab') { close(); return; }

        if (key === 'Enter' || (key === ' ' && document.activeElement !== searchInputEl)) {
            e.preventDefault();
            const highlighted = filteredOptions[highlightedIndex];
            if (highlighted && !highlighted.disabled) {
                value = highlighted.value;
                if (onChange) onChange(value);
                close();
            }
            return;
        }

        if (key === 'ArrowDown' || key === 'ArrowUp') {
            e.preventDefault();
            const diff = key === 'ArrowDown' ? 1 : -1;
            let nextIndex = highlightedIndex + diff;
            while (nextIndex >= 0 && nextIndex <= maxIndex && filteredOptions[nextIndex].disabled) nextIndex += diff;
            if (nextIndex >= 0 && nextIndex <= maxIndex) {
                highlightedIndex = nextIndex;
                scrollToHighlighted();
            }
            return;
        }

        if (key === 'Home') { e.preventDefault(); highlightedIndex = filteredOptions.findIndex(o => !o.disabled); scrollToHighlighted(); return; }
        if (key === 'End') {
            e.preventDefault();
            for (let i = maxIndex; i >= 0; i--) { if (!filteredOptions[i].disabled) { highlightedIndex = i; break; } }
            scrollToHighlighted();
            return;
        }

        // Type-ahead (arama kutusu yokken, kısa listelerde)
        if (document.activeElement !== searchInputEl && key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
            e.preventDefault();
            clearTimeout(searchTimeout);
            searchBuffer += key.toLowerCase();
            searchTimeout = setTimeout(() => { searchBuffer = ''; }, 500);
            const isRepeated = searchBuffer.length > 1 && searchBuffer.split('').every(c => c === searchBuffer[0]);
            let matchIndex = -1;
            if (isRepeated) {
                const char = searchBuffer[0];
                for (let i = 1; i <= filteredOptions.length; i++) {
                    const idx = (highlightedIndex + i) % filteredOptions.length;
                    if (!filteredOptions[idx].disabled && filteredOptions[idx].label.toLowerCase().startsWith(char)) { matchIndex = idx; break; }
                }
            } else {
                matchIndex = filteredOptions.findIndex(o => !o.disabled && o.label.toLowerCase().startsWith(searchBuffer));
            }
            if (matchIndex !== -1) { highlightedIndex = matchIndex; scrollToHighlighted(); }
        }
    }
</script>

<svelte:window
    onscroll={onScrollClose}
    onclick={onOutsideClick}
/>

<!-- ─── Trigger ─────────────────────────────────────────────── -->
<div
    class="dropdown dropdown--{variant}"
    class:dropdown--open={isOpen}
    class:dropdown--disabled={disabled}
>
    <button
        bind:this={triggerEl}
        class="dropdown__trigger dropdown__trigger--{variant}"
        class:dropdown__trigger--open={isOpen}
        class:dropdown__trigger--disabled={disabled}
        type="button"
        {disabled}
        aria-haspopup="listbox"
        aria-expanded={isOpen}
        onclick={toggle}
        onkeydown={handleKeyDown}
    >
        <span class="dropdown__label">{displayLabel}</span>
        <div class="dropdown__chevron">{@html icon('chevronDown')}</div>
    </button>
</div>

<!-- ─── Menu Panel ──────────────────────────────────────────── -->
{#if isOpen}
    {#if useModal}
        <div class="u-hidden">
            <div
                bind:this={overlayEl}
                class="c-menu__overlay c-menu__overlay--open"
                use:portal
                onclick={(e) => { e.stopPropagation(); close(); }}
                role="presentation"
            ></div>
        </div>
    {/if}

    <div class="u-hidden">
        <div
            bind:this={menuEl}
            class="c-menu c-menu--open"
            class:c-menu--modal={useModal}
            role="listbox"
            use:portal
            use:popover={{ triggerEl, align: 'left' }}
        >
        {#if isLongList}
            <div class="c-menu__search">
                <input
                    bind:this={searchInputEl}
                    type="text"
                    placeholder="Ara..."
                    bind:value={searchQuery}
                    onkeydown={handleKeyDown}
                />
            </div>
        {/if}

        <div bind:this={listEl} class="c-menu__scroll-area">
            {#if actionItem}
                <button
                    class="c-menu__item c-menu__item--accent c-menu__item--action"
                    type="button"
                    role="option"
                    aria-selected="false"
                    onclick={handleActionClick}
                >
                    <span class="c-menu__item-label">{actionItem.label}</span>
                    {#if actionItem.icon}
                        <span class="c-menu__item-icon">{@html icon(actionItem.icon, 16)}</span>
                    {/if}
                </button>
            {/if}

            {#each filteredOptions as o, index}
                <button
                    class="c-menu__item"
                    class:c-menu__item--selected={o.value === value}
                    class:c-menu__item--disabled={o.disabled}
                    class:c-menu__item--highlighted={index === highlightedIndex}
                    type="button"
                    role="option"
                    aria-selected={o.value === value}
                    disabled={o.disabled}
                    onclick={(e) => selectOption(e, o)}
                >
                    <span class="c-menu__item-label">{o.label}</span>
                    {#if o.value === value}
                        <span class="c-menu__item-check">{@html icon('check', 16)}</span>
                    {/if}
                </button>
            {/each}

            {#if filteredOptions.length === 0}
                <div class="c-menu__item c-menu__item--disabled c-menu__empty-state">
                    Sonuç bulunamadı
                </div>
            {/if}
        </div>

        {#if specialItem}
            <button
                class="c-menu__item c-menu__item--special"
                type="button"
                role="option"
                aria-selected="false"
                onclick={handleSpecialClick}
            >
                <span class="c-menu__item-label">{specialItem.label}</span>
            </button>
        {/if}
    </div>
    </div>
{/if}
