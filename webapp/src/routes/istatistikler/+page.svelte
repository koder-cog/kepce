<script>
  import "@/styles/pages/_statistics.css";
  import Seo from "@/components/ui/Seo.svelte";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  onMount(() => {
    goto("/istatistikler/yemekler", { replaceState: true });
  });
</script>

<Seo
  title="Yemekhane İstatistikleri | Kepçe"
  description="KYK yurtlarında en sık çıkan yemekler, öğrenci puanlamaları ve menü verileri."
  image="https://kepce.org/api/v1/public/og/page/istatistikler"
/>
<h1 class="sr-only">İstatistikler</h1>
