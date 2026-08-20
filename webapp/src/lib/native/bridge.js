/**
 * Kepçe Mobil & Masaüstü Çift Yönlü Yerel Köprü (Native Bridge)
 * Hem Android (JavascriptInterface) hem de iOS (WKScriptMessageHandler) ile uyumludur.
 */

export const nativeBridge = {
  /**
   * Yerel kabuğa tipli JSON mesajı gönderir.
   * @param {string} type - Olay tipi (Örn: "STATE_CHANGED", "ROUTE_CHANGED", "OVERLAY_TOGGLED", "SET_TITLE", "HAPTIC", "SHARE")
   * @param {Record<string, any>} payload - Veri nesnesi
   */
  send(type, payload = {}) {
    if (typeof window === 'undefined') return;

    const message = { type, payload };

    // 1. Android Köprüsü: window.KepceBridge.postMessage(...)
    if (window.KepceBridge && typeof window.KepceBridge.postMessage === 'function') {
      try {
        window.KepceBridge.postMessage(JSON.stringify(message));
        return;
      } catch (e) {
        console.warn('[Bridge] Android postMessage hatası:', e);
      }
    }

    // 2. iOS Köprüsü: window.webkit.messageHandlers.KepceBridge.postMessage(...)
    if (window.webkit?.messageHandlers?.KepceBridge?.postMessage) {
      try {
        window.webkit.messageHandlers.KepceBridge.postMessage(message);
        return;
      } catch (e) {
        console.warn('[Bridge] iOS postMessage hatası:', e);
      }
    }
  },

  /**
   * Tema, görsel efekt ve oturum durumunu yerel kabukla eşitler.
   */
  sendState({ isDark, effectsEnabled, bgColorHex, navBgOpacity = 0.8, colorSurface, colorBorder, unreadCount = 0 }) {
    this.send('STATE_CHANGED', {
      isDark,
      effectsEnabled,
      bgColorHex,
      navBgOpacity,
      colorSurface,
      colorBorder,
      unreadCount
    });
  },

  /**
   * Sayfa rotası, başlığı, geri dönülebilirlik ve tab-bar gizleme durumunu yerel kabuğa bildirir.
   */
  sendRoute({ path, title, canGoBack, isRoot = false, hideBottomNav = false }) {
    this.send('ROUTE_CHANGED', {
      path,
      title,
      canGoBack,
      isRoot,
      hideBottomNav
    });
  },

  /**
   * Dinamik sayfa başlığı güncellemesi (Örn: Menü veya Profil detayında).
   */
  sendTitle(title) {
    this.send('SET_TITLE', { title });
  },

  /**
   * Modal, açılır menü veya yorum kutusu açıldığında alt Tab-Bar'ı gizler/gösterir.
   */
  sendOverlayToggle(isOpen) {
    this.send('OVERLAY_TOGGLED', { isOpen });
  },

  /**
   * Sistem dokunsal geri bildirimi (haptic feedback) tetikler.
   * @param {"click" | "light" | "medium" | "heavy" | "success" | "error"} style
   */
  triggerHaptic(style = 'light') {
    this.send('HAPTIC', { style });
  },

  /**
   * Sistem paylaşım menüsünü açar.
   */
  share({ title, text, url }) {
    this.send('SHARE', { title, text, url });
  }
};
