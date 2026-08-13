<script>
  import ContentPage from "@/components/layout/ContentPage.svelte";
  let { data } = $props();
</script>

<svelte:head>
  <title>Gizlilik Politikası (Arşiv: {data.versionData.version}) - Kepçe</title>
  <meta name="robots" content="noindex, nofollow" />
</svelte:head>

<ContentPage 
  title="Gizlilik Politikası (KVKK Aydınlatma Metni)" 
  dateLabel={`Arşivlenmiş sürüm tarihi: ${data.versionData.version}`}
  isArchived={true}
  archiveLink="/gizlilik-politikasi/arsiv"
>
  <!-- Dinamik olarak yüklenen arşivi render et -->
  <data.component />
</ContentPage>
