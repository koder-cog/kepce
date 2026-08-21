<script>
  import "@/styles/pages/_admin.css";
  import { page } from "$app/stores";
  import { globalState } from "@/state.svelte.js";
  import { icon } from "@/components/ui/icons.js";
  import { slide } from "svelte/transition";
  import { backOut, sineIn } from "svelte/easing";
  import Seo from "@/components/ui/Seo.svelte";
  import Dropdown from "@/components/features/Dropdown.svelte";
  import { goto } from "$app/navigation";

  let { children } = $props();

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
</script>

<Seo title="Yönetim Paneli - Kepçe" noindex={true} />

<h1 class="sr-only">Yönetim Paneli</h1>

{#if globalState.isModerator}
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
