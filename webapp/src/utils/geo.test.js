import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { detectCityPrecise } from './geo.js';

describe('detectCityPrecise Geolocation Error and Location Handling', () => {
    const originalGeolocation = navigator.geolocation;

    afterEach(() => {
        Object.defineProperty(navigator, 'geolocation', {
            value: originalGeolocation,
            configurable: true,
        });
    });

    it('returns not_supported when navigator.geolocation is unavailable', async () => {
        Object.defineProperty(navigator, 'geolocation', {
            value: undefined,
            configurable: true,
        });

        const result = await detectCityPrecise(['istanbul', 'ankara']);
        expect(result).toEqual({ success: false, error: 'not_supported' });
    });

    it('returns permission_denied when user blocks location access (code 1)', async () => {
        Object.defineProperty(navigator, 'geolocation', {
            value: {
                getCurrentPosition: vi.fn((_success, error) => {
                    error({ code: 1, message: 'User denied Geolocation' });
                }),
            },
            configurable: true,
        });

        const result = await detectCityPrecise(['istanbul', 'ankara']);
        expect(result).toEqual({ success: false, error: 'permission_denied' });
    });

    it('returns position_unavailable when device location cannot be acquired (code 2)', async () => {
        Object.defineProperty(navigator, 'geolocation', {
            value: {
                getCurrentPosition: vi.fn((_success, error) => {
                    error({ code: 2, message: 'Position unavailable' });
                }),
            },
            configurable: true,
        });

        const result = await detectCityPrecise(['istanbul', 'ankara']);
        expect(result).toEqual({ success: false, error: 'position_unavailable' });
    });

    it('returns timeout when location acquisition takes too long (code 3)', async () => {
        Object.defineProperty(navigator, 'geolocation', {
            value: {
                getCurrentPosition: vi.fn((_success, error) => {
                    error({ code: 3, message: 'Timeout expired' });
                }),
            },
            configurable: true,
        });

        const result = await detectCityPrecise(['istanbul', 'ankara']);
        expect(result).toEqual({ success: false, error: 'timeout' });
    });

    it('returns out_of_bounds when coordinates are outside Turkey (> 150 km from all provinces)', async () => {
        Object.defineProperty(navigator, 'geolocation', {
            value: {
                getCurrentPosition: vi.fn((success) => {
                    // London coordinates
                    success({ coords: { latitude: 51.5074, longitude: -0.1278 } });
                }),
            },
            configurable: true,
        });

        const result = await detectCityPrecise(['istanbul', 'ankara']);
        expect(result).toEqual({ success: false, error: 'out_of_bounds' });
    });

    it('returns success: true and slug when coordinates match an active province', async () => {
        Object.defineProperty(navigator, 'geolocation', {
            value: {
                getCurrentPosition: vi.fn((success) => {
                    // Kadıköy, Istanbul coordinates
                    success({ coords: { latitude: 40.9819, longitude: 29.0576 } });
                }),
            },
            configurable: true,
        });

        const result = await detectCityPrecise(['istanbul', 'ankara']);
        expect(result).toEqual({ success: true, slug: 'istanbul' });
    });

    it('returns unsupported: true when coordinates match a province that has no active menus', async () => {
        Object.defineProperty(navigator, 'geolocation', {
            value: {
                getCurrentPosition: vi.fn((success) => {
                    // Sinop coordinates
                    success({ coords: { latitude: 42.0231, longitude: 35.1531 } });
                }),
            },
            configurable: true,
        });

        // Sinop is not in availableSlugs
        const result = await detectCityPrecise(['istanbul', 'ankara']);
        expect(result).toEqual({
            success: false,
            unsupported: true,
            error: 'unsupported',
            slug: 'sinop',
        });
    });
});
