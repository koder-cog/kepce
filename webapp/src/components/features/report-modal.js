import { mount, unmount } from 'svelte';
import ReportModalComponent from './ReportModal.svelte';

/**
 * @param {string} mode - 'menu', 'bot', 'comment', or 'user'
 * @param {string|number} targetId - The ID of the item being reported
 * @param {HTMLElement} reportBtn - The button element that triggered the modal (to disable it after submit)
 */
export function openReportModal(mode, targetId, reportBtn = null) {
  const target = document.createElement('div');
  document.body.appendChild(target);

  const component = mount(ReportModalComponent, {
    target,
    props: {
      mode,
      targetId,
      reportBtn,
      onClose: () => {
        unmount(component);
        target.remove();
      }
    }
  });
}

export function openMenuReportModal(menu, reportBtn = null) {
  openReportModal('menu', menu?.id, reportBtn);
}

export function openBotReportModal(menu) {
  openReportModal('bot', menu?.id || 0);
}

export function openCommentReportModal(comment, reportBtn = null) {
  openReportModal('comment', comment?.id, reportBtn);
}

export function openUserReportModal(userId, reportBtn = null) {
  openReportModal('user', userId, reportBtn);
}
