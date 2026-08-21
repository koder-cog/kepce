<script>
  import "@/styles/pages/_statistics.css";
  import Seo from "@/components/ui/Seo.svelte";
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  onMount(() => {
    goto('/istatistikler/yemekler', { replaceState: true });
  });
</script>

<Seo title="İstatistikler - Kepçe" description="KYK yemekhaneleri öğrenci istatistikleri ve yemek analizleri." />

<h1 class="sr-only">İstatistikler</h1>
