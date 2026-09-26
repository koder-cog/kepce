<script module>
    let activeDropdownClose = null;
</script>

<script>
    import { icon } from "../ui/icons.js";
    import { animate, getDuration } from "../../lib/dom/motion.js";
    import { popover } from "../../lib/dom/popover.js";
    import { lockScroll, unlockScroll } from "../../lib/dom/scroll-lock.js";
    import { nativeBridge } from "../../lib/native/bridge.js";
    import { onMount, tick } from "svelte";

    let {
        options = [],
        groups = [],
        value = $bindable(),
        placeholder = "Seçiniz",
        ariaLabel = undefined,
        id = null,
        disabled = false,
        variant = "primary", // primary | secondary | ghost
        actionItem = null,
        onActionClick = null,
        specialItem = null,
        onSpecialClick = null,
        onChange = null,
        forceModal = false,
    } = $props();

    // ── State ──────────────────────────────────────────────────
    let isOpen = $state(false);
    let triggerEl = $state(null);
    let menuEl = $state(null);
    let overlayEl = $state(null);
    let listEl = $state(null);
    let searchInputEl = $state(null);
    let highlightedIndex = $state(-1);
    let searchQuery = $state("");
    let isMobile = $state(false);
    let sheetSnap = $state("default"); // "default" | "expanded"

    let searchBuffer = "";
    let searchTimeout = null;
    let openTime = 0;
    let isProgrammaticScroll = false;
    let animation = null;
    let overlayAnimation = null;
    let pushedState = false;
    let hasKeyboardNavigated = $state(false);

    function isIOSDevice() {
        if (typeof navigator === "undefined") return false;
        if (navigator.userAgentData?.platform === "iOS") return true;
        return (
            /iPad|iPhone|iPod/.test(navigator.userAgent) ||
            (navigator.userAgent.includes("Macintosh") &&
                navigator.maxTouchPoints > 1)
        );
    }

    let dragStartY = 0;
    let dragStartTime = 0;
    let dragStartHeight = 0;
    let isDragging = false;
    let dragFromHandle = false;
    let dragMode = "none";
    let currentDragY = 0;

    // ── Derived ────────────────────────────────────────────────
    let allOptions = $derived(
        groups.length > 0
            ? groups.flatMap((g) =>
                  (g.options || g.links || []).map((item) => ({
                      ...item,
                      value: item.value ?? item.href,
                      label: item.label,
                      groupTitle: g.title,
                      disabled: item.disabled,
                  })),
              )
            : options,
    );

    let displayLabel = $derived(
        (() => {
            if (value !== undefined && value !== null) {
                const found = allOptions.find((o) => {
                    if (o.value === value) return true;
                    if (o.isActive && typeof o.isActive === "function")
                        return o.isActive(value);
                    return false;
                });
                if (found) return found.label;
            }
            return placeholder;
        })(),
    );

    let isLongList = $derived(allOptions.length >= 10);
    let useModal = $derived(
        isMobile && (isLongList || forceModal || groups.length > 0),
    );

    let filteredOptions = $derived(
        searchQuery
            ? allOptions.filter((o) =>
                  o.label
                      .toLocaleLowerCase("tr-TR")
                      .includes(searchQuery.toLocaleLowerCase("tr-TR")),
              )
            : allOptions,
    );

    let filteredGroups = $derived(
        groups
            .map((g) => {
                const items = (g.options || g.links || []).filter((item) => {
                    if (!searchQuery) return true;
                    return item.label
                        .toLocaleLowerCase("tr-TR")
                        .includes(searchQuery.toLocaleLowerCase("tr-TR"));
                });
                return { ...g, items };
            })
            .filter((g) => g.items.length > 0),
    );

    // ── Global Scroll Lock ─────────────────────────────────────
    $effect(() => {
        if (isOpen && useModal) {
            lockScroll();
            return () => unlockScroll();
        }
    });

    $effect(() => {
        if (!isOpen) return;
        void searchQuery;
        hasKeyboardNavigated = false;
        if (searchQuery) {
            highlightedIndex = filteredOptions.length > 0 ? 0 : -1;
        }
    });

    // ── Lifecycle ──────────────────────────────────────────────
    function checkMobile() {
        if (typeof window !== "undefined" && typeof window.matchMedia === "function") {
            isMobile = window.matchMedia("(max-width: 600px)").matches;
        }
    }

    onMount(() => {
        checkMobile();
        window.addEventListener("resize", checkMobile);

        const onNavMenuOpen = () => {
            if (isOpen) close();
        };
        window.addEventListener("kepce:nav-menu-open", onNavMenuOpen);

        return () => {
            window.removeEventListener("resize", checkMobile);
            window.removeEventListener("kepce:nav-menu-open", onNavMenuOpen);
        };
    });

    // ── Portal ─────────────────────────────────────────────────
    function portal(node) {
        let parent = node.parentNode;
        let placeholder = document.createComment("portal");
        if (parent) parent.insertBefore(placeholder, node);
        document.body.appendChild(node);
        return {
            destroy() {
                if (placeholder.parentNode) {
                    placeholder.parentNode.insertBefore(node, placeholder);
                    placeholder.parentNode.removeChild(placeholder);
                } else if (node.parentNode) {
                    node.parentNode.removeChild(node);
                }
            },
        };
    }

    // ── Scroll helpers ─────────────────────────────────────────
    function scrollToHighlighted() {
        if (!listEl) return;
        const items = listEl.querySelectorAll(
            ".c-menu__item:not(.c-menu__item--action):not(.c-menu__item--special)",
        );
        const target = items[highlightedIndex];
        if (!target) return;
        isProgrammaticScroll = true;
        const listHeight = listEl.clientHeight;
        const itemTop = target.offsetTop;
        const itemHeight = target.offsetHeight;
        listEl.scrollTop = itemTop - listHeight / 2 + itemHeight / 2;
        setTimeout(() => {
            isProgrammaticScroll = false;
        }, 50);
    }

    // ── Close helpers ──────────────────────────────────────────
    function onOutsideClick(e) {
        if (!isOpen) return;
        if (Date.now() - openTime < 300) return;
        if (e.target && !document.contains(e.target)) return;
        if (triggerEl?.contains(e.target)) return;
        if (menuEl?.contains(e.target)) return;
        close();
    }

    function onOverlayClick(e) {
        e.stopPropagation();
        if (Date.now() - openTime < 300) return;
        close();
    }

    function onScrollClose(e) {
        if (isProgrammaticScroll || !isOpen || useModal) return;
        if (Date.now() - openTime < 300) return;
        const path = e.composedPath?.() || [];
        if (
            path.some(
                (el) => el === listEl || el === menuEl || el === overlayEl,
            )
        )
            return;
        const target = e.target;
        if (
            target !== window &&
            target !== document &&
            target !== document.documentElement &&
            target !== document.body
        ) {
            return;
        }
        close();
    }

    // ── Open / Close / Toggle ──────────────────────────────────
    async function open() {
        if (isOpen || disabled) return;
        checkMobile();
        if (activeDropdownClose && activeDropdownClose !== close)
            activeDropdownClose();
        activeDropdownClose = close;

        window.dispatchEvent(new CustomEvent("kepce:dropdown-open"));

        searchQuery = "";
        isOpen = true;
        openTime = Date.now();

        if (useModal) {
            sheetSnap = "default";
            if (!isIOSDevice()) {
                history.pushState({ kepceDropdown: true }, "");
                pushedState = true;
            }
            nativeBridge.sendOverlayToggle(true);
        }

        hasKeyboardNavigated = false;
        highlightedIndex = filteredOptions.findIndex(
            (o) =>
                o.value === value ||
                (o.isActive &&
                    typeof o.isActive === "function" &&
                    o.isActive(value)),
        );
        if (highlightedIndex === -1 && filteredOptions.length > 0)
            highlightedIndex = 0;

        await tick();
        scrollToHighlighted();

        if (isLongList && searchInputEl && !isMobile)
            searchInputEl.focus({ preventScroll: true });

        if (animation) animation.cancel();

        requestAnimationFrame(() => {
            if (!menuEl) return;

            // ORGANİK, NEFES ALAN ANİMASYON (OVERSHOOT YOK)
            // cubic-bezier(0.16, 1, 0.3, 1): Çok enerjik başlar, hedefe yaklaştıkça ipeksi yavaşlar.
            // Bu, sıçrama (bounce) yapmadan dinamik hissettiren tek native-benzeri eğridir.

            if (useModal) {
                menuEl.style.removeProperty("transform");
                menuEl.style.removeProperty("height");
                menuEl.style.removeProperty("transition");
                currentDragY = 0;

                animation = animate(
                    menuEl,
                    [
                        { opacity: 0, transform: "translateY(100%)" },
                        { opacity: 1, transform: "translateY(0)" },
                    ],
                    {
                        duration: getDuration(450),
                        easing: "cubic-bezier(0.16, 1, 0.3, 1)",
                    },
                );

                if (overlayAnimation) overlayAnimation.cancel();
                if (overlayEl) {
                    overlayAnimation = animate(
                        overlayEl,
                        [{ opacity: 0 }, { opacity: 1 }],
                        {
                            duration: getDuration(300),
                            easing: "ease-out",
                        },
                    );
                }
            } else {
                // Masaüstü (Popover) için balon gibi şişerek gelme
                const isUp = menuEl.dataset.openingDirection === "up";
                animation = animate(
                    menuEl,
                    [
                        {
                            opacity: 0,
                            transform: `scale(0.92) translateY(${isUp ? "12px" : "-12px"})`,
                        },
                        { opacity: 1, transform: "scale(1) translateY(0)" },
                    ],
                    {
                        duration: getDuration(350),
                        easing: "cubic-bezier(0.16, 1, 0.3, 1)",
                    },
                );
            }
        });
    }

    let isClosing = false;

    function close(fromPopState = false) {
        if (!isOpen || isClosing) return;
        isClosing = true;
        if (activeDropdownClose === close) activeDropdownClose = null;
        if (animation) animation.cancel();

        if (useModal) {
            if (!fromPopState && pushedState && history.state?.kepceDropdown) {
                history.back();
            }
            pushedState = false;
            nativeBridge.sendOverlayToggle(false);
        }

        // KAPANIŞ ANİMASYONLARI (Kullanıcıyı bekletmemek için daha hızlı)
        if (useModal && menuEl) {
            // Tam ekran veya varsayılan fark etmeksizin, kapanış sırasında
            // sınıf ya da history kaynaklı flicker yaşanmaması için fiziksel yüksekliği sabitle.
            const currentHeight = menuEl.offsetHeight;
            menuEl.style.height = `${currentHeight}px`;
            menuEl.style.maxHeight = "none";

            const startY = currentDragY;
            currentDragY = 0;

            menuEl.style.transform = `translateY(${startY}px)`;
            void menuEl.offsetHeight; // force reflow
            menuEl.style.transition = `transform ${getDuration(250)}ms cubic-bezier(0.3, 0, 0.8, 0.15)`;
            menuEl.style.transform = "translateY(100%)";

            if (overlayEl) {
                overlayEl.style.transition = `opacity ${getDuration(250)}ms linear`;
                overlayEl.style.opacity = "0";
            }

            setTimeout(() => {
                isOpen = false;
                isClosing = false;
                sheetSnap = "default";
                menuEl?.style.removeProperty("height");
                menuEl?.style.removeProperty("maxHeight");
            }, getDuration(250));
        } else if (menuEl) {
            const isUp = menuEl.dataset.openingDirection === "up";
            animation = animate(
                menuEl,
                [
                    { opacity: 1, transform: "scale(1) translateY(0)" },
                    {
                        opacity: 0,
                        transform: `scale(0.96) translateY(${isUp ? "6px" : "-6px"})`,
                    },
                ],
                {
                    duration: getDuration(200),
                    easing: "ease-in",
                },
            );

            if (animation) {
                animation.onfinish = () => {
                    isOpen = false;
                    isClosing = false;
                };
            } else {
                isOpen = false;
                isClosing = false;
            }
        } else {
            isOpen = false;
            isClosing = false;
        }
    }

    const EXPANDED_HEIGHT = "calc(100dvh - env(safe-area-inset-top, 16px))";
    const DEFAULT_HEIGHT = "65dvh";

    // ── Animation Helpers ──────────────────────────────────────
    function animateTransform(fromY, toY, duration = 200, onDone) {
        if (!menuEl) return;
        menuEl.style.transform = `translateY(${fromY}px)`;
        void menuEl.offsetHeight;
        menuEl.style.transition = `transform ${getDuration(duration)}ms cubic-bezier(0.25, 1, 0.35, 1)`;
        menuEl.style.transform = `translateY(${toY}px)`;

        let finished = false;
        const finish = () => {
            if (finished) return;
            finished = true;
            if (onDone) onDone();
        };
        menuEl.addEventListener("transitionend", finish, { once: true });
        setTimeout(finish, duration + 40);
    }

    // ── Sheet Snap Helpers ─────────────────────────────────────
    function snapTo(target, duration = 240) {
        if (!menuEl) return;

        if (target === "closed") {
            close();
            return;
        }

        const startHeight = menuEl.offsetHeight;

        // Hedef sınıfın CSS altındaki gerçek piksel yüksekliğini ölç:
        // Geçici inline stilleri kaldırıp hedef sınıfı anlık uygula
        menuEl.style.transition = "none";
        menuEl.style.removeProperty("height");
        sheetSnap = target;
        menuEl.classList.toggle("c-menu--expanded", target === "expanded");
        const naturalTargetHeight = menuEl.offsetHeight;

        // Eğer ölçülen yükseklik 0 veya geçersizse (test/headless ortamı), viewport hesabı ile fallback sağla
        const fallbackTargetHeight = target === "expanded"
            ? Math.round(window.innerHeight - 16)
            : Math.round(window.innerHeight * 0.65);
        const targetHeight = naturalTargetHeight > 0 ? naturalTargetHeight : fallbackTargetHeight;

        // Zaten hedef yükseklikteyse animasyonsuz bitir
        if (Math.abs(targetHeight - startHeight) < 2) {
            menuEl.style.removeProperty("height");
            menuEl.style.removeProperty("transition");
            return;
        }

        // Başlangıç yüksekliğini kilitle ve reflow zorla
        menuEl.style.height = `${startHeight}px`;
        void menuEl.offsetHeight;

        // Başlangıçtan hedefe tek, kesintisiz, pürüzsüz geçiş
        const animDuration = getDuration(duration);
        menuEl.style.transition = `height ${animDuration}ms cubic-bezier(0.25, 1, 0.35, 1)`;
        menuEl.style.height = `${targetHeight}px`;

        let finished = false;
        const finish = () => {
            if (finished || !menuEl) return;
            finished = true;
            menuEl.style.removeProperty("height");
            menuEl.style.removeProperty("transition");
        };
        menuEl.addEventListener("transitionend", finish, { once: true });
        setTimeout(finish, animDuration + 50);
    }

    function handleHandleClick(e) {
        e.stopPropagation();
        if (isDragging) return;
        snapTo(sheetSnap === "expanded" ? "default" : "expanded");
    }

    // ── Pointer Gestures (Touch & Mouse) ──────────────────────
    let activePointerId = null;

    function onSheetPointerStart(e) {
        if (!useModal || !isOpen || !menuEl) return;
        // Farede sadece sol tık ile sürüklemeye izin ver
        if (e.pointerType === "mouse" && e.button !== 0) return;

        const target = e.target;

        // Input ile etkileşime girerken sürükleme başlatma
        if (target.closest?.("input")) return;

        // Klavye açıkken (arama kutusu odaktayken) dokunulduğunda
        // sürükleme başlatmak yerine önce klavyeyi kapat.
        if (searchInputEl && document.activeElement === searchInputEl) {
            searchInputEl.blur();
            return;
        }

        const isHandle = target.closest?.(".c-menu__handle");
        const isSearch = target.closest?.(".c-menu__search");
        const isListArea = target.closest?.(".c-menu__scroll-area");

        if (isHandle || isSearch) {
            dragFromHandle = true;
        } else if (isListArea && listEl && listEl.scrollTop <= 0) {
            // Fare ile liste içinde gezinirken sürükleme tetiklemeyelim (seçim veya tekerlek scroll bozulmasın)
            if (e.pointerType === "mouse") return;
            dragFromHandle = false;
        } else {
            return;
        }

        activePointerId = e.pointerId;
        dragStartY = e.clientY;
        dragStartTime = Date.now();
        dragStartHeight = menuEl.offsetHeight;
        isDragging = false;
        dragMode = "none";
        menuEl.style.transition = "none";

        if (typeof menuEl.setPointerCapture === "function") {
            try {
                menuEl.setPointerCapture(e.pointerId);
            } catch {}
        }
    }

    function onSheetPointerMove(e) {
        if (!useModal || !menuEl || dragStartY === 0) return;
        if (activePointerId !== null && e.pointerId !== activePointerId) return;

        const deltaY = e.clientY - dragStartY;

        if (!dragFromHandle && listEl && listEl.scrollTop > 0) {
            return;
        }

        if (!dragFromHandle && deltaY < 0 && listEl) {
            return;
        }

        if (Math.abs(deltaY) > 5) {
            isDragging = true;
        }

        if (!isDragging) return;
        if (e.cancelable) e.preventDefault();

        const maxHeight = window.innerHeight - 16;
        const defaultHeight = Math.round(window.innerHeight * 0.65);

        if (sheetSnap === "default") {
            if (deltaY < 0) {
                // Dragging UP from default: expand height, bottom stays pinned at 0!
                dragMode = "height";
                const targetH = Math.min(dragStartHeight - deltaY, maxHeight);
                menuEl.style.height = `${targetH}px`;
                menuEl.style.transform = "translateY(0)";
                currentDragY = 0;
            } else {
                // Dragging DOWN from default: slide down towards close
                dragMode = "translate";
                menuEl.style.removeProperty("height");
                menuEl.style.transform = `translateY(${deltaY}px)`;
                currentDragY = deltaY;
            }
        } else if (sheetSnap === "expanded") {
            if (deltaY > 0) {
                // Dragging DOWN from expanded: shrink height, bottom stays pinned at 0!
                dragMode = "height";
                const targetH = Math.max(dragStartHeight - deltaY, defaultHeight * 0.7);
                menuEl.style.height = `${targetH}px`;
                menuEl.style.transform = "translateY(0)";
                currentDragY = 0;
            } else {
                // Dragging UP while already expanded: clamped
                dragMode = "none";
                menuEl.style.transform = "translateY(0)";
                currentDragY = 0;
            }
        }
    }

    function onSheetPointerEnd(e) {
        if (!useModal || !menuEl || dragStartY === 0) return;
        if (activePointerId !== null && e.pointerId !== activePointerId) return;

        const deltaY = e.clientY - dragStartY;
        const elapsed = Date.now() - dragStartTime;
        const velocity = Math.abs(deltaY) / Math.max(elapsed, 1);

        if (typeof menuEl.releasePointerCapture === "function") {
            try {
                if (menuEl.hasPointerCapture?.(e.pointerId)) {
                    menuEl.releasePointerCapture(e.pointerId);
                }
            } catch {}
        }

        activePointerId = null;
        dragStartY = 0;
        const wasDragging = isDragging;

        // isDragging bayrağını peşinden gelen click olayını yutana kadar koru
        setTimeout(() => {
            isDragging = false;
        }, 50);

        if (!wasDragging) {
            currentDragY = 0;
            return;
        }

        const isFastSwipe = velocity > 0.35;

        if (dragMode === "translate") {
            // User dragged DOWN in default mode
            if (deltaY > 80 || (deltaY > 30 && isFastSwipe)) {
                // Smooth close from current dragged translateY!
                close();
            } else {
                // Bounce back up from current translateY!
                animateTransform(deltaY, 0, 200, () => {
                    menuEl?.style.removeProperty("transform");
                    menuEl?.style.removeProperty("transition");
                    currentDragY = 0;
                });
            }
        } else if (dragMode === "height") {
            if (sheetSnap === "default") {
                // Dragged UP towards expanded
                if (-deltaY > 40 || (-deltaY > 15 && isFastSwipe)) {
                    snapTo("expanded", 220);
                } else {
                    snapTo("default", 200);
                }
            } else if (sheetSnap === "expanded") {
                // Dragged DOWN towards default
                if (deltaY > 50 || (deltaY > 20 && isFastSwipe)) {
                    snapTo("default", 220);
                } else {
                    snapTo("expanded", 200);
                }
            }
        }
    }

    function handlePopState(e) {
        if (isClosing || !isOpen) return;
        if (useModal && pushedState) {
            if (Date.now() - openTime < 300) return;
            if (sheetSnap === "expanded") {
                snapTo("default");
                if (!isIOSDevice()) {
                    history.pushState({ kepceDropdown: true }, "");
                }
                return;
            }
            close(true);
        }
    }

    function toggle(e) {
        if (e) e.stopPropagation();
        if (isOpen) {
            if (Date.now() - openTime < 300) return;
            close();
        } else {
            open();
        }
    }

    // ── Selection ──────────────────────────────────────────────
    function selectOption(e, opt) {
        e.stopPropagation();
        if (opt.disabled) return;
        const optVal = opt.value ?? opt.href;
        value = optVal;
        if (onChange) onChange(optVal, opt);
        close();
    }

    function handleActionClick(e) {
        e.stopPropagation();
        close();
        if (onActionClick) onActionClick();
    }

    function handleSpecialClick(e) {
        e.stopPropagation();
        close();
        if (onSpecialClick) onSpecialClick();
    }

    // ── Keyboard ───────────────────────────────────────────────
    function handleKeyDown(e) {
        if (disabled) return;
        const key = e.key;
        const maxIndex = filteredOptions.length - 1;

        if (!isOpen) {
            if (key === "ArrowDown" || key === "ArrowUp") {
                e.preventDefault();
                const diff = key === "ArrowDown" ? 1 : -1;
                let nextIndex =
                    filteredOptions.findIndex((o) => o.value === value) + diff;
                while (
                    nextIndex >= 0 &&
                    nextIndex <= maxIndex &&
                    filteredOptions[nextIndex].disabled
                )
                    nextIndex += diff;
                if (nextIndex >= 0 && nextIndex <= maxIndex) {
                    value = filteredOptions[nextIndex].value;
                    if (onChange) onChange(value, filteredOptions[nextIndex]);
                }
            } else if (key === "Enter" || key === " ") {
                e.preventDefault();
                open();
            }
            return;
        }

        if (key === "Escape") {
            e.preventDefault();
            close();
            triggerEl?.focus();
            return;
        }

        if (key === "Tab") {
            close();
            return;
        }

        if (key === "ArrowDown") {
            e.preventDefault();
            hasKeyboardNavigated = true;
            let next = highlightedIndex + 1;
            while (next <= maxIndex && filteredOptions[next].disabled) next++;
            if (next <= maxIndex) {
                highlightedIndex = next;
                scrollToHighlighted();
            }
            return;
        }

        if (key === "ArrowUp") {
            e.preventDefault();
            hasKeyboardNavigated = true;
            let prev = highlightedIndex - 1;
            while (prev >= 0 && filteredOptions[prev].disabled) prev--;
            if (prev >= 0) {
                highlightedIndex = prev;
                scrollToHighlighted();
            }
            return;
        }

        if (key === "Home") {
            e.preventDefault();
            hasKeyboardNavigated = true;
            highlightedIndex = filteredOptions.findIndex((o) => !o.disabled);
            scrollToHighlighted();
            return;
        }

        if (key === "End") {
            e.preventDefault();
            hasKeyboardNavigated = true;
            for (let i = maxIndex; i >= 0; i--) {
                if (!filteredOptions[i].disabled) {
                    highlightedIndex = i;
                    break;
                }
            }
            scrollToHighlighted();
            return;
        }

        if (key === "Enter") {
            e.preventDefault();
            // Arama kutusundayken (özellikle mobil sanal klavyede "Ara / Git" tuşuna basıldığında),
            // kullanıcı ok tuşlarıyla liste içinde gezinmediyse seçimi tetikleyip modalı kapatma;
            // sadece klavyeyi kapat (blur).
            if (document.activeElement === searchInputEl && !hasKeyboardNavigated) {
                searchInputEl?.blur();
                return;
            }
            if (highlightedIndex >= 0 && highlightedIndex <= maxIndex) {
                const opt = filteredOptions[highlightedIndex];
                if (opt && !opt.disabled) selectOption(e, opt);
            }
            return;
        }

        // Type-ahead
        if (
            key.length === 1 &&
            !e.ctrlKey &&
            !e.altKey &&
            !e.metaKey &&
            document.activeElement !== searchInputEl
        ) {
            clearTimeout(searchTimeout);
            searchBuffer += key.toLocaleLowerCase("tr-TR");
            searchTimeout = setTimeout(() => {
                searchBuffer = "";
            }, 500);

            const matchIndex = filteredOptions.findIndex(
                (o) =>
                    !o.disabled &&
                    o.label.toLocaleLowerCase("tr-TR").startsWith(searchBuffer),
            );
            if (matchIndex !== -1) {
                highlightedIndex = matchIndex;
                scrollToHighlighted();
            }
        }
    }

    let triggerAriaLabel = $derived(
        ariaLabel ||
            (displayLabel
                ? `${placeholder}: ${displayLabel}`
                : `${placeholder} seçiniz`),
    );
    let menuId = "c-menu-" + Math.random().toString(36).slice(2, 8);
