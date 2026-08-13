<script module>
    let activeActionMenuClose = null;
</script>

<script>
    import { animate, getDuration } from '../../lib/dom/motion.js';
    import { popover } from '../../lib/dom/popover.js';
    import { onMount, tick } from 'svelte';
    import { icon } from '../ui/icons.js';

    let {
        items = [], // [{ label, icon, variant: 'danger'|'accent', onClick }]
        triggerClass = 'btn-icon',
        triggerTitle = 'Menü',
    } = $props();

    let isOpen = $state(false);
    let triggerEl = $state(null);
    let menuEl = $state(null);

    let openTime = 0;
    let animation = null;

    const isMobile = () => window.innerWidth <= 600;

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

    function onOutsideClick(e) {
        if (isOpen && triggerEl && !triggerEl.contains(e.target) && menuEl && !menuEl.contains(e.target)) {
            close();
        }
    }

    function onScrollClose(e) {
        if (!isOpen) return;
        if (Date.now() - openTime < 100) return;
        const path = e.composedPath ? e.composedPath() : [];
        if (path.some(el => el === menuEl)) return;
        close();
    }

    async function open() {
        if (isOpen) return;
        if (activeActionMenuClose && activeActionMenuClose !== close) {
            activeActionMenuClose();
        }
        activeActionMenuClose = close;
        isOpen = true;
        openTime = Date.now();

        await tick();

        if (animation) animation.cancel();
        
        // Let popover.js run its first tick, then animate
        requestAnimationFrame(() => {
            if (!menuEl) return;
            const isUp = menuEl.dataset.openingDirection === 'up';
            animation = animate(menuEl, [
                { opacity: 0, transform: `scale(0.95) translateY(${isUp ? '8px' : '-8px'})` },
                { opacity: 1, transform: 'scale(1) translateY(0)' }
            ], { duration: getDuration(450), easing: 'cubic-bezier(0.34, 1.56, 0.64, 1)' });
        });
    }

    function close() {
        if (!isOpen) return;
        if (activeActionMenuClose === close) activeActionMenuClose = null;

        if (animation) animation.cancel();
        
        if (menuEl) {
            const isUp = menuEl.dataset.openingDirection === 'up';
            animation = animate(menuEl, [
                { opacity: 1, transform: 'scale(1) translateY(0)' },
                { opacity: 0, transform: `scale(0.96) translateY(${isUp ? '4px' : '-4px'})` }
            ], { duration: getDuration(180), easing: 'ease-in' });
            
            animation.onfinish = () => {
                isOpen = false;
            };
        } else {
            isOpen = false;
        }
    }

    function toggle(e) {
        if (e) e.stopPropagation();
        isOpen ? close() : open();
    }

    function handleKeyDown(e) {
        if (!isOpen) return;
        if (e.key === 'Escape') { e.preventDefault(); close(); }
    }

    function handleItemClick(e, item) {
        e.stopPropagation();
        close();
        if (item.onClick) item.onClick();
    }
</script>

<svelte:window 
    onscroll={onScrollClose}
    onclick={onOutsideClick}
    onkeydown={handleKeyDown}
/>

<div class="admin-action-menu-container">
    <button
        bind:this={triggerEl}
        class={triggerClass}
        type="button"
        title={triggerTitle}
        aria-haspopup="menu"
        aria-expanded={isOpen}
        onclick={toggle}
    >
        {@html icon('more', 16)}
    </button>
</div>

{#if isOpen}
    <div class="u-hidden">
        <div 
            bind:this={menuEl} 
            class="c-menu c-menu--open" 
            role="menu" 
            use:portal 
            use:popover={{ triggerEl, align: 'center' }}
        >
        {#each items as item}
            {#if item.divider}
                <div class="c-menu__divider"></div>
            {:else}
                <button 
                    class="c-menu__item {item.variant === 'danger' ? 'c-menu__item--danger' : ''} {item.variant === 'accent' ? 'c-menu__item--accent' : ''}" 
                    role="menuitem"
                    onclick={(e) => handleItemClick(e, item)}
                >
                    {#if item.icon}
                        {@html icon(item.icon, 18)}
                    {/if}
                    {item.label}
                </button>
            {/if}
        {/each}
        </div>
    </div>
{/if}
