<script>
  import Modal from "@/components/ui/Modal.svelte";
  import { icon } from "@/components/ui/icons.js";

  let { url = "", onClose, onContinue } = $props();
</script>

<Modal
  options={{
    title: "Dış Bağlantı",
    iconHtml: icon("warning", 24),
    iconColor: "warning",
  }}
  {onClose}
>
  {#snippet children()}
    <p>
      Şu anda Kepçe'den ayrılıyorsunuz ve aşağıdaki bağlantıya
      yönlendirileceksiniz. Devam etmek istediğinize emin misiniz?
    </p>
    <div class="external-url-box u-mt-md u-p-sm u-text-sm u-color-muted">
      {url}
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn btn--secondary" onclick={onClose}>İptal</button>
    <button class="btn btn--primary" onclick={onContinue}>Devam Et</button>
  {/snippet}
</Modal>

<style>
  .external-url-box {
    background-color: var(--bg-surface-2);
    border-radius: var(--radius-md);
    word-break: break-all;
  }
</style>
