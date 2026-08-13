import { mount, unmount } from 'svelte';
import TakeawayModalComponent from '../../components/features/TakeawayModal.svelte';

export function openTakeawayModal({ takeawayMenu, takeawayId, takeawayLabel, currentCity }) {
  const target = document.createElement('div');
  document.body.appendChild(target);

  const component = mount(TakeawayModalComponent, {
    target,
    props: {
      takeawayMenu,
      takeawayId,
      takeawayLabel,
      currentCity,
      onClose: () => {
        unmount(component);
        target.remove();
      }
    }
  });
}
