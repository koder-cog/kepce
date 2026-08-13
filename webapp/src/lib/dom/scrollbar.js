/**
 * Kepçe — Custom Scrollbar Logic
 * Handles a bespoke scrollbar that respects header/footer boundaries.
 */
import { isMotionEnabled } from './motion.js';

export function initScrollbar() {
  // Do not initialize on touch devices to preserve native scroll momentum and performance
  if (window.matchMedia('(hover: none)').matches) {
    return { update: () => {}, observer: null, bar: null };
  }

  const bar = document.createElement('div');
  bar.className = 'c-scrollbar';
  
  const thumb = document.createElement('div');
  thumb.className = 'c-scrollbar__thumb';
  
  bar.appendChild(thumb);
  document.body.appendChild(bar);

  let isDragging = false;
  let startY = 0;
  let startScrollTop = 0;

  let scrollTimeout = null;
  let ticking = false;
  function update() {
    if (!ticking) {
      window.requestAnimationFrame(() => {
        actuallyUpdate();
        ticking = false;
      });
      ticking = true;
    }
  }

  function actuallyUpdate() {
    const scrollHeight = document.documentElement.scrollHeight;
    const clientHeight = window.innerHeight;
    const scrollTop = window.scrollY;
    
    // Hide if content fits in viewport
    if (scrollHeight <= clientHeight) {
      bar.classList.add('c-scrollbar--hidden');
      return;
    } else {
      bar.classList.remove('c-scrollbar--hidden');
    }

    // Auto-hide logic: Add .is-scrolling class and remove after delay
    document.body.classList.add('is-scrolling');
    clearTimeout(scrollTimeout);
    if (!isDragging) {
      scrollTimeout = setTimeout(() => {
        document.body.classList.remove('is-scrolling');
      }, 2000);
    }

    const navBar = document.getElementById('main-nav');
    const navOffset = navBar ? navBar.clientHeight : 0;
    
    // Mobil görünümde (genişlik <= 600) .ci-container fixed olduğu için bottom offset ekle
    let bottomOffset = 0;
    if (window.innerWidth <= 600) {
      const ciContainer = document.querySelector('.ci-container');
      if (ciContainer) {
        bottomOffset = ciContainer.clientHeight;
      }
    }

    // Container spans end-to-end (except for navbar and bottom bars)
    bar.style.top = `${navOffset}px`;
    bar.style.bottom = `${bottomOffset}px`;

    const margin = 8; // Safety margin for the thumb only
    const barHeight = bar.clientHeight - (margin * 2);
    const scrollPercentage = scrollTop / (scrollHeight - clientHeight);
    
    // Thumb height proportional to content
    const thumbHeight = Math.max(40, (clientHeight / scrollHeight) * barHeight);
    const maxThumbTop = barHeight - thumbHeight;
    const thumbTop = margin + (scrollPercentage * maxThumbTop);

    thumb.style.height = `${thumbHeight}px`;
    thumb.style.transform = `translateY(${thumbTop}px)`;
  }

  // Handle Drag
  thumb.addEventListener('mousedown', (e) => {
    isDragging = true;
    startY = e.clientY;
    startScrollTop = window.scrollY;
    bar.classList.add('is-active');
    document.body.classList.add('is-scrolling'); // Keep visible while dragging
    document.body.classList.add('is-scrolling-custom');
    e.preventDefault();
  });

  document.addEventListener('mousemove', (e) => {
    if (!isDragging) return;
    
    const deltaY = e.clientY - startY;
    const scrollHeight = document.documentElement.scrollHeight;
    const clientHeight = window.innerHeight;
    const barHeight = bar.clientHeight;
    const thumbHeight = parseFloat(thumb.style.height);
    
    const scrollableRange = scrollHeight - clientHeight;
    const barRange = barHeight - thumbHeight;
    
    const scrollDelta = (deltaY / barRange) * scrollableRange;
    window.scrollTo(0, startScrollTop + scrollDelta);
  });

  document.addEventListener('mouseup', () => {
    if (isDragging) {
      isDragging = false;
      bar.classList.remove('is-active');
      document.body.classList.remove('is-scrolling-custom');
      
      // Start hide timer after drag ends
      clearTimeout(scrollTimeout);
      scrollTimeout = setTimeout(() => {
        document.body.classList.remove('is-scrolling');
      }, 2000);
    }
  });

  // Track click to jump
  bar.addEventListener('mousedown', (e) => {
    if (e.target === thumb) return;
    const rect = bar.getBoundingClientRect();
    const clickY = e.clientY - rect.top;
    const thumbHeight = parseFloat(thumb.style.height);
    const scrollHeight = document.documentElement.scrollHeight;
    const clientHeight = window.innerHeight;
    
    const scrollPercentage = (clickY - thumbHeight / 2) / (bar.clientHeight - thumbHeight);
    const isEnabled = isMotionEnabled();
    window.scrollTo({
      top: scrollPercentage * (scrollHeight - clientHeight),
      behavior: isEnabled ? 'smooth' : 'auto'
    });
  });

  window.addEventListener('scroll', update, { passive: true });
  window.addEventListener('resize', update, { passive: true });
  
  // Observer for dynamic content height changes
  const observer = new MutationObserver(update);
  
  // Resize observer to track navbar height dynamically (especially during banner dismissal animations)
  const resizeObserver = new ResizeObserver(update);

  // Defer initial DOM measurement and observer to avoid "Layout was forced before page fully loaded"
  function attachObserver() {
    observer.observe(document.body, { childList: true, subtree: true });
    
    const navBar = document.getElementById('main-nav');
    if (navBar) {
      resizeObserver.observe(navBar);
    }
    
    update();
  }

  if (document.readyState === 'complete') {
    attachObserver();
  } else {
    window.addEventListener('load', attachObserver, { once: true });
  }

  // Handle permanent visibility setting
  function applyPermanentState() {
    const isPermanent = localStorage.getItem('kepce_scrollbar_permanent') === 'true';
    bar.classList.toggle('is-permanent', isPermanent);
  }

  window.addEventListener('scrollbar-setting-changed', applyPermanentState);
  applyPermanentState();

  // Return update function for manual triggers if needed
  return { update, observer, bar };
}
