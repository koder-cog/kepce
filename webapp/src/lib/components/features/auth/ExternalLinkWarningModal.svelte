<script>
  import Modal from '$lib/components/ui/Modal.svelte';
  import { icon } from '$lib/components/ui/icons.js';

  let { url = '', onClose, onContinue } = $props();
</script>

<Modal
  options={{
    title: 'Dış Bağlantı',
    iconHtml: icon('warning', 24),
    iconColor: 'warning'
  }}
  {onClose}
>
  {#snippet children()}
    <p>
      Şu anda Kepçe'den ayrılıyorsunuz ve aşağıdaki bağlantıya yönlendirileceksiniz. Devam etmek
      istediğinize emin misiniz?
    </p>
    <div class="u-bg-sunken u-radius-lg u-break-all u-mt-md u-p-sm u-text-sm u-color-muted">
      {url}
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn btn--secondary" onclick={onClose}>İptal</button>
    <button class="btn btn--primary" onclick={onContinue}>Devam Et</button>
  {/snippet}
</Modal>
