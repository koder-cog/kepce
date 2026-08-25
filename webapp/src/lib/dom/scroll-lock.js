/**
 * Merkezi scroll kilidi - modal ve bottom-sheet gibi overlay bindiren
 * tüm bileşenler arka plan kaydırmasını bu util üzerinden kilitler.
 *
 * Sayaç tabanlıdır: üst üste binen overlay'lerde (modal içinde açılan
 * dropdown bottom-sheet'i gibi) son overlay kapanana kadar kilit sürer.
 * SSR güvenlidir (document yoksa no-op).
 */
let lockCount = 0;

export function lockScroll() {
    if (typeof document === "undefined") return;
    lockCount += 1;
    document.documentElement.classList.add("overlay-open");
}

export function unlockScroll() {
    if (typeof document === "undefined") return;
    lockCount = Math.max(0, lockCount - 1);
    if (lockCount === 0) {
        document.documentElement.classList.remove("overlay-open");
    }
}
