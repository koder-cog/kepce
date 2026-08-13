<script>
  import { LEGAL_VERSIONS } from "$lib/data/legal/versions.js";
  import ContentPage from "@/components/layout/ContentPage.svelte";

  const versions = [...LEGAL_VERSIONS["gizlilik-politikasi"]].reverse();
</script>

<svelte:head>
  <title>Gizlilik Politikası - Arşiv - Kepçe</title>
</svelte:head>

<ContentPage title="Gizlilik Politikası Geçmiş Sürümler">
  <p>Aşağıda gizlilik politikamızın geçmiş sürümlerini bulabilirsiniz.</p>
  <ul>
    {#each versions as v}
      <li>
        <a
          href={v.current
            ? "/gizlilik-politikasi"
            : `/gizlilik-politikasi/arsiv/${v.slug}`}
        >
          {v.current ? "Mevcut sürüm" : v.version}
        </a>
        {#if v.note}
          <br /><small class="u-color-muted">{v.note}</small>
        {/if}
      </li>
    {/each}
  </ul>
</ContentPage>
