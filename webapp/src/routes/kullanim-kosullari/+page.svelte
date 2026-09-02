<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import CurrentContent from "$lib/data/legal/kullanim-kosullari/20260902.svelte";
  import Seo from "@/components/ui/Seo.svelte";
</script>

<Seo
  title="Kullanım Koşulları - Kepçe"
  description="Kepçe platformu kullanım koşulları, kullanıcı hakları, sorumluluklar ve yasal kurallar."
  image="https://kepce.org/api/v1/public/og/page/kullanim-kosullari"
/>

<ContentPage
  title="Kepçe Kullanım Koşulları"
  dateLabel="Son güncelleme: 2026.09.02"
  archiveLink="/kullanim-kosullari/arsiv"
>
  <CurrentContent />
</ContentPage>
