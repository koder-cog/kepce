import { mount, unmount } from 'svelte';
import SpamInfoModalComponent from '$lib/components/features/moderation/SpamInfoModal.svelte';

export function openSpamInfoModal() {
  const target = document.createElement('div');
  document.body.appendChild(target);

  const controller = {
    close: null,
    modalElement: null
  };

  const component = mount(SpamInfoModalComponent, {
    target,
    props: {
      controller,
      onClose: () => {
        unmount(component);
        target.remove();
      }
    }
  });
}
