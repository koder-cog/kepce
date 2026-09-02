<script>
  import { page } from "$app/stores";

  let {
    title = "KYK Yemek Menüsü - Bugün KYK'da Ne Yemek Var? | Kepçe",
    description = "Bugün KYK yurtlarında çıkan kahvaltı ve akşam yemeği menüsü. 81 il reklamsız, güncel tabldot listeleri, kalori ve beslenme yardımı detayları.",
    image = "https://kepce.org/og_image.png",
    type = "website",
    canonical = null,
    noindex = false,
    schema = null,
  } = $props();

  const BASE_URL = "https://kepce.org";

  let canonicalUrl = $derived.by(() => {
    if (canonical) {
      return canonical.replace(/\/+$/, "") || BASE_URL;
    }
    const path = ($page.url.pathname || "").replace(/\/+$/, "");
    return path ? `${BASE_URL}${path}` : BASE_URL;
  });

  let defaultSchema = $derived(
    schema || {
      "@context": "https://schema.org",
      "@graph": [
        {
          "@type": "WebSite",
          "@id": `${BASE_URL}/#website`,
          url: `${BASE_URL}/`,
          name: "Kepçe",
          alternateName: ["Kepçe KYK", "KYK Yemek Menüsü", "KYK Yemek Listesi"],
          description: "Bugün KYK'da Ne Yemek Var? Günlük KYK Yurt Menüleri",
          inLanguage: "tr-TR",
          potentialAction: {
            "@type": "SearchAction",
            target: `${BASE_URL}/ara?q={search_term_string}`,
            "query-input": "required name=search_term_string",
          },
        },
        {
          "@type": "Organization",
          "@id": `${BASE_URL}/#organization`,
          name: "Kepçe",
          url: `${BASE_URL}/`,
          logo: `${BASE_URL}/icon-512.png`,
        },
      ],
    },
  );
</script>

<svelte:head>
  <title>{title}</title>
  <meta name="description" content={description} />
  <link rel="canonical" href={canonicalUrl} />

  <!-- Robots -->
  {#if noindex}
    <meta name="robots" content="noindex, follow" />
  {:else}
    <meta name="robots" content="index, follow" />
  {/if}

  <!-- Open Graph / Facebook -->
  <meta property="og:site_name" content="Kepçe - KYK Yemek Menüsü" />
  <meta property="og:locale" content="tr_TR" />
  <meta property="og:type" content={type} />
  <meta property="og:url" content={canonicalUrl} />
  <meta property="og:title" content={title} />
  <meta property="og:description" content={description} />
  <meta property="og:image" content={image} />

  <!-- Twitter -->
  <meta property="twitter:card" content="summary_large_image" />
  <meta property="twitter:url" content={canonicalUrl} />
  <meta property="twitter:title" content={title} />
  <meta property="twitter:description" content={description} />
  <meta property="twitter:image" content={image} />

  <!-- Structured Data (JSON-LD) -->
  {@html `<script type="application/ld+json">${JSON.stringify(defaultSchema)}</script>`}
</svelte:head>
