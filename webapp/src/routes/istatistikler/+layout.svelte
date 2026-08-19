<script>
  import '@/styles/pages/_statistics.css';
  import { page } from '$app/stores';
  import { icon } from '@/components/ui/icons.js';
  import { slide } from 'svelte/transition';
  import { backOut, sineIn } from 'svelte/easing';
  import Dropdown from '@/components/features/Dropdown.svelte';
  import { goto } from '$app/navigation';

  let { children } = $props();

  const navConfig = [
    {
      id: "yemekler",
      title: "Yemekler",
      links: [
        { href: "/istatistikler/yemekler", label: "Yemek analizleri" }
      ]
    },
    {
      id: "topluluk",
      title: "Topluluk",
      links: [
        { href: "/istatistikler/yorumlar", label: "En beğenilen yorumlar" },
        { href: "/istatistikler/insaniyet", label: "İnsaniyet tablosu" }
      ]
    },
    {
      id: "sistem",
      title: "Sistem",
      links: [
        { href: "/istatistikler/denetim", label: "Denetim hareketleri" }
      ]
    }
  ];

  let navState = $state({
    yemekler: true,
    topluluk: true,
    sistem: true
  });

  let currentPath = $derived($page.url.pathname);
  
  function isActive(href) {
    return currentPath === href || currentPath.startsWith(href + '/');
  }
</script>

<div class="sidebar-layout">
  
  <!-- MOBİL COMBO-BOX MENÜSÜ -->
  <div class="sidebar-mobile-nav">
    <Dropdown
      groups={navConfig}
      value={currentPath}
      onChange={(href) => goto(href)}
      placeholder="Sayfa Seçiniz"
    />
  </div>
  
  <!-- MASAÜSTÜ ADA SİDEBAR -->
  <aside class="sidebar-island">
    <nav class="sidebar__nav">
      {#each navConfig as group}
        <div class="sidebar-nav-group">
          <button class="sidebar-nav-group__title" class:open={navState[group.id]} onclick={() => navState[group.id] = !navState[group.id]}>
            {group.title}
            <div class="sidebar-nav-group__chevron">{@html icon('chevronDown')}</div>
          </button>
          {#if navState[group.id]}
            <div class="sidebar-nav-group__links" in:slide={{ duration: 400, easing: backOut }} out:slide={{ duration: 300, easing: sineIn }}>
              {#each group.links as link}
                <a 
                  href={link.href} 
                  class="sidebar-nav-link" 
                  class:sidebar-nav-link--active={link.isActive ? link.isActive(currentPath, $page.url) : isActive(link.href.split('?')[0])}
                >
                  {link.label}
                </a>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </nav>
  </aside>

  <!-- İÇERİK ALANI -->
  <section class="sidebar-content">
    {@render children()}
  </section>
</div>
