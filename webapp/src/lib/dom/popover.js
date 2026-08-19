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

        // ── Phase 1: DOM Reads (Single Batch) ───────────────────
        const rect = triggerEl.getBoundingClientRect();
        const nodeRect = node.getBoundingClientRect();
        const nodeScrollHeight = node.scrollHeight;
        const scrollY = window.scrollY;
        const scrollX = window.scrollX;
        const vpH = window.innerHeight;
        const vpW = window.innerWidth;

        const spaceBelow = vpH - rect.bottom;
        const spaceAbove = rect.top;

        // ── Phase 2: Pure Calculations ──────────────────────────
        const shouldOpenUp = (spaceBelow - EDGE) < Math.min(MAX_H, 180) && spaceAbove > spaceBelow;
        const availableSpace = shouldOpenUp ? spaceAbove - EDGE : spaceBelow - EDGE;
        const dynamicMaxHeight = Math.min(MAX_H, Math.max(120, availableSpace));

        let topStyle = '';
        let bottomStyle = 'auto';
        let leftStyle = '';
        let rightStyle = 'auto';
        let minWidthStyle = '';
        let transformOriginStyle = '';
        let openingDir = shouldOpenUp ? 'up' : 'down';

        if (shouldOpenUp) {
            const menuH = Math.min(nodeScrollHeight || nodeRect.height, dynamicMaxHeight);
            topStyle = `${rect.top + scrollY - menuH - GAP}px`;
            transformOriginStyle = 'bottom left';
        } else {
            topStyle = `${rect.bottom + scrollY + GAP}px`;
            transformOriginStyle = 'top left';
        }

        const initialMenuW = Math.max(nodeRect.width, rect.width);
        if (align === 'center') {
            const triggerCenter = rect.left + (rect.width / 2);
            let idealLeft = triggerCenter - (initialMenuW / 2);

            if (idealLeft < EDGE) idealLeft = EDGE;
            else if (idealLeft + initialMenuW > vpW - EDGE) idealLeft = vpW - initialMenuW - EDGE;

            leftStyle = `${idealLeft + scrollX}px`;
            const originX = triggerCenter - idealLeft;
            transformOriginStyle = `${originX}px ${shouldOpenUp ? 'bottom' : 'top'}`;
        } else {
            minWidthStyle = `${rect.width}px`;
            if (rect.left + initialMenuW > vpW - EDGE) {
                leftStyle = 'auto';
                rightStyle = `${vpW - rect.right - scrollX}px`;
                transformOriginStyle = shouldOpenUp ? 'bottom right' : 'top right';
            } else {
                leftStyle = `${rect.left + scrollX}px`;
            }
        }

        // ── Phase 3: DOM Writes (Single Batch) ──────────────────
        node.style.position = 'absolute';
        node.style.maxHeight = `${dynamicMaxHeight}px`;
        node.style.top = topStyle;
        node.style.bottom = bottomStyle;
        node.style.left = leftStyle;
        node.style.right = rightStyle;
        node.style.minWidth = minWidthStyle;
        node.style.transformOrigin = transformOriginStyle;
        node.dataset.openingDirection = openingDir;
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
