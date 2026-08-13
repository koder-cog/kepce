<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  let { data } = $props();
</script>

<svelte:head>
  <title>Kullanım Koşulları (Arşiv: {data.versionData.version}) - Kepçe</title>
  <meta name="robots" content="noindex, nofollow" />
</svelte:head>

<ContentPage 
  title="Kepçe Kullanım Koşulları" 
  dateLabel={`Arşivlenmiş sürüm tarihi: ${data.versionData.version}`}
  isArchived={true}
  archiveLink="/kullanim-kosullari/arsiv"
>
  <!-- Dinamik olarak yüklenen arşivi render et -->
  <data.component />
</ContentPage>
