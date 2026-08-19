(function() {
  window.applyTheme = function(themeStr, isInit = false) {
    const el = document.documentElement;
    const updateDOM = () => {
      if (themeStr === 'koyu') {
        el.classList.add('dark');
        el.classList.remove('light');
      } else if (themeStr === 'acik') {
        el.classList.add('light');
        el.classList.remove('dark');
      } else {
        el.classList.remove('dark', 'light');
      }
    };

    if (!isInit && typeof document.startViewTransition === 'function') {
      try {
        const transition = document.startViewTransition(updateDOM);
        if (transition) {
          if (transition.ready) transition.ready.catch(() => {});
          if (transition.finished) transition.finished.catch(() => {});
        }
      } catch (_) {
        updateDOM();
      }
    } else {
      if (!isInit) {
        el.classList.add('theme-transitioning');
        // Double requestAnimationFrame ensures transition class is fully registered by Firefox/Safari
        requestAnimationFrame(() => {
          requestAnimationFrame(() => {
            updateDOM();
            window.setTimeout(() => {
              el.classList.remove('theme-transitioning');
            }, 500);
          });
        });
      } else {
        updateDOM();
      }
    }
  };

  const syncThemeState = function() {
    const savedTheme = localStorage.getItem('renkTercihi') || 'sistem';
    window.applyTheme(savedTheme, true);
    if (localStorage.getItem('kepce_show_bot') === 'false') {
      document.documentElement.classList.add('hide-ai');
    } else {
      document.documentElement.classList.remove('hide-ai');
    }
  };

  syncThemeState();

  window.addEventListener('popstate', syncThemeState);
  window.addEventListener('pageshow', syncThemeState);
})();
