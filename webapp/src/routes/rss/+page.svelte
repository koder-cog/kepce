<script>
  import { showToast } from "@/components/ui/toast.js";

  const rssUrl = `https://kepce.org/rss.xml`;
  let buttonText = $state("Aynen öyle");

  function handleCopy() {
    navigator.clipboard.writeText(rssUrl).then(() => {
      showToast("RSS bağlantısı panoya kopyalandı!", "success");
      buttonText = "Kopyalandı";
      setTimeout(() => {
        buttonText = "Aynen öyle";
      }, 2000);
    });
  }
</script>

<div class="content-page">
  <div class="content-page__header">
    <h1 class="content-page__title">RSS Akışı</h1>
  </div>
  <div class="content-page__body">
    <p>
      Menünün amele gibi siteye girmeden direkt ayağına gelmesini isteyen ama
      API yerine RSS tercih eden dedelere adanmış nostaljik amme hizmeti.
      Sınırlamalara tabidir.
    </p>

    <div class="u-flex u-flex-wrap u-flex-justify-between u-flex-align-center u-flex-gap-md u-mb-md">
      <h2 class="u-m-0">Bağlantıyı kopyala:</h2>
      <button class="btn btn--primary u-flex-shrink-0" onclick={handleCopy}>{buttonText}</button>
    </div>
  </div>
</div>

<svelte:head>
  <title>RSS - Kepçe</title>
</svelte:head>
