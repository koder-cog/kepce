/**
 * Kepçe UI Form Components Factory
 * Standardized components for the Benmari system (Libadwaita + MD3 Expressive).
 */
import { icon } from './icons.js';

/**
 * Creates a standard boxed list container.
 * @param {string} content - HTML string of list rows
 * @returns {string} HTML string
 */
export function createBoxedList(content) {
  return `<div class="c-boxed-list">${content}</div>`;
}

/**
 * Creates a standardized list row with a control.
 * @param {Object} options
 * @param {string} options.title - Primary title
 * @param {string} [options.desc] - Optional description
 * @param {string} [options.tooltip] - Optional tooltip text
 * @param {string} options.control - HTML of the control component
 * @param {string} [options.id] - Optional ID for the label wrapper
 * @param {boolean} [options.isLabel=true] - Use <label> or <div>
 * @returns {string} HTML string
 */
export function createListRow({ title, desc, tooltip, control, id, isLabel = true }) {
  const tag = isLabel ? 'label' : 'div';
  const labelId = id ? `id="${id}"` : '';
  const tooltipHtml = tooltip ? `
    <span class="c-list-row__info-icon" data-tooltip="${tooltip}">
      ${icon('info', 20)}
    </span>
  ` : '';

  return `
    <${tag} class="c-list-row" ${labelId}>
      <div class="c-list-row__content">
        <div class="c-list-row__title">${title}</div>
        ${desc ? `<div class="c-list-row__desc">${desc}</div>` : ''}
      </div>
      <div class="settings-row__control">
        ${tooltipHtml}
        ${control}
      </div>
    </${tag}>
  `;
}

/**
 * Creates a Benmari Switch.
 */
export function createSwitch({ id, checked = false }) {
  return `
    <input type="checkbox" class="c-input-hidden" id="${id}" ${checked ? 'checked' : ''}>
    <div class="c-switch">
      <div class="c-switch__handle"></div>
    </div>
  `;
}

/**
 * Creates a generic Segmented Control (Switcher).
 */
export function createSegmentedControl({ id, options, activeValue, indicatorId, className = '' }) {
  const indicatorHtml = indicatorId ? `<div class="c-segmented-control__indicator" id="${indicatorId}"></div>` : '';
  
  return `
    <div class="c-segmented-control c-segmented-control--responsive ${className}" id="${id}">
      ${indicatorHtml}
      ${options.map(opt => `
        <button class="c-segmented-control__btn ${activeValue === opt.value ? 'c-segmented-control__btn--active' : ''}" 
                data-value="${opt.value}" ${opt.tooltip ? `data-tooltip="${opt.tooltip}"` : ''}>
          ${opt.icon ? icon(opt.icon, 18) : ''}
          ${opt.label ? `<span>${opt.label}</span>` : ''}
        </button>
      `).join('')}
    </div>
  `;
}

/**
 * Creates a Segmented Theme Switcher.
 */
export function createThemeSwitcher(currentTheme) {
  return createSegmentedControl({
    id: 'settings-theme-switcher',
    className: 'settings-theme-switcher',
    indicatorId: 'settings-theme-indicator',
    activeValue: currentTheme,
    options: [
      { value: 'sistem', label: 'Sistem', icon: 'system', tooltip: 'Sistem Teması' },
      { value: 'acik', label: 'Açık', icon: 'sun', tooltip: 'Açık Tema' },
      { value: 'koyu', label: 'Koyu', icon: 'moon', tooltip: 'Koyu Tema' }
    ]
  });
}

/**
 * Creates a standard button for list rows.
 */
export function createButton({ label, variant = 'secondary', id }) {
  const btnId = id ? `id="${id}"` : '';
  return `
    <button class="btn btn--${variant}" ${btnId}>
      <span class="btn__text">${label}</span>
      <div class="btn__loader">
        <div class="m3-loader m3-loader--sm m3-loader--current">
          <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
            <path d="M50,10 C72,10 90,28 90,50 C90,72 72,90 50,90 C28,90 10,72 10,50 C10,28 28,10 50,10 Z" fill="currentColor">
              <animate attributeName="d" dur="3s" repeatCount="indefinite" calcMode="spline"
                keyTimes="0; 0.33; 0.66; 1" keySplines="0.4 0 0.2 1; 0.4 0 0.2 1; 0.4 0 0.2 1"
                values="
                  M50,10 C72,10 90,28 90,50 C90,72 72,90 50,90 C28,90 10,72 10,50 C10,28 28,10 50,10 Z;
                  M50,10 C85,10 90,15 90,50 C90,85 85,90 50,90 C15,90 10,85 10,50 C10,15 15,10 50,10 Z;
                  M50,10 C60,40 60,40 90,50 C60,60 60,60 50,90 C40,60 40,60 10,50 C40,40 40,40 50,10 Z;
                  M50,10 C72,10 90,28 90,50 C90,72 72,90 50,90 C28,90 10,72 10,50 C10,28 28,10 50,10 Z" />
            </path>
          </svg>
        </div>
      </div>
    </button>`;
}

/**
 * Creates a simple value display (e.g. for email).
 */
export function createValueDisplay(value) {
  return `<span class="settings-row__value">${value}</span>`;
}
/**
 * Creates a standardized Badge (rozet).
 * @param {Object} options
 * @param {string} options.label - Text content
 * @param {string} [options.variant='default'] - default, primary, success, warning, info
 * @param {string} [options.size='md'] - sm, md
 * @param {string} [options.iconName] - Optional icon name
 * @returns {string} HTML string
 */
export function createBadge({ label, variant = 'default', size = 'md', iconName }) {
  const iconHtml = iconName ? icon(iconName, size === 'sm' ? 12 : 14) : '';
  return `
    <span class="c-badge c-badge--${variant} c-badge--${size}">
      ${iconHtml}
      <span>${label}</span>
    </span>
  `;
}
