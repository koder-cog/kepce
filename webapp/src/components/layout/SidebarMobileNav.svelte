<script>
    import { page } from '$app/stores';
    import { icon } from '@/components/ui/icons.js';
    import { fade, slide } from 'svelte/transition';
    import { sineOut } from 'svelte/easing';

    let { navConfig = [] } = $props();

    let isOpen = $state(false);
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

    function toggle(e) {
        if (e) e.stopPropagation();
        isOpen = !isOpen;
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

<svelte:window onclick={close} onkeydown={handleKeydown} />

<div class="sidebar-mobile-dropdown dropdown" class:dropdown--open={isOpen}>
    {#if isOpen}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
            class="dropdown__overlay"
            transition:fade={{ duration: 150 }}
            onclick={toggle}
        ></div>
    {/if}

    <button
        class="dropdown__trigger sidebar-mobile-trigger"
        class:sidebar-mobile-trigger--open={isOpen}
        type="button"
        aria-haspopup="true"
        aria-expanded={isOpen}
        onclick={toggle}
    >
        <div class="sidebar-mobile-trigger__content">
            <span class="sidebar-mobile-trigger__group">{activeItem.groupTitle}</span>
            <span class="sidebar-mobile-trigger__label">{activeItem.linkTitle}</span>
        </div>
        <div class="sidebar-mobile-trigger__chevron" class:sidebar-mobile-trigger__chevron--rotated={isOpen}>
            {@html icon('chevronDown')}
        </div>
    </button>

    {#if isOpen}
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <nav
            class="dropdown__menu sidebar-mobile-menu"
            aria-label="Mobil Sayfa Gezintisi"
            transition:slide={{ duration: 180, easing: sineOut }}
            onclick={(e) => e.stopPropagation()}
        >
            <div class="dropdown__list sidebar-mobile-list">
                {#each navConfig as group, i}
                    {#if i > 0}
                        <div class="sidebar-dropdown-divider"></div>
                    {/if}
                    <div class="sidebar-dropdown-title">{group.title}</div>
                    {#each group.links as link}
                        {@const active = isLinkActive(link)}
                        <a
                            href={link.href}
                            class="dropdown__item sidebar-dropdown-item"
                            class:sidebar-dropdown-item--selected={active}
                            aria-current={active ? 'page' : undefined}
                            onclick={close}
                        >
                            <span class="sidebar-dropdown-item__label">{link.label}</span>
                            {#if active}
                                <span class="sidebar-dropdown-item__check">
                                    {@html icon('check')}
                                </span>
                            {/if}
                        </a>
                    {/each}
                {/each}
            </div>
        </nav>
    {/if}
</div>