</script>

<svelte:window
    onclick={onOutsideClick}
    onscrollcapture={onScrollClose}
    onpopstate={handlePopState}
/>

<div
    {id}
    class="dropdown dropdown--{variant}"
    class:dropdown--open={isOpen}
    class:dropdown--disabled={disabled}
>
    <button
        bind:this={triggerEl}
        class="dropdown__trigger dropdown__trigger--{variant}"
        class:dropdown__trigger--open={isOpen}
        class:dropdown__trigger--disabled={disabled}
        class:dropdown__trigger--has-value={value !== undefined &&
            value !== null &&
            value !== ""}
        class:dropdown__trigger--placeholder={value === undefined ||
            value === null ||
            value === ""}
        type="button"
        role="combobox"
        {disabled}
        aria-disabled={disabled}
        aria-haspopup="listbox"
        aria-expanded={isOpen}
        aria-controls={isOpen ? menuId : undefined}
        aria-label={triggerAriaLabel}
        onclick={toggle}
        onkeydown={handleKeyDown}
    >
        <span class="dropdown__label">{displayLabel}</span>
        <div class="dropdown__chevron" aria-hidden="true">
            {@html icon("chevronDown")}
        </div>
    </button>
</div>

<!-- ─── Menu Panel ──────────────────────────────────────────── -->
{#if isOpen}
    {#if useModal}
        <div class="u-hidden">
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
                bind:this={overlayEl}
                class="c-menu__overlay c-menu__overlay--open"
                use:portal
                onclick={onOverlayClick}
                role="presentation"
            ></div>
        </div>
    {/if}

    <div class="u-hidden">
        <div
            bind:this={menuEl}
            id={menuId}
            class="c-menu c-menu--open"
            class:c-menu--modal={useModal}
            class:c-menu--expanded={sheetSnap === "expanded"}
            role="listbox"
            tabindex="-1"
            use:portal
            use:popover={{ triggerEl, align: "left", disabled: useModal }}
            onpointerdown={onSheetPointerStart}
            onpointermove={onSheetPointerMove}
            onpointerup={onSheetPointerEnd}
            onpointercancel={onSheetPointerEnd}
        >
            {#if useModal}
                <button
                    type="button"
                    class="c-menu__handle"
                    aria-label="Menü boyutu kontrolü"
                    onclick={handleHandleClick}
                >
                    <span class="c-menu__handle-bar"></span>
                </button>
            {/if}
            {#if isLongList}
                <div class="c-menu__search">
                    <input
                        bind:this={searchInputEl}
                        type="text"
                        placeholder="Ara..."
                        bind:value={searchQuery}
                        onkeydown={handleKeyDown}
                    />
                </div>
            {/if}

            <div bind:this={listEl} class="c-menu__scroll-area">
                {#if actionItem}
                    <button
                        class="c-menu__item c-menu__item--accent c-menu__item--action"
                        type="button"
                        role="option"
                        aria-selected="false"
                        onclick={handleActionClick}
                    >
                        <span class="c-menu__item-label"
                            >{actionItem.label}</span
                        >
                        {#if actionItem.icon}
                            <span class="c-menu__item-icon"
                                >{@html icon(actionItem.icon, 16)}</span
                            >
                        {/if}
                    </button>
                {/if}

                {#if groups.length > 0}
                    {#each filteredGroups as group}
                        <div class="c-menu__section">
                            <div class="c-menu__section-title">
                                {group.title}
                            </div>
                            {#each group.items as opt}
                                {@const optVal = opt.value ?? opt.href}
                                {@const isSel =
                                    optVal === value ||
                                    (opt.isActive &&
                                        typeof opt.isActive === "function" &&
                                        opt.isActive(value))}
                                <button
                                    class="c-menu__item"
                                    class:c-menu__item--selected={isSel}
                                    class:c-menu__item--disabled={opt.disabled}
                                    type="button"
                                    role="option"
                                    aria-selected={isSel}
                                    disabled={opt.disabled}
                                    onclick={(e) =>
                                        selectOption(e, {
                                            ...opt,
                                            value: optVal,
                                        })}
                                >
                                    <span class="c-menu__item-label"
                                        >{opt.label}</span
                                    >
                                    {#if isSel}
                                        <span class="c-menu__item-check"
                                            >{@html icon("check", 16)}</span
                                        >
                                    {/if}
                                </button>
                            {/each}
                        </div>
                    {/each}
                {:else}
                    {#each filteredOptions as o, index}
                        <button
                            class="c-menu__item"
                            class:c-menu__item--selected={o.value === value}
                            class:c-menu__item--disabled={o.disabled}
                            class:c-menu__item--highlighted={index ===
                                highlightedIndex}
                            type="button"
                            role="option"
                            aria-selected={o.value === value}
                            disabled={o.disabled}
                            onclick={(e) => selectOption(e, o)}
                        >
                            <span class="c-menu__item-label">{o.label}</span>
                            {#if o.value === value}
                                <span class="c-menu__item-check"
                                    >{@html icon("check", 16)}</span
                                >
                            {/if}
                        </button>
                    {/each}
                {/if}

                {#if groups.length > 0 ? filteredGroups.length === 0 : filteredOptions.length === 0}
                    <div
                        class="c-menu__item c-menu__item--disabled c-menu__empty-state"
                    >
                        Sonuç bulunamadı
                    </div>
                {/if}
            </div>

            {#if specialItem}
                <button
                    class="c-menu__item c-menu__item--special"
                    type="button"
                    role="option"
                    aria-selected="false"
                    onclick={handleSpecialClick}
                >
                    <span class="c-menu__item-label">{specialItem.label}</span>
                </button>
            {/if}
        </div>
    </div>
{/if}
