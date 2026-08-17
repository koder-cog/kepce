<script>
  import "@/styles/pages/_admin.css";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { globalState } from "@/state.svelte.js";
  import { onMount } from "svelte";
  import { icon } from "@/components/ui/icons.js";
  import { slide, fade } from "svelte/transition";
  import { backOut, sineIn } from "svelte/easing";
  import Seo from "@/components/ui/Seo.svelte";

  function dropdownAnim(node, { duration = 200, easing = sineIn }) {
    return {
      duration,
      easing,
      css: (t) => `
        opacity: ${t};
        transform: scale(${0.96 + 0.04 * t}) translateY(${-4 + 4 * t}px);
      `,
    };
  }

  let { children } = $props();

  let mobileMenuOpen = $state(false);
  const navConfig = [
    {
      id: "mutfak",
      title: "Mutfak",
      links: [
        { href: "/moderasyon/mutfak/yemekler", label: "Yemekler" },
        { href: "/moderasyon/mutfak/tabela", label: "Tabela" },
        { href: "/moderasyon/mutfak/bot", label: "Kepçe Bot" },
      ],
    },
    {
      id: "yorumlar",
      title: "Yorumlar",
      links: [{ href: "/moderasyon/yorumlar", label: "Tüm yorumlar" }],
    },
    {
      id: "denetim",
      title: "Denetim",
      links: [
        {
          href: "/moderasyon/denetim/sikayetler?tip=icerik",
          label: "İçerik şikayetleri",
          isActive: (path, url) =>
            path.includes("/denetim/sikayetler") &&
            url.searchParams.get("tip") === "icerik",
        },
        {
          href: "/moderasyon/denetim/sikayetler?tip=hata",
          label: "Hata bildirimleri",
          isActive: (path, url) =>
            path.includes("/denetim/sikayetler") &&
            url.searchParams.get("tip") === "hata",
        },
        {
          href: "/moderasyon/denetim/sikayetler?tip=iletisim",
          label: "İletişim mesajları",
          isActive: (path, url) =>
            path.includes("/denetim/sikayetler") &&
            url.searchParams.get("tip") === "iletisim",
        },
        { href: "/moderasyon/denetim/kullanicilar", label: "Kullanıcılar" },
      ],
    },
    {
      id: "altyapi",
      title: "Altyapı",
      links: [
        { href: "/moderasyon/altyapi/sistem-sagligi", label: "Sistem sağlığı" },
        { href: "/moderasyon/altyapi/olaylar", label: "Olaylar" },
        { href: "/moderasyon/altyapi/etiketler", label: "Etiketler" },
      ],
    },
  ];

  let navState = $state({
    mutfak: true,
    yorumlar: true,
    denetim: true,
    altyapi: true,
  });

  let currentPath = $derived($page.url.pathname);

  function isActive(href) {
    return currentPath === href || currentPath.startsWith(href + "/");
  }

  let mobileMenuLabel = $derived(
    (() => {
      for (const group of navConfig) {
        for (const link of group.links) {
          if (
            link.isActive
              ? link.isActive(currentPath, $page.url)
              : isActive(link.href.split("?")[0])
          ) {
            return link.label;
          }
        }
      }
      return "Menü";
    })(),
  );

  function toggleMobileMenu(e) {
    if (e) e.stopPropagation();
    mobileMenuOpen = !mobileMenuOpen;
  }

  function handleBodyClick() {
    if (mobileMenuOpen) mobileMenuOpen = false;
  }
</script>

<svelte:window onclick={handleBodyClick} />

<Seo title="Yönetim Paneli - Kepçe" noindex={true} />

{#if globalState.isModerator}
  <div class="sidebar-layout">
    <!-- MOBİL İÇİN ÖZEL DROPDOWN (standart dropdown kullanılarak) -->
    <div
      class="sidebar-mobile-dropdown dropdown"
      class:dropdown--open={mobileMenuOpen}
    >
      {#if mobileMenuOpen}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="dropdown__overlay"
          transition:fade={{ duration: 200 }}
          onclick={toggleMobileMenu}
        ></div>
      {/if}
      <button
        class="dropdown__trigger sidebar-mobile-trigger"
        class:dropdown__trigger--open={mobileMenuOpen}
        onclick={toggleMobileMenu}
      >
        <span>{mobileMenuLabel}</span>
        <div class="dropdown__chevron">{@html icon("chevronDown")}</div>
      </button>

      {#if mobileMenuOpen}
        <div class="dropdown__menu" transition:dropdownAnim>
          <div class="dropdown__list">
            {#each navConfig as group, i}
              {#if i > 0}
                <div class="sidebar-dropdown-divider"></div>
              {/if}
              <div class="sidebar-dropdown-title">{group.title}</div>
              {#each group.links as link}
                <a
                  href={link.href}
                  class="dropdown__item sidebar-dropdown-item"
                  class:dropdown__item--selected={link.isActive
                    ? link.isActive(currentPath, $page.url)
                    : isActive(link.href.split("?")[0])}
                >
                  {link.label}
                </a>
              {/each}
            {/each}
          </div>
        </div>
      {/if}
    </div>

    <!-- MASAÜSTÜ ADA SİDEBAR -->
    <aside class="sidebar-island">
      <nav class="sidebar__nav">
        {#each navConfig as group}
          <div class="sidebar-nav-group">
            <button
              class="sidebar-nav-group__title"
              class:open={navState[group.id]}
              onclick={() => (navState[group.id] = !navState[group.id])}
            >
              {group.title}
              <div class="sidebar-nav-group__chevron">
                {@html icon("chevronDown")}
              </div>
            </button>
            {#if navState[group.id]}
              <div
                class="sidebar-nav-group__links"
                in:slide={{ duration: 400, easing: backOut }}
                out:slide={{ duration: 300, easing: sineIn }}
              >
                {#each group.links as link}
                  <a
                    href={link.href}
                    class="sidebar-nav-link"
                    class:sidebar-nav-link--active={link.isActive
                      ? link.isActive(currentPath, $page.url)
                      : isActive(link.href.split("?")[0])}
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
{/if}
