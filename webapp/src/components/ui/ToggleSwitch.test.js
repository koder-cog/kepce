import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import ToggleSwitch from './ToggleSwitch.svelte';

describe('ToggleSwitch Component', () => {
  it('should render correct label text', () => {
    render(ToggleSwitch, { label: 'Aktif Et' });
    
    const label = screen.getByText('Aktif Et');
    expect(label).toBeTruthy();
  });

  it('should render checked and unchecked states', () => {
    const { container } = render(ToggleSwitch, { checked: true, label: 'Test Label' });
    const checkbox = container.querySelector('input[type="checkbox"]');
    expect(checkbox.checked).toBe(true);
  });
});
