<svelte:head>
  <title>İstatistikler - Kepçe</title>
</svelte:head>

<script>
  import "@/styles/pages/_statistics.css";
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  onMount(() => {
    goto('/istatistikler/yemekler', { replaceState: true });
  });
</script>
