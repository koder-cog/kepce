import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import SegmentedControl from './SegmentedControl.svelte';

describe('SegmentedControl Component', () => {
  const options = [
    { value: 'tab1', label: 'Birinci' },
    { value: 'tab2', label: 'İkinci' },
    { value: 'tab3', label: 'Üçüncü' },
  ];

  it('renders all options with appropriate accessibility roles', () => {
    const { container } = render(SegmentedControl, {
      value: 'tab1',
      options,
    });

    const radiogroup = container.querySelector('[role="radiogroup"]');
    expect(radiogroup).toBeTruthy();

    const buttons = screen.getAllByRole('radio');
    expect(buttons.length).toBe(3);
    expect(buttons[0].getAttribute('aria-checked')).toBe('true');
    expect(buttons[0].getAttribute('tabindex')).toBe('0');
    expect(buttons[1].getAttribute('aria-checked')).toBe('false');
    expect(buttons[1].getAttribute('tabindex')).toBe('-1');
  });

  it('triggers onChange when clicking an option', async () => {
    const onChange = vi.fn();
    render(SegmentedControl, {
      value: 'tab1',
      options,
      onChange,
    });

    const secondBtn = screen.getByText('İkinci');
    await fireEvent.click(secondBtn);

    expect(onChange).toHaveBeenCalledWith('tab2', expect.anything());
  });

  it('navigates with keyboard arrow keys and supports wrap-around', async () => {
    const onChange = vi.fn();
    const { container } = render(SegmentedControl, {
      value: 'tab1',
      options,
      onChange,
    });

    const radiogroup = container.querySelector('[role="radiogroup"]');
    expect(radiogroup).toBeTruthy();

    // ArrowRight from tab1 (0) -> tab2 (1)
    await fireEvent.keyDown(radiogroup, { key: 'ArrowRight' });
    expect(onChange).toHaveBeenNthCalledWith(1, 'tab2', expect.anything());

    // ArrowLeft from tab2 (1) -> tab1 (0)
    await fireEvent.keyDown(radiogroup, { key: 'ArrowLeft' });
    expect(onChange).toHaveBeenNthCalledWith(2, 'tab1', expect.anything());

    // End key -> tab3 (2)
    await fireEvent.keyDown(radiogroup, { key: 'End' });
    expect(onChange).toHaveBeenNthCalledWith(3, 'tab3', expect.anything());

    // Home key -> tab1 (0)
    await fireEvent.keyDown(radiogroup, { key: 'Home' });
    expect(onChange).toHaveBeenNthCalledWith(4, 'tab1', expect.anything());
  });
});
