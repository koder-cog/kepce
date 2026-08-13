<script>
  import { page } from '$app/stores';

  let {
    title = 'Kepçe - Yurt Yemek Menüsü',
    description = 'KYK yurt menüleri için bağımsız, açık kaynaklı ve reklamsız platform.',
    image = 'https://kepce.org/og_image.png',
    type = 'website'
  } = $props();

  let url = $derived($page.url.href);
</script>

<svelte:head>
  <title>{title}</title>
  <meta name="description" content={description} />

  <!-- Open Graph / Facebook -->
  <meta property="og:type" content={type} />
  <meta property="og:url" content={url} />
  <meta property="og:title" content={title} />
  <meta property="og:description" content={description} />
  <meta property="og:image" content={image} />

  <!-- Twitter -->
  <meta property="twitter:card" content="summary_large_image" />
  <meta property="twitter:url" content={url} />
  <meta property="twitter:title" content={title} />
  <meta property="twitter:description" content={description} />
  <meta property="twitter:image" content={image} />
</svelte:head>
