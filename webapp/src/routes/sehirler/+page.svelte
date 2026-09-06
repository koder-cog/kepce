<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";

  let { data } = $props();

  const breadcrumbs = [
    { name: "Ana Sayfa", item: "https://kepce.org/" },
    { name: "Şehirler", item: "https://kepce.org/sehirler" },
  ];

  const citySchema = {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "CollectionPage",
        "@id": "https://kepce.org/sehirler#webpage",
        url: "https://kepce.org/sehirler",
        name: "KYK Yemek Menüsü Çıkan Şehirler | Kepçe",
        description: "Kepçe üzerinde KYK yurt yemek menüsü bulunan şehirler ve tabldot listeleri.",
        inLanguage: "tr-TR",
      },
      {
        "@type": "BreadcrumbList",
        itemListElement: breadcrumbs.map((b, idx) => ({
          "@type": "ListItem",
          position: idx + 1,
          name: b.name,
          item: b.item,
        })),
      },
    ],
  };
</script>

<Seo
  title="KYK Yemek Menüsü Çıkan Şehirler | Kepçe"
  description="Kepçe üzerinde KYK yurt yemek menüsü bulunan şehirler. Güncel ve geçmiş dönem tabldot listesi olan iller."
  image="https://kepce.org/og_image.png"
  canonical="https://kepce.org/sehirler"
  schema={citySchema}
/>

<ContentPage title="Menüsü Olan Şehirler">
  <p class="city-directory__intro">
    Kepçe'de tabldot menü verisi bulunan şehirler, güncellik durumuna göre aşağıda listelenmiştir. Şehrinizi seçerek menüleri inceleyebilirsiniz.
  </p>

  <div class="city-directory">
    {#each data.groups as group (group.id)}
      <section class="city-directory__group">
        <h2 class="city-directory__heading">{group.title}</h2>
        <p class="city-directory__desc">{group.description}</p>

        <div class="city-directory__grid">
          {#each group.cities as city (city.slug)}
            <a href="/{city.slug}" class="city-directory__card" data-link>
              <span class="city-directory__name">{city.name}</span>
              {#if city.lastmodLabel}
                <span class="city-directory__date">{city.lastmodLabel}</span>
              {/if}
            </a>
          {/each}
        </div>
      </section>
    {/each}
  </div>
</ContentPage>
