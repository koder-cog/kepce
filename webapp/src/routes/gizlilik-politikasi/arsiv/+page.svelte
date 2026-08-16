<script>
  import { LEGAL_VERSIONS } from "$lib/data/legal/versions.js";
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";

  const versions = [...LEGAL_VERSIONS["gizlilik-politikasi"]].reverse();
</script>

<Seo
  title="Gizlilik Politikası Arşivi - Kepçe"
  description="Kepçe gizlilik politikası ve KVKK metni geçmiş sürümleri ve değişiklik kayıtları."
/>

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
