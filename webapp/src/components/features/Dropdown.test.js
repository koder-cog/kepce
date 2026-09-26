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

    it('bypasses history.pushState on iOS devices to prevent predictive back conflicts', async () => {
        const pushStateSpy = vi.spyOn(history, 'pushState');
        const originalUserAgent = navigator.userAgent;

        // Mock iPhone userAgent
        Object.defineProperty(navigator, 'userAgent', {
            value: 'Mozilla/5.0 (iPhone; CPU iPhone OS 27_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.0 Mobile/15E148 Safari/604.1',
            configurable: true
        });

        const { container } = render(Dropdown, {
            options: sampleOptions,
            value: 'istanbul',
            placeholder: 'Şehir'
        });

        const triggerBtn = container.querySelector('.dropdown__trigger');
        await fireEvent.click(triggerBtn);
        expect(container.querySelector('.dropdown--open')).toBeTruthy();

        // Verification: pushState must NOT be called on iOS!
        expect(pushStateSpy).not.toHaveBeenCalled();

        // Restore userAgent
        Object.defineProperty(navigator, 'userAgent', {
            value: originalUserAgent,
            configurable: true
        });
        pushStateSpy.mockRestore();
    });

    it('bypasses history.pushState on iPadOS in desktop Safari mode without navigator.platform', async () => {
        const pushStateSpy = vi.spyOn(history, 'pushState');
        const originalUserAgent = navigator.userAgent;
        const originalTouchPoints = navigator.maxTouchPoints;

        // Mock iPad desktop mode userAgent and touch points
        Object.defineProperty(navigator, 'userAgent', {
            value: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.0 Safari/605.1.15',
            configurable: true
        });
        Object.defineProperty(navigator, 'maxTouchPoints', {
            value: 5,
            configurable: true
        });

        const { container } = render(Dropdown, {
            options: sampleOptions,
            value: 'istanbul',
            placeholder: 'Şehir'
        });

        const triggerBtn = container.querySelector('.dropdown__trigger');
        await fireEvent.click(triggerBtn);
        expect(container.querySelector('.dropdown--open')).toBeTruthy();

        // Verification: pushState must NOT be called on iPadOS!
        expect(pushStateSpy).not.toHaveBeenCalled();

        // Restore
        Object.defineProperty(navigator, 'userAgent', {
            value: originalUserAgent,
            configurable: true
        });
        Object.defineProperty(navigator, 'maxTouchPoints', {
            value: originalTouchPoints,
            configurable: true
        });
        pushStateSpy.mockRestore();
    });

    it('calls history.pushState on non-iOS (Android) devices for system back support', async () => {
        const pushStateSpy = vi.spyOn(history, 'pushState');
        const originalUserAgent = navigator.userAgent;

        // Mock Android userAgent
        Object.defineProperty(navigator, 'userAgent', {
            value: 'Mozilla/5.0 (Linux; Android 15; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Mobile Safari/537.36',
            configurable: true
        });

        const { container } = render(Dropdown, {
            options: sampleOptions,
            value: 'istanbul',
            placeholder: 'Şehir'
        });

        const triggerBtn = container.querySelector('.dropdown__trigger');
        await fireEvent.click(triggerBtn);
        expect(container.querySelector('.dropdown--open')).toBeTruthy();

        // Verification: pushState must be called on Android!
        expect(pushStateSpy).toHaveBeenCalledWith({ kepceDropdown: true }, '');

        // Restore userAgent
        Object.defineProperty(navigator, 'userAgent', {
            value: originalUserAgent,
            configurable: true
        });
        pushStateSpy.mockRestore();
    });

    it('blurs search input instead of selecting option when Enter is pressed without arrow key navigation', async () => {
        const onChange = vi.fn();
        const { container } = render(Dropdown, {
            options: sampleOptions,
            value: 'istanbul',
            placeholder: 'Şehir',
            onChange
        });

        const triggerBtn = container.querySelector('.dropdown__trigger');
        await fireEvent.click(triggerBtn);
        expect(container.querySelector('.dropdown--open')).toBeTruthy();

        const searchInput = document.querySelector('.c-menu__search input');
        expect(searchInput).toBeTruthy();

        const blurSpy = vi.spyOn(searchInput, 'blur');
        searchInput.focus();

        // User types search query (mobile keyboard input)
        await fireEvent.input(searchInput, { target: { value: 'ank' } });

        // User hits the on-screen keyboard "Ara / Git" button (Enter key)
        await fireEvent.keyDown(searchInput, { key: 'Enter', code: 'Enter' });

        // Verification: blur is called, onChange is NOT called, menu stays OPEN!
        expect(blurSpy).toHaveBeenCalled();
        expect(onChange).not.toHaveBeenCalled();
        expect(container.querySelector('.dropdown--open')).toBeTruthy();

        blurSpy.mockRestore();
    });

    it('selects option with Enter when user intentionally navigates using arrow keys', async () => {
        const onChange = vi.fn();
        const { container } = render(Dropdown, {
            options: sampleOptions,
            value: 'istanbul',
            placeholder: 'Şehir',
            onChange
        });

        const triggerBtn = container.querySelector('.dropdown__trigger');
        await fireEvent.click(triggerBtn);
        expect(container.querySelector('.dropdown--open')).toBeTruthy();

        const searchInput = document.querySelector('.c-menu__search input');
        expect(searchInput).toBeTruthy();
        searchInput.focus();

        // User navigates down with ArrowDown
        await fireEvent.keyDown(searchInput, { key: 'ArrowDown', code: 'ArrowDown' });

        // Now Enter commits the selection
        await fireEvent.keyDown(searchInput, { key: 'Enter', code: 'Enter' });
        expect(onChange).toHaveBeenCalled();
    });

    it('animates smoothly between default and expanded when handle is clicked', async () => {
        const { container } = render(Dropdown, {
            options: sampleOptions,
            value: 'istanbul',
            placeholder: 'Şehir'
        });

        const triggerBtn = container.querySelector('.dropdown__trigger');
        await fireEvent.click(triggerBtn);
        expect(container.querySelector('.dropdown--open')).toBeTruthy();

        const menuEl = document.querySelector('.c-menu');
        const handle = document.querySelector('.c-menu__handle');
        expect(handle).toBeTruthy();
        expect(menuEl.classList.contains('c-menu--expanded')).toBe(false);

        // 1. Click handle to expand
        await fireEvent.click(handle);
        expect(menuEl.classList.contains('c-menu--expanded')).toBe(true);
        // Verify inline transition is set for smooth height animation
        expect(menuEl.style.transition).toContain('height');

        // Advance timers past transition (240ms + 50ms buffer)
        vi.advanceTimersByTime(300);
        // Once done, inline transition and height are cleaned up so CSS takes over
        expect(menuEl.style.transition).toBe('');
        expect(menuEl.style.height).toBe('');
        expect(menuEl.classList.contains('c-menu--expanded')).toBe(true);

        // 2. Click handle to toggle back to default
        await fireEvent.click(handle);
        expect(menuEl.classList.contains('c-menu--expanded')).toBe(false);
        expect(menuEl.style.transition).toContain('height');

        vi.advanceTimersByTime(300);
        expect(menuEl.style.transition).toBe('');
        expect(menuEl.style.height).toBe('');
        expect(menuEl.classList.contains('c-menu--expanded')).toBe(false);
    });

    it('maintains expanded state and height during escape or overlay close without jumping to default', async () => {
        const { container } = render(Dropdown, {
            options: sampleOptions,
            value: 'istanbul',
            placeholder: 'Şehir'
        });

        const triggerBtn = container.querySelector('.dropdown__trigger');
        await fireEvent.click(triggerBtn);

        const menuEl = document.querySelector('.c-menu');
        const handle = document.querySelector('.c-menu__handle');

        // Click handle to expand
        await fireEvent.click(handle);
        vi.advanceTimersByTime(300);
        expect(menuEl.classList.contains('c-menu--expanded')).toBe(true);

        // Press Escape while expanded
        const searchInput = document.querySelector('.c-menu__search input');
        await fireEvent.keyDown(searchInput, { key: 'Escape', code: 'Escape' });

        // During close animation, menu must NOT drop back to default class
        expect(menuEl.classList.contains('c-menu--expanded')).toBe(true);
        expect(menuEl.style.transform).toBe('translateY(100%)');
        expect(menuEl.style.maxHeight).toBe('none');

        // After close animation finishes, it fully closes
        await vi.advanceTimersByTimeAsync(300);
        expect(container.querySelector('.dropdown--open')).toBeNull();
    });

    it('allows dragging handle with mouse pointer to expand the sheet', async () => {
        const { container } = render(Dropdown, {
            options: sampleOptions,
            value: 'istanbul',
            placeholder: 'Şehir'
        });

        const triggerBtn = container.querySelector('.dropdown__trigger');
        await fireEvent.click(triggerBtn);

        const menuEl = document.querySelector('.c-menu');
        const handle = document.querySelector('.c-menu__handle');
        expect(handle).toBeTruthy();
        expect(menuEl.classList.contains('c-menu--expanded')).toBe(false);

        // Simulate mouse pointer drag upwards on the handle
        await fireEvent.pointerDown(handle, {
            pointerId: 1,
            pointerType: 'mouse',
            button: 0,
            clientY: 500
        });

        await fireEvent.pointerMove(menuEl, {
            pointerId: 1,
            clientY: 420
        });

        await fireEvent.pointerUp(menuEl, {
            pointerId: 1,
            clientY: 420
        });

        // Verify it snapped to expanded
        expect(menuEl.classList.contains('c-menu--expanded')).toBe(true);
    });
});


