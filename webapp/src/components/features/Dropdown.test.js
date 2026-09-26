import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import Dropdown from './Dropdown.svelte';

describe('Dropdown Race Condition and Synthetic Click Simulation', () => {
    beforeEach(() => {
        vi.useFakeTimers();
        // Simulate mobile viewport (width <= 600px)
        window.matchMedia = vi.fn().mockImplementation((query) => ({
            matches: query.includes('max-width: 600px'),
            media: query,
            onchange: null,
            addListener: vi.fn(),
            removeListener: vi.fn(),
            addEventListener: vi.fn(),
            removeEventListener: vi.fn(),
            dispatchEvent: vi.fn(),
        }));
    });

    afterEach(() => {
        vi.useRealTimers();
        // Clean up portal elements attached to document.body
        const portals = document.querySelectorAll('.c-menu, .c-menu__overlay');
        portals.forEach(el => el.remove());
    });

    const sampleOptions = [
        { value: 'istanbul', label: 'İstanbul' },
        { value: 'ankara', label: 'Ankara' },
        { value: 'izmir', label: 'İzmir' },
        { value: 'bursa', label: 'Bursa' },
        { value: 'antalya', label: 'Antalya' },
        { value: 'adana', label: 'Adana' },
        { value: 'konya', label: 'Konya' },
        { value: 'trabzon', label: 'Trabzon' },
        { value: 'eskisehir', label: 'Eskişehir' },
        { value: 'gaziantep', label: 'Gaziantep' },
    ];

    it('simulates rapid synthetic click on trigger button (ghost click): menu stays open', async () => {
        const { container } = render(Dropdown, {
            options: sampleOptions,
            value: 'istanbul',
            placeholder: 'Şehir'
        });

        const triggerBtn = container.querySelector('.dropdown__trigger');
        expect(triggerBtn).toBeTruthy();

        // 1. User taps the button (first click/touch)
        await fireEvent.click(triggerBtn);
        expect(container.querySelector('.dropdown--open')).toBeTruthy();

        // 2. Browser fires synthetic click 80ms later (within the 300ms window)
        vi.advanceTimersByTime(80);
        await fireEvent.click(triggerBtn);

        // Verification: The menu must still be OPEN!
        expect(container.querySelector('.dropdown--open')).toBeTruthy();

        // 3. User legitimately clicks after 350ms to close
        vi.advanceTimersByTime(300); // total 380ms
        await fireEvent.click(triggerBtn);
        await vi.advanceTimersByTimeAsync(300);
        // Now it closes
        expect(container.querySelector('.dropdown--open')).toBeNull();
    });

    it('simulates synthetic click on backdrop overlay: menu stays open', async () => {
        const { container } = render(Dropdown, {
            options: sampleOptions,
            value: 'istanbul',
            placeholder: 'Şehir'
        });

        const triggerBtn = container.querySelector('.dropdown__trigger');
        await fireEvent.click(triggerBtn);
        expect(container.querySelector('.dropdown--open')).toBeTruthy();

        // The overlay was appended to body via portal
        const overlay = document.querySelector('.c-menu__overlay');
        expect(overlay).toBeTruthy();

        // Synthetic click lands on the overlay at 120ms
        vi.advanceTimersByTime(120);
        await fireEvent.click(overlay);

        // Verification: Ghost click ignored, menu stays open!
        expect(container.querySelector('.dropdown--open')).toBeTruthy();

        // After cooldown passes (350ms total), tapping overlay closes it
        vi.advanceTimersByTime(250);
        await fireEvent.click(overlay);
        await vi.advanceTimersByTimeAsync(300);
        expect(container.querySelector('.dropdown--open')).toBeNull();
    });

    it('simulates page loading scroll event: modal menu stays open', async () => {
        const { container } = render(Dropdown, {
            options: sampleOptions,
            value: 'istanbul',
            placeholder: 'Şehir'
        });

        const triggerBtn = container.querySelector('.dropdown__trigger');
        await fireEvent.click(triggerBtn);
        expect(container.querySelector('.dropdown--open')).toBeTruthy();

        // While menu is open, page scroll event fires (e.g. from calendar scrollIntoView) at 150ms
        vi.advanceTimersByTime(150);
        await fireEvent.scroll(window);

        // Verification: Mobile bottom sheet does NOT close on background scroll
        expect(container.querySelector('.dropdown--open')).toBeTruthy();
    });

    it('simulates click outside: ignored during 300ms cooldown, closes after cooldown', async () => {
        const { container } = render(Dropdown, {
            options: sampleOptions,
            value: 'istanbul',
            placeholder: 'Şehir'
        });

        const triggerBtn = container.querySelector('.dropdown__trigger');
        await fireEvent.click(triggerBtn);
        expect(container.querySelector('.dropdown--open')).toBeTruthy();

        // Click outside at 100ms
        vi.advanceTimersByTime(100);
        await fireEvent.click(document.body);
        // Still open
        expect(container.querySelector('.dropdown--open')).toBeTruthy();

        // Click outside after cooldown (400ms total)
        vi.advanceTimersByTime(300);
        await fireEvent.click(document.body);
        await vi.advanceTimersByTimeAsync(300);
        // Closes
        expect(container.querySelector('.dropdown--open')).toBeNull();
    });
});
