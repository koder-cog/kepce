/**
 * Global Dropdown Enhancer
 * Automatically finds native <select> elements and upgrades them to custom Kepçe dropdowns
 */
import { mount } from 'svelte';
import Dropdown from '../../components/features/Dropdown.svelte';

export function enhanceSelects(root = document) {
  // Find all select elements that haven't been enhanced yet
  const selects = root.querySelectorAll('select:not(.enhanced)');

  selects.forEach(select => {
    // Skip if it's explicitly marked to stay native
    if (select.dataset.native === 'true') return;

    // 1. Extract options from native select
    const options = Array.from(select.options).map(opt => ({
      value: opt.value,
      label: opt.text,
      disabled: opt.disabled
    }));

    const isSecondary = select.classList.contains('select-secondary') ||
      select.classList.contains('select--secondary') ||
      select.dataset.variant === 'secondary';

    // 2. Create a wrapper for the custom dropdown
    const wrapper = document.createElement('div');
    wrapper.className = 'select-enhancer-wrapper';
    if (isSecondary) {
      wrapper.classList.add('select-enhancer-wrapper--secondary');
    }
    // Match the original width if needed
    wrapper.style.width = select.style.width || (isSecondary ? 'max-content' : '100%');
    if (select.classList.contains('btn--sm') || select.classList.contains('form-input--sm')) {
      wrapper.classList.add('select-enhancer-wrapper--sm');
    }

    // 3. Insert wrapper before select and hide original select
    select.parentNode.insertBefore(wrapper, select);
    select.style.display = 'none';
    select.classList.add('enhanced');

    // 4. Initialize our custom dropdown
    mount(Dropdown, {
      target: wrapper,
      props: {
        options,
        value: select.value,
        placeholder: select.getAttribute('placeholder') || 'Seçiniz',
        disabled: select.disabled,
        variant: isSecondary ? 'secondary' : 'primary',
        onChange: (value) => {
          select.value = value;
          // Trigger native change event so any listeners on the original select still work
          select.dispatchEvent(new Event('change', { bubbles: true }));
        }
      }
    });

    // 5. Watch for programmatic changes on the original select
    const observer = new MutationObserver(() => {
      // This is a bit complex for a generic enhancer, 
      // but for now we trust the one-way sync.
    });
    observer.observe(select, { attributes: true });
  });
}
