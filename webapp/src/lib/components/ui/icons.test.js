import { describe, it, expect } from 'vitest';
import { icon, svgToDataUri, icons } from './icons.js';

describe('icon() component helper', () => {
  it('renders icon with explicit aria-label and empty className', () => {
    const markup = icon('logoExperimental', null, '', 'Kepçe Logosu');
    expect(markup).toContain('role="img"');
    expect(markup).toContain('aria-label="Kepçe Logosu"');
    expect(markup).not.toContain('class="Kepçe Logosu"');
    expect(markup).not.toContain('aria-hidden');
  });

  it('defensively re-routes human-readable label with Turkish characters from className to ariaLabel', () => {
    // If a developer accidentally passes the label as the 3rd argument (className)
    const markup = icon('logoExperimental', null, 'Kepçe Logosu');
    expect(markup).not.toContain('class="Kepçe Logosu"');
    expect(markup).toContain('role="img"');
    expect(markup).toContain('aria-label="Kepçe Logosu"');
  });

  it('keeps valid CSS class names as className and applies aria-hidden when no label', () => {
    const markup = icon('search', 20, 'c-search-box__icon');
    expect(markup).toContain('class="c-search-box__icon"');
    expect(markup).toContain('aria-hidden="true"');
    expect(markup).not.toContain('role="img"');
  });

  it('supports options object signature', () => {
    const markup = icon('logoSmallExperimental', { size: 36, ariaLabel: 'Kepçe Logosu' });
    expect(markup).toContain('width="36"');
    expect(markup).toContain('height="36"');
    expect(markup).toContain('role="img"');
    expect(markup).toContain('aria-label="Kepçe Logosu"');
  });

  it('does not contain unresolvable CSS variables in isolated SVG assets', () => {
    // Isolated SVGs used as images, data URIs or canvas cannot resolve CSS var(--...)
    expect(icons.logoExperimental).not.toContain('var(--');
    expect(icons.logoSmallExperimental).not.toContain('var(--');
    expect(icons.avatarEmpty).not.toContain('var(--');

    // Confirm solid brand colors are present
    expect(icons.logoExperimental).toContain('#ECBF7F');
    expect(icons.logoExperimental).toContain('#242828');
    expect(icons.logoSmallExperimental).toContain('#ECBF7F');
    expect(icons.logoSmallExperimental).toContain('#242828');
  });
});

describe('svgToDataUri() helper', () => {
  it('converts SVG with Turkish UTF-8 characters to percent-encoded data URI', () => {
    const svg = icon('logoExperimental', null, '', 'Kepçe Logosu');
    const uri = svgToDataUri(svg, false);

    expect(uri.startsWith('data:image/svg+xml;charset=utf-8,')).toBe(true);
    expect(uri).toContain(encodeURIComponent('Kepçe Logosu'));
  });

  it('converts SVG with Turkish UTF-8 characters to base64 without btoa Latin-1 error', () => {
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" aria-label="Kepçe Logosu"><text>Kepçe</text></svg>`;
    const uri = svgToDataUri(svg, true);

    expect(uri.startsWith('data:image/svg+xml;base64,')).toBe(true);
    const base64Data = uri.replace('data:image/svg+xml;base64,', '');

    // Decoding should retrieve original UTF-8 content
    const binary = atob(base64Data);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    const decoded = new TextDecoder().decode(bytes);
    expect(decoded).toBe(svg);
  });
});
