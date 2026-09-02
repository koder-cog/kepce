<script>
  let { children } = $props();
</script>

<svelte:head>
  <link
    rel="search"
    type="application/opensearchdescription+xml"
    title="Kepçe Ara"
    href="/ara/opensearch.xml"
  />
</svelte:head>

<div class="c-search-root">
  {@render children()}
</div>
