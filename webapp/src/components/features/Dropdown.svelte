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
        groups = [],
        value = $bindable(),
        placeholder = 'Seçiniz',
        ariaLabel = undefined,
        id = null,
        disabled = false,
        variant = 'primary', // primary | secondary | ghost
        actionItem = null,
        onActionClick = null,
        specialItem = null,
        onSpecialClick = null,
        onChange = null,
        forceModal = false
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
    let allOptions = $derived(
        groups.length > 0
            ? groups.flatMap(g => (g.options || g.links || []).map(item => ({
                ...item,
                value: item.value ?? item.href,
                label: item.label,
                groupTitle: g.title,
                disabled: item.disabled
            })))
            : options
    );

    let displayLabel = $derived((() => {
        if (value !== undefined && value !== null) {
            const found = allOptions.find(o => {
                if (o.value === value) return true;
                if (o.isActive && typeof o.isActive === 'function') return o.isActive(value);
                return false;
            });
            if (found) return found.label;
        }
        return placeholder;
    })());

    let isLongList = $derived(allOptions.length >= 10);
    let useModal = $derived(isMobile && (isLongList || forceModal || groups.length > 0));

    let filteredOptions = $derived(
        searchQuery
            ? allOptions.filter(o => o.label.toLocaleLowerCase('tr-TR').includes(searchQuery.toLocaleLowerCase('tr-TR')))
            : allOptions
    );

    let filteredGroups = $derived(
        groups.map(g => {
            const items = (g.options || g.links || []).filter(item => {
                if (!searchQuery) return true;
                return item.label.toLocaleLowerCase('tr-TR').includes(searchQuery.toLocaleLowerCase('tr-TR'));
            });
            return { ...g, items };
        }).filter(g => g.items.length > 0)
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

        highlightedIndex = filteredOptions.findIndex(o => o.value === value || (o.isActive && typeof o.isActive === 'function' && o.isActive(value)));
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
        const optVal = opt.value ?? opt.href;
        value = optVal;
        if (onChange) onChange(optVal, opt);
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
                    if (onChange) onChange(value, filteredOptions[nextIndex]);
                }
            } else if (key === 'Enter' || key === ' ') {
                e.preventDefault();
                open();
            }
            return;
        }

        if (key === 'Escape') {
            e.preventDefault();
            close();
            triggerEl?.focus();
            return;
        }

        if (key === 'Tab') {
            close();
            return;
        }

        if (key === 'ArrowDown') {
            e.preventDefault();
            let next = highlightedIndex + 1;
            while (next <= maxIndex && filteredOptions[next].disabled) next++;
            if (next <= maxIndex) {
                highlightedIndex = next;
                scrollToHighlighted();
            }
            return;
        }

        if (key === 'ArrowUp') {
            e.preventDefault();
            let prev = highlightedIndex - 1;
            while (prev >= 0 && filteredOptions[prev].disabled) prev--;
            if (prev >= 0) {
                highlightedIndex = prev;
                scrollToHighlighted();
            }
            return;
        }

        if (key === 'Home') {
            e.preventDefault();
            highlightedIndex = filteredOptions.findIndex(o => !o.disabled);
            scrollToHighlighted();
            return;
        }

        if (key === 'End') {
            e.preventDefault();
            for (let i = maxIndex; i >= 0; i--) {
                if (!filteredOptions[i].disabled) {
                    highlightedIndex = i;
                    break;
                }
            }
            scrollToHighlighted();
            return;
        }

        if (key === 'Enter') {
            e.preventDefault();
            if (highlightedIndex >= 0 && highlightedIndex <= maxIndex) {
                const opt = filteredOptions[highlightedIndex];
                if (opt && !opt.disabled) selectOption(e, opt);
            }
            return;
        }

        // Type-ahead
        if (key.length === 1 && !e.ctrlKey && !e.altKey && !e.metaKey && document.activeElement !== searchInputEl) {
            clearTimeout(searchTimeout);
            searchBuffer += key.toLocaleLowerCase('tr-TR');
            searchTimeout = setTimeout(() => { searchBuffer = ''; }, 500);

            const matchIndex = filteredOptions.findIndex(o =>
                !o.disabled && o.label.toLocaleLowerCase('tr-TR').startsWith(searchBuffer)
            );
            if (matchIndex !== -1) {
                highlightedIndex = matchIndex;
                scrollToHighlighted();
            }
        }
    }

    let triggerAriaLabel = $derived(
        ariaLabel || (displayLabel ? `${placeholder}: ${displayLabel}` : `${placeholder} seçiniz`)
    );
    let menuId = 'c-menu-' + Math.random().toString(36).slice(2, 8);
</script>

<svelte:window onclick={onOutsideClick} onscrollcapture={onScrollClose} />

<div
    {id}
    class="dropdown dropdown--{variant}"
    class:dropdown--open={isOpen}
    class:dropdown--disabled={disabled}
>
    <button
        bind:this={triggerEl}
        class="dropdown__trigger dropdown__trigger--{variant}"
        class:dropdown__trigger--open={isOpen}
        class:dropdown__trigger--disabled={disabled}
        class:dropdown__trigger--has-value={value !== undefined && value !== null && value !== ''}
        class:dropdown__trigger--placeholder={value === undefined || value === null || value === ''}
        type="button"
        role="combobox"
        aria-haspopup="listbox"
        aria-expanded={isOpen}
        aria-controls={isOpen ? menuId : undefined}
        aria-label={triggerAriaLabel}
        onclick={toggle}
        onkeydown={handleKeyDown}
    >
        <span class="dropdown__label">{displayLabel}</span>
        <div class="dropdown__chevron" aria-hidden="true">{@html icon('chevronDown')}</div>
    </button>
</div>

<!-- ─── Menu Panel ──────────────────────────────────────────── -->
{#if isOpen}
    {#if useModal}
        <div class="u-hidden">
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
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
            id={menuId}
            class="c-menu c-menu--open"
            class:c-menu--modal={useModal}
            role="listbox"
            use:portal
            use:popover={{ triggerEl, align: 'left' }}
        >
        {#if isLongList && !useModal}
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

            {#if groups.length > 0}
                {#each filteredGroups as group}
                    <div class="c-menu__section">
                        <div class="c-menu__section-title">{group.title}</div>
                        {#each group.items as opt}
                            {@const optVal = opt.value ?? opt.href}
                            {@const isSel = optVal === value || (opt.isActive && typeof opt.isActive === 'function' && opt.isActive(value))}
                            <button
                                class="c-menu__item"
                                class:c-menu__item--selected={isSel}
                                class:c-menu__item--disabled={opt.disabled}
                                type="button"
                                role="option"
                                aria-selected={isSel}
                                disabled={opt.disabled}
                                onclick={(e) => selectOption(e, { ...opt, value: optVal })}
                            >
                                <span class="c-menu__item-label">{opt.label}</span>
                                {#if isSel}
                                    <span class="c-menu__item-check">{@html icon('check', 16)}</span>
                                {/if}
                            </button>
                        {/each}
                    </div>
                {/each}
            {:else}
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
            {/if}

            {#if (groups.length > 0 ? filteredGroups.length === 0 : filteredOptions.length === 0)}
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
