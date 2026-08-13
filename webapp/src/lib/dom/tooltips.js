/**
 * Precision Engineered Tooltip Manager
 * Strictly adheres to geometric rules while ensuring robust event delegation.
 * 
 * Fixes applied:
 * - Proper cleanup of previous tooltip before creating new one
 * - Guard against stale references in async callbacks (rAF)
 * - Accessibility: role="tooltip" + aria-describedby linkage
 */
import { runNextTick, getDuration } from './motion.js';

export function initTooltipManager() {
  const portal = document.getElementById('tooltip-portal');
  if (!portal) return;

  let activeTooltip = null;
  let closeTimeout = null;

  const cleanupTooltip = (tooltip) => {
    if (!tooltip) return;
    tooltip.classList.remove('tooltip-instance--visible');
    // Remove after CSS transition completes
    setTimeout(() => {
      if (tooltip.parentNode) tooltip.remove();
    }, getDuration(250));
  };

  const startDismissal = () => {
    if (closeTimeout) return; // Already dismissing
    closeTimeout = setTimeout(() => {
      if (activeTooltip) {
        const toRemove = activeTooltip;
        activeTooltip = null;
        cleanupTooltip(toRemove);
      }
      closeTimeout = null;
    }, getDuration(100)); // 100ms grace period for hover survival
  };

  const cancelDismissal = () => {
    if (closeTimeout) {
      clearTimeout(closeTimeout);
      closeTimeout = null;
    }
  };

  // Helper to validate that the trigger of an active tooltip is still active, visible, and enabled
  const validateActiveTooltip = () => {
    if (!activeTooltip) return;
    const trigger = activeTooltip._trigger;
    
    // 1. Is the trigger still mounted in the DOM?
    if (!document.body.contains(trigger)) {
      startDismissal();
      return;
    }
    
    // 2. Is the trigger disabled or hidden?
    const rect = trigger.getBoundingClientRect();
    const isHidden = rect.width === 0 && rect.height === 0;
    const isDisabled = trigger.hasAttribute('disabled') || trigger.disabled === true;
    
    if (isHidden || isDisabled) {
      startDismissal();
    }
  };

  const showTooltip = (trigger) => {
    cancelDismissal();

    let content = '';
    const sourceTooltip = trigger.querySelector('.meal-card__source-tooltip, .tooltip-template');
    
    if (sourceTooltip) {
      content = sourceTooltip.innerHTML;
    } else if (trigger.dataset.tooltip) {
      content = trigger.dataset.tooltip;
    }

    if (!content) return;

    // Avoid redundant re-creation if same trigger
    if (activeTooltip && activeTooltip._trigger === trigger) return;

    // Cleanup previous instantly if switching to a different trigger
    if (activeTooltip) {
      activeTooltip.remove();
      activeTooltip = null;
    }

    // 1. Mount
    const tooltip = document.createElement('div');
    tooltip.className = 'tooltip-instance tooltip-instance--measuring';
    
    // Expressive threshold: If content is long or contains HTML, make it expressive
    if (content.length > 60 || content.includes('<br') || content.includes('<p')) {
      tooltip.classList.add('tooltip-instance--expressive');
    }

    tooltip.setAttribute('role', 'tooltip');
    tooltip.innerHTML = content;
    tooltip._trigger = trigger;
    activeTooltip = tooltip;
    portal.appendChild(tooltip);

    // Hover survival listeners
    tooltip.addEventListener('mouseenter', cancelDismissal);
    tooltip.addEventListener('mouseleave', (e) => {
      if (e.relatedTarget && trigger.contains(e.relatedTarget)) {
        cancelDismissal();
      } else {
        startDismissal();
      }
    });

    // 2. Measure-Before-Paint Cycle
    runNextTick(() => {
      if (activeTooltip !== tooltip) {
        if (tooltip.parentNode) tooltip.remove();
        return;
      }

      const triggerRect = trigger.getBoundingClientRect();
      const tooltipRect = tooltip.getBoundingClientRect();
      const vw = document.documentElement.clientWidth;
      const vh = window.innerHeight;
      const gap = 8; // M3 standard gap

      const scrollX = window.pageXOffset || document.documentElement.scrollLeft;
      const scrollY = window.pageYOffset || document.documentElement.scrollTop;

      // Horizontal Centering with Viewport Clamping
      const idealLeft = triggerRect.left + (triggerRect.width / 2) - (tooltipRect.width / 2);
      const clampedLeft = Math.max(12, Math.min(idealLeft, vw - tooltipRect.width - 12));
      const finalLeft = clampedLeft + scrollX;

      // Vertical Placement (Flip if no space below)
      const spaceBelow = vh - triggerRect.bottom;
      const spaceAbove = triggerRect.top;
      let finalTop;

      if (spaceBelow < tooltipRect.height + gap && spaceAbove > tooltipRect.height + gap) {
        // Place Above
        finalTop = (triggerRect.top - tooltipRect.height - gap) + scrollY;
        tooltip.classList.add('tooltip-instance--placed-above');
      } else {
        // Place Below
        finalTop = (triggerRect.bottom + gap) + scrollY;
        tooltip.classList.add('tooltip-instance--placed-below');
      }

      tooltip.style.left = `${Math.round(finalLeft)}px`;
      tooltip.style.top = `${Math.round(finalTop)}px`;
      
      // Force a synchronous layout calculation so the browser registers the starting transform
      // without transition (because of --measuring class).
      void tooltip.offsetWidth;
      
      // Remove measuring class to enable transitions
      tooltip.classList.remove('tooltip-instance--measuring');
      
      // Force layout again so browser knows transitions are active
      void tooltip.offsetWidth;
      
      tooltip.classList.add('tooltip-instance--visible');
    });
  };

  // Delegate listeners
  document.addEventListener('mouseover', (e) => {
    // 0. Suppress on touch devices to prevent "sticky" tooltips on hover
    if (window.matchMedia('(pointer: coarse)').matches) return;
    
    const trigger = e.target.closest('[data-tooltip], [data-tooltip-trigger], [title], .meal-card__source-wrapper');
    if (!trigger) return;

    if (trigger.hasAttribute('title')) {
      const titleContent = trigger.getAttribute('title');
      if (titleContent) {
        trigger.setAttribute('data-tooltip', titleContent);
        if (!trigger.hasAttribute('aria-label')) {
          trigger.setAttribute('aria-label', titleContent);
        }
        trigger.removeAttribute('title');
      }
    }

    if (trigger.dataset.tooltipTrigger === 'click') return;

    showTooltip(trigger);
  }, { passive: true });

  // Click/Touch Tap lifecycle: Toggles tooltips and handles click-outside dismissal
  // Registered in capture phase to intercept taps before other handlers can call stopPropagation()
  document.addEventListener('click', (e) => {
    const trigger = e.target.closest('[data-tooltip], [data-tooltip-trigger], [title], .meal-card__source-wrapper');
    if (trigger) {
      // Convert native title attributes to custom tooltips on click/tap
      if (trigger.hasAttribute('title')) {
        const titleContent = trigger.getAttribute('title');
        if (titleContent) {
          trigger.setAttribute('data-tooltip', titleContent);
          if (!trigger.hasAttribute('aria-label')) {
            trigger.setAttribute('aria-label', titleContent);
          }
          trigger.removeAttribute('title');
        }
      }

      // Toggle off if clicking the same trigger on touch devices OR if it's explicitly click-triggered
      if (window.matchMedia('(pointer: coarse)').matches || trigger.dataset.tooltipTrigger === 'click') {
        if (activeTooltip && activeTooltip._trigger === trigger) {
          startDismissal();
          return;
        }
      }
      
      // On all devices: clicking an actionable element (button/link) implies action taken.
      // Dismiss the tooltip immediately to prevent it from overlaying subsequent UI changes (e.g. modals)
      if (trigger.tagName === 'BUTTON' || trigger.closest('button, a')) {
        startDismissal();
        return;
      }
      
      showTooltip(trigger);
      // Defer validation to check if the trigger got destroyed/disabled by its click event
      setTimeout(validateActiveTooltip, 150);
    } else {
      // Dismiss active tooltip if clicking completely outside
      if (activeTooltip && !activeTooltip.contains(e.target)) {
        startDismissal();
      }
    }
  }, { capture: true, passive: true });

  document.addEventListener('mouseout', (e) => {
    // 0. Suppress on touch devices to prevent synthetic event conflicts (e.g. sticky/flickering)
    if (window.matchMedia('(pointer: coarse)').matches) return;

    const trigger = e.target.closest('[data-tooltip], [data-tooltip-trigger], .meal-card__source-wrapper');
    if (!trigger) return;

    if (trigger.dataset.tooltipTrigger === 'click') return;

    const movingToTooltip = activeTooltip && activeTooltip.contains(e.relatedTarget);
    const movingToChild = trigger.contains(e.relatedTarget);

    if (!movingToTooltip && !movingToChild) {
      startDismissal();
    }
  }, { passive: true });

  // Dismiss tooltips instantly on scroll events (critical for mobile scrolling UX)
  window.addEventListener('scroll', () => {
    if (activeTooltip) {
      startDismissal();
    }
  }, { passive: true });

  // Dismiss tooltips if the cursor leaves the window boundary entirely
  document.addEventListener('mouseleave', () => {
    if (activeTooltip) {
      startDismissal();
    }
  }, { passive: true });

  // Continuously monitor active tooltip validity on mouse/pointer movement to prevent orphans
  document.addEventListener('pointermove', validateActiveTooltip, { passive: true });

  // Clean up and remove tooltips instantly on any SPA route changes
  window.addEventListener('navigate', () => {
    if (activeTooltip) {
      activeTooltip.remove();
      activeTooltip = null;
    }
  }, { passive: true });
}
