<script>
    import { page } from '$app/stores';
    import { icon } from '@/components/ui/icons.js';
    import { onMount } from 'svelte';

    let { navConfig = [] } = $props();

    let isOpen = $state(false);
    let triggerEl = $state(null);
    let modalEl = $state(null);
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
        let placeholder = document.createComment('portal-sidebar-nav');
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
            <span class="sidebar-mobile-trigger__badge">Değiştir</span>
            <div class="sidebar-mobile-trigger__chevron" class:sidebar-mobile-trigger__chevron--rotated={isOpen}>
                {@html icon('chevronDown')}
            </div>
        </div>
    </button>
</div>

<!-- ─── Modal Sheet Portal ────────────────────────────────────── -->
{#if isOpen}
    <div class="c-modal c-modal--open c-sheet-nav" use:portal role="dialog" aria-modal="true">
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
            class="c-modal__backdrop"
            onclick={close}
        ></div>

        <div
            bind:this={modalEl}
            class="c-modal__surface c-sheet-nav__surface"
        >
            <div class="c-modal__header c-sheet-nav__header">
                <div class="c-sheet-nav__title-group">
                    <span class="c-sheet-nav__tag">Sayfa Gezintisi</span>
                    <h3 class="c-sheet-nav__title">Bölüm Seçiniz</h3>
                </div>
                <button class="c-sheet-nav__close btn-icon" type="button" aria-label="Kapat" onclick={close}>
                    {@html icon('crossSmall')}
                </button>
            </div>

            <div class="c-modal__body c-sheet-nav__body">
                {#each navConfig as group}
                    <div class="c-sheet-nav__group">
                        <div class="c-sheet-nav__group-header">
                            <span class="c-sheet-nav__group-title">{group.title}</span>
                        </div>
                        <div class="c-sheet-nav__links">
                            {#each group.links as link}
                                {@const active = isLinkActive(link)}
                                <a
                                    href={link.href}
                                    class="c-sheet-nav__link"
                                    class:c-sheet-nav__link--active={active}
                                    aria-current={active ? 'page' : undefined}
                                    onclick={close}
                                >
                                    <span class="c-sheet-nav__link-label">{link.label}</span>
                                    {#if active}
                                        <span class="c-sheet-nav__check">
                                            {@html icon('check')}
                                        </span>
                                    {/if}
                                </a>
                            {/each}
                        </div>
                    </div>
                {/each}
            </div>
        </div>
    </div>
{/if}
