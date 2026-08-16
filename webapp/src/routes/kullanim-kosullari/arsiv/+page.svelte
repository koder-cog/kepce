<script>
  import { LEGAL_VERSIONS } from "$lib/data/legal/versions.js";
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";

  const versions = [...LEGAL_VERSIONS["kullanim-kosullari"]].reverse();
</script>

<Seo
  title="Kullanım Koşulları Arşivi - Kepçe"
  description="Kepçe kullanım koşulları geçmiş sürümleri ve değişiklik kayıtları."
/>

<ContentPage title="Kullanım Koşulları Geçmiş Sürümler">
  <p>Aşağıda kullanım koşullarımızın geçmiş sürümlerini bulabilirsiniz.</p>
  <ul>
    {#each versions as v}
      <li>
        <a
          href={v.current
            ? "/kullanim-kosullari"
            : `/kullanim-kosullari/arsiv/${v.slug}`}
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
