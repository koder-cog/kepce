/**
 * Auth Gate yöneticisi - Görev #13-15
 *
 * `authActions.triggerLogin()` üzerinden çağrılır; sayfa yönlendirmesi
 * yapmak yerine AuthGateModal'i imperative olarak mount eder.
 * modal.js'teki createModal kalıbını takip eder (tek aktif örnek).
 */
import { mount, unmount } from 'svelte';
import AuthGateModal from './AuthGateModal.svelte';

let activeGate = null;

export function openAuthGate(options = {}) {
    // Zaten açıksa yeni bir tane açma
    if (activeGate) return activeGate;

    const target = document.createElement('div');
    document.body.appendChild(target);

    const app = mount(AuthGateModal, {
        target,
        props: {
            reason: options.reason || null,
            onClose: () => {
                unmount(app);
                target.remove();
                if (activeGate?.target === target) activeGate = null;
            }
        }
    });

    activeGate = {
        target,
        close: () => {
            // Bileşenin export ettiği close() çıkış animasyonunu oynatır;
            // onClose zinciri unmount + temizliği halleder.
            if (app && app.close) app.close();
        }
    };

    return activeGate;
}
