<script>
  import { onMount, onDestroy } from 'svelte';
  import { subscribeToasts, dismissActiveToast } from './toast.js';
  import { icon } from './icons.js';

  let activeToast = $state(null);
  let queue = $state([]);

  let unsubscribe;

  onMount(() => {
    unsubscribe = subscribeToasts(state => {
      activeToast = state.activeToast;
      queue = state.toastQueue;
    });
  });

  onDestroy(() => {
    if (unsubscribe) unsubscribe();
  });

  function progressAnim(node, duration) {
    node.style.transform = 'scaleX(1)';
    requestAnimationFrame(() => {
      node.style.transition = `transform ${duration}ms linear`;
      node.style.transform = 'scaleX(0)';
    });
    
    return {
      update(newDuration) {
        node.style.transition = 'none';
        node.style.transform = 'scaleX(1)';
        requestAnimationFrame(() => {
          node.style.transition = `transform ${newDuration}ms linear`;
          node.style.transform = 'scaleX(0)';
        });
      }
    }
  }
</script>

{#if activeToast || queue.length > 0}
  <!-- #71: Dinamik bildirimler ekran okuyucuya duyurulur -->
  <div class="c-toast-container" role="status" aria-live="polite">
    {#if activeToast}
      {#key activeToast.id}
        <div class="c-toast {activeToast.isClosing ? 'c-toast--closing' : ''}">
          <p class="c-toast__message">{activeToast.message}</p>
          
          {#if activeToast.action}
            <button class="c-toast__action" onclick={() => {
              if (activeToast.action.callback) activeToast.action.callback();
              dismissActiveToast();
            }}>
              {activeToast.action.text}
            </button>
          {/if}

          <div class="c-toast__progress" use:progressAnim={activeToast.timeoutMs}></div>

          <button class="c-toast__close" aria-label="Kapat" onclick={(e) => {
            e.stopPropagation();
            dismissActiveToast();
          }}>
            {@html icon('close', 16)}
          </button>
        </div>
      {/key}
    {/if}
  </div>
{/if}
