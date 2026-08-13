<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import CurrentContent from "$lib/data/legal/kullanim-kosullari/20260805.svelte";
</script>

<svelte:head>
  <title>Kullanım Koşulları - Kepçe</title>
</svelte:head>

<ContentPage 
  title="Kepçe Kullanım Koşulları" 
  dateLabel="Son güncelleme: 2026.08.05" 
  archiveLink="/kullanim-kosullari/arsiv"
>
  <CurrentContent />
</ContentPage>
