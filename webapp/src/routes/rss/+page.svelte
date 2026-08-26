<script>
  import "@/styles/pages/_content.css";
  import { showToast } from "@/components/ui/toast.js";
  import Seo from "@/components/ui/Seo.svelte";

  const rssUrl = `https://kepce.org/rss.xml`;
  let buttonText = $state("Bağlantıyı Kopyala");

  function handleCopy() {
    navigator.clipboard.writeText(rssUrl).then(() => {
      showToast("RSS bağlantısı panoya kopyalandı!", "success");
      buttonText = "Kopyalandı";
      setTimeout(() => {
        buttonText = "Bağlantıyı Kopyala";
      }, 2000);
    });
  }
</script>

<Seo
  title="Günlük RSS Menü Akışı | Kepçe"
  description="Günlük KYK yemek menülerini RSS okuyucunuz üzerinden anlık olarak takip edin."
  image="https://kepce.org/api/v1/public/og/page/rss"
/>

<div class="content-page">
  <div class="content-page__header">
    <h1 class="content-page__title">RSS Akışı</h1>
  </div>
  <div class="content-page__body">
    <p>
      Menüleri tarayıcı açmadan doğrudan RSS okuyucunuza veya bildirim botunuza
      çekmek isteyenler için hafif ve işlevsel bir akış. Günlük olarak güncellenir.
    </p>

    <div
      class="u-flex u-flex-wrap u-flex-justify-between u-flex-align-center u-flex-gap-md u-mb-md"
    >
      <h2 class="u-m-0">Bağlantıyı kopyala:</h2>
      <button class="btn btn--primary u-flex-shrink-0" onclick={handleCopy}
        >{buttonText}</button
      >
    </div>
  </div>
</div>
