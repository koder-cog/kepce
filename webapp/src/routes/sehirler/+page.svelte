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
        description: "Türkiye genelinde KYK yurt yemek menüsü yayımlanan aktif iller ve günlük tabldot listeleri.",
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
  description="Bugün KYK yurtlarında yemek menüsü yayımlanan aktif iller. İstanbul, Ankara, İzmir ve diğer şehirlerin günlük kahvaltı ve akşam tabldot listeleri."
  image="https://kepce.org/og_image.png"
  canonical="https://kepce.org/sehirler"
  schema={citySchema}
/>

<ContentPage title="Menüsü Olan Şehirler">
  <p>
    Kepçe üzerinde günlük ve aylık tabldot menü verisi doğrulanmış aktif iller
    aşağıda listelenmiştir. Şehrinizi seçerek bugünün kahvaltı ve akşam yemeği
    menüsünü inceleyebilirsiniz.
  </p>

  <ul>
    {#each data.cities as city (city.slug)}
      <li>
        <a href="/{city.slug}" data-link>
          {city.name}
        </a>
      </li>
    {/each}
  </ul>
</ContentPage>
