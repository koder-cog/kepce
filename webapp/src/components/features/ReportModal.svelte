<script>
  import Modal from '../ui/Modal.svelte';
  import Dropdown from './Dropdown.svelte';
  import { icon } from '../ui/icons.js';
  import { api } from '../../api/index.js';
  import { showToast } from '../ui/toast.js';
  import { initCharCounter } from '../../utils/char-counter.js';
  import { onMount, tick } from 'svelte';

  let { mode = 'menu', targetId, reportBtn = null, onClose } = $props();

  const MODES = {
    menu: {
      title: 'Menü hata bildir',
      options: [
        { value: 'wrong_meal', label: 'Menü yanlış' },
        { value: 'typo',       label: 'Yazım hatası' },
        { value: 'other',      label: 'Diğer' },
      ],
      otherValue: 'other',
    },
    bot: {
      title: 'Yapay zeka yanıtını bildir',
      options: [
        { value: 'bot_incorrect',     label: 'Hatalı / yanlış bilgi' },
        { value: 'bot_incoherent',    label: 'Tutarsız / anlamsız yanıt' },
        { value: 'bot_inappropriate', label: 'Uygunsuz veya kaba dil' },
        { value: 'bot_other',         label: 'Diğer' },
      ],
      otherValue: 'bot_other',
    },
    comment: {
      title: 'Yorum şikayet et',
      options: [
        { value: 'spam', label: 'Spam/Reklam' },
        { value: 'inappropriate', label: 'Uygunsuz veya hakaret içeriyor' },
        { value: 'other', label: 'Diğer' },
      ],
      otherValue: 'other',
    },
    user: {
      title: 'Kullanıcıyı şikayet et',
      options: [
        { value: 'abusive', label: 'Rahatsız edici davranış' },
        { value: 'fake_account', label: 'Sahte hesap' },
        { value: 'other', label: 'Diğer' },
      ],
      otherValue: 'other',
    }
  };

  let cfg = $derived(MODES[mode]);

  let selectedType = $state();
  
  $effect(() => {
    if (cfg && !selectedType) {
        selectedType = cfg.options[0].value;
    }
  });
  let description = $state('');
  let isSubmitting = $state(false);
  let charOver = $state(false);

  let textareaEl = $state(null);

  onMount(() => {
    if (textareaEl) {
      initCharCounter(textareaEl, {
        onUpdate: (_count, _limit, isOver) => {
          charOver = isOver;
        }
      });
    }
  });

  let isOther = $derived(selectedType === cfg.otherValue);
  let placeholderText = $derived(isOther ? 'Lütfen sorunu burada açıklayınız...' : 'Neyin yanlış olduğunu kısaca belirt...');
  
  let submitDisabled = $derived(
    isSubmitting || 
    charOver || 
    (isOther && !description.trim())
  );

  let modalOptions = $derived({
    title: cfg.title,
    iconHtml: icon('warning', 24),
    iconColor: 'danger',
    disableEscape: false
  });

  async function submitReport() {
    isSubmitting = true;
    const desc = description.trim();

    try {
      await api.submitReport({
        target_type: mode,
        target_id: String(targetId),
        reason: selectedType,
        description: desc || null,
      });

      showToast('Bildirimin için teşekkürler.');

      if (reportBtn) {
        reportBtn.disabled = true;
        reportBtn.classList.add('u-opacity-disabled');
      }

      return true;
    } catch (err) {
      showToast(err.message, 'error');
      return false;
    } finally {
      isSubmitting = false;
    }
  }

  let controller = {};
</script>

{#if cfg}
  <Modal options={modalOptions} {onClose} {controller}>
      <div class="form-group">
        <div class="form-label">Sebep</div>
        <Dropdown options={cfg.options} bind:value={selectedType} />
      </div>
      <div class="form-group">
        <label class="form-label" for="report-desc">Açıklama</label>
        <textarea
          class="form-textarea form-textarea--resizable"
          id="report-desc"
          maxlength="512"
          placeholder={placeholderText}
          bind:value={description}
          bind:this={textareaEl}
        ></textarea>
      </div>

    {#snippet footer()}
      <button class="btn btn--secondary" onclick={() => controller?.close()} disabled={isSubmitting}>Vazgeç</button>
      <button class="btn btn--primary" onclick={async () => {
        const success = await submitReport();
        if (success) controller?.close();
      }} disabled={submitDisabled}>
        {isSubmitting ? 'Gönderiliyor...' : 'Bildir'}
      </button>
    {/snippet}
  </Modal>
{/if}
