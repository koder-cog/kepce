/**
 * popover.js – Svelte Action
 * Paylaşılan konumlandırma motoru: Dropdown ve ActionMenu tarafından kullanılır.
 * Menüyü tetikleyicinin altına (veya üstüne) sabitler, ekran kenarlarını
 * dikkate alarak taşmayı engeller ve max-height'ı dinamik olarak hesaplar.
 */
export function popover(node, params) {
    let { triggerEl, align = 'left' } = params;

    const GAP = 4;       // Trigger ile menü arası boşluk (px)
    const EDGE = 16;     // Ekran kenarından minimum mesafe (px)
    const MAX_H = 300;   // Mutlak max yükseklik sınırı (px)

    function computePosition() {
        if (!triggerEl || !node) return;

        // Modal modda popover konumlandırması devre dışı
        if (node.classList.contains('c-menu--modal')) {
            node.style.position = '';
            node.style.top = '';
            node.style.bottom = '';
            node.style.left = '';
            node.style.right = '';
            node.style.transformOrigin = '';
            node.style.maxHeight = '';
            node.style.width = '';
            node.style.minWidth = '';
            return;
        }

        const rect = triggerEl.getBoundingClientRect();
        const scrollY = window.scrollY;
        const scrollX = window.scrollX;
        const vpH = window.innerHeight;
        const vpW = window.innerWidth;

        const spaceBelow = vpH - rect.bottom;
        const spaceAbove = rect.top;

        // Yukarı mı aşağı mı açılmalı?
        const shouldOpenUp = (spaceBelow - EDGE) < Math.min(MAX_H, 180) && spaceAbove > spaceBelow;

        // Dinamik max-height: mevcut alan ile mutlak sınırın küçük olanı
        const availableSpace = shouldOpenUp ? spaceAbove - EDGE : spaceBelow - EDGE;
        const dynamicMaxHeight = Math.min(MAX_H, Math.max(120, availableSpace));

        node.style.position = 'absolute';
        node.style.maxHeight = `${dynamicMaxHeight}px`;

        // Dikey konum
        node.style.bottom = 'auto';
        if (shouldOpenUp) {
            // Üstte açılırken, menünün yüksekliğini bilmemiz gerekiyor
            // Geçici olarak maxHeight'ı ayarlayıp offsetHeight okuyoruz
            const menuH = Math.min(node.scrollHeight, dynamicMaxHeight);
            node.style.top = `${rect.top + scrollY - menuH - GAP}px`;
            node.style.transformOrigin = 'bottom left';
            node.dataset.openingDirection = 'up';
        } else {
            node.style.top = `${rect.bottom + scrollY + GAP}px`;
            node.style.transformOrigin = 'top left';
            node.dataset.openingDirection = 'down';
        }

        // Yatay konum ve genişlik
        if (align === 'center') {
            const triggerCenter = rect.left + (rect.width / 2);
            const menuW = node.offsetWidth;
            let idealLeft = triggerCenter - (menuW / 2);

            if (idealLeft < EDGE) idealLeft = EDGE;
            else if (idealLeft + menuW > vpW - EDGE) idealLeft = vpW - menuW - EDGE;

            node.style.left = `${idealLeft + scrollX}px`;
            node.style.right = 'auto';
            node.style.minWidth = '';

            const originX = triggerCenter - idealLeft;
            node.style.transformOrigin = `${originX}px ${shouldOpenUp ? 'bottom' : 'top'}`;
        } else {
            // align = 'left' → Trigger genişliğiyle eşleş, sağdan taşmayı önle
            node.style.minWidth = `${rect.width}px`;
            node.style.left = `${rect.left + scrollX}px`;
            node.style.right = 'auto';

            // Sağdan taşıyor mu kontrol
            const menuW = node.offsetWidth;
            if (rect.left + menuW > vpW - EDGE) {
                node.style.left = 'auto';
                node.style.right = `${vpW - rect.right - scrollX}px`;
                node.style.transformOrigin = shouldOpenUp ? 'bottom right' : 'top right';
            }
        }
    }

    let positionTicking = false;
    function updatePosition() {
        if (positionTicking) return;
        positionTicking = true;
        window.requestAnimationFrame(() => {
            computePosition();
            positionTicking = false;
        });
    }

    window.addEventListener('resize', updatePosition);

    // İlk hesaplama – Çifte rAF ile DOM'un tamamen render olmasını bekler
    requestAnimationFrame(() => {
        computePosition();
        requestAnimationFrame(updatePosition);
    });

    return {
        update(newParams) {
            triggerEl = newParams.triggerEl;
            align = newParams.align || 'left';
            computePosition();
        },
        destroy() {
            window.removeEventListener('resize', updatePosition);
        }
    };
}
