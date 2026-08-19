<script>
    import { page } from '$app/stores';
    import { icon } from '@/components/ui/icons.js';
    import { onMount } from 'svelte';

    let { navConfig = [] } = $props();

    let isOpen = $state(false);
    let triggerEl = $state(null);
    let currentPath = $derived($page.url.pathname);

    function isLinkActive(link) {
        if (link.isActive) {
            return link.isActive(currentPath, $page.url);
        }
        const baseHref = link.href.split('?')[0];
        return currentPath === baseHref || currentPath.startsWith(baseHref + '/');
    }

    let activeItem = $derived((() => {
        for (const group of navConfig) {
            for (const link of group.links) {
                if (isLinkActive(link)) {
                    return { groupTitle: group.title, linkTitle: link.label, href: link.href };
                }
            }
        }
        return { groupTitle: 'Menü', linkTitle: 'Sayfa Seçiniz', href: '#' };
    })());

    $effect(() => {
        if (isOpen) {
            document.documentElement.classList.add('dropdown-open');
        } else {
            document.documentElement.classList.remove('dropdown-open');
        }
    });

    onMount(() => {
        const onNavMenuOpen = () => { if (isOpen) close(); };
        window.addEventListener('kepce:nav-menu-open', onNavMenuOpen);

        return () => {
            document.documentElement.classList.remove('dropdown-open');
            window.removeEventListener('kepce:nav-menu-open', onNavMenuOpen);
        };
    });

    function portal(node) {
        let parent = node.parentNode;
        let placeholder = document.createComment('portal-sidebar-sheet');
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

    function open(e) {
        if (e) e.stopPropagation();
        window.dispatchEvent(new CustomEvent('kepce:nav-menu-open'));
        isOpen = true;
    }

    function close() {
        isOpen = false;
    }

    function handleKeydown(e) {
        if (e.key === 'Escape' && isOpen) {
            close();
        }
    }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="sidebar-mobile-dropdown">
    <button
        bind:this={triggerEl}
        class="sidebar-mobile-trigger"
        class:sidebar-mobile-trigger--open={isOpen}
        type="button"
        aria-haspopup="dialog"
        aria-expanded={isOpen}
        onclick={open}
    >
        <div class="sidebar-mobile-trigger__content">
            <span class="sidebar-mobile-trigger__group">{activeItem.groupTitle}</span>
            <span class="sidebar-mobile-trigger__label">{activeItem.linkTitle}</span>
        </div>
        <div class="sidebar-mobile-trigger__action">
            <div class="sidebar-mobile-trigger__chevron" class:sidebar-mobile-trigger__chevron--rotated={isOpen}>
                {@html icon('chevronDown', 18)}
            </div>
        </div>
    </button>
</div>

<!-- ─── Modal Sheet ───────────────────────────────────────────── -->
{#if isOpen}
    <div class="u-hidden">
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
            class="c-menu__overlay c-menu__overlay--open"
            use:portal
            onclick={close}
            role="presentation"
        ></div>
    </div>

    <div class="u-hidden">
        <div
            class="c-menu c-menu--open c-menu--sheet"
            role="dialog"
            aria-modal="true"
            use:portal
            tabindex="-1"
        >
            <div class="c-menu__scroll-area">
                {#each navConfig as group}
                    <div class="c-menu__group">
                        <div class="c-menu__group-title">{group.title}</div>
                        {#each group.links as link}
                            {@const active = isLinkActive(link)}
                            <a
                                href={link.href}
                                class="c-menu__item"
                                class:c-menu__item--selected={active}
                                aria-current={active ? 'page' : undefined}
                                onclick={close}
                            >
                                <span class="c-menu__item-label">{link.label}</span>
                                {#if active}
                                    <span class="c-menu__item-check">
                                        {@html icon('check', 18)}
                                    </span>
                                {/if}
                            </a>
                        {/each}
                    </div>
                {/each}
            </div>
        </div>
    </div>
{/if}
