/**
 * Kepçe Modular Modal System (M3 & GNOME HIG)
 */

import { icon } from '../ui/icons.js';
import { mount, unmount } from 'svelte';
import ModalComponent from '../ui/Modal.svelte';

let activeModalController = null;

export function createModal(options) {
    if (activeModalController) closeModal(activeModalController.modalElement);

    const target = document.createElement('div');
    document.body.appendChild(target);

    if ((!options.buttons || options.buttons.length === 0) && !options.disableEscape) {
        options.buttons = [{ label: 'Kapat', variant: 'secondary' }];
    }

    // Wrap options in Svelte state proxy using our controller hack for state reactivity
    // Or just rely on Svelte 5 runes passing object
    // Wait, passing an object into $state from outside works if we mutate it later
    const stateObj = { ...options };

    const controller = {};

    const modalApp = mount(ModalComponent, {
        target,
        props: {
            options: stateObj,
            controller,
            onClose: () => {
                unmount(modalApp);
                target.remove();
                if (activeModalController === controller) activeModalController = null;
            }
        }
    });

    activeModalController = controller;

    // Because mount is synchronous, target.firstChild is the actual modal DOM element
    const modalElement = target.firstChild;
    controller.modalElement = modalElement;

    return {
        modal: modalElement,
        close: () => {
            if (controller.close) controller.close();
            else {
                unmount(modalApp);
                target.remove();
            }
        },
        updateTitle: (newTitle) => {
            stateObj.title = newTitle;
            // Hack to trigger reactivity if simple object mutation isn't picked up
            // Ideally should be handled properly, but let's assume Svelte 5 catches object mutations if passed as state
            // If it doesn't, we might need a workaround. Let's just mutate the DOM for backward compatibility if reactivity fails
            const titleEl = modalElement.querySelector('.c-modal__title');
            if (titleEl) titleEl.textContent = newTitle;
        },
        updateContent: (newHtml) => {
            stateObj.contentHtml = newHtml;
            const contentEl = modalElement.querySelector('.c-modal__content');
            if (contentEl) contentEl.innerHTML = newHtml;
        }
    };
}

export function closeModal(modal) {
    if (!modal) return;
    if (activeModalController && activeModalController.modalElement === modal) {
        if (activeModalController.close) activeModalController.close();
    } else {
        // Fallback for custom modals not created by active controller
        modal.classList.remove('c-modal--open');
        document.body.style.overflow = '';
        setTimeout(() => {
            if (modal.parentNode) modal.parentNode.removeChild(modal);
        }, 350);
    }
}



