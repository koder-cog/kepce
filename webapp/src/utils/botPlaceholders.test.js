import { describe, it, expect } from "vitest";
import {
    BOT_EMPTY_PLACEHOLDERS,
    parseDateParts,
    getEpochDay,
    getBotPlaceholderComment,
} from "./botPlaceholders.js";

describe("botPlaceholders Utility", () => {
    it("havuz tam 14 metin içermeli ve hiçbiri boş olmamalı", () => {
        expect(BOT_EMPTY_PLACEHOLDERS.length).toBe(14);
        for (const text of BOT_EMPTY_PLACEHOLDERS) {
            expect(typeof text).toBe("string");
            expect(text.trim().length).toBeGreaterThan(20);
        }
    });

    it("metinler responsive uyum için sol/sağ/yan gibi konumsal kelimeler barındırmamalı", () => {
        const forbiddenWords = ["sol tarafta", "sağ tarafta", "yandaki", "şu yandaki"];
        for (const text of BOT_EMPTY_PLACEHOLDERS) {
            for (const word of forbiddenWords) {
                expect(text.toLowerCase()).not.toContain(word);
            }
        }
    });

    it("parseDateParts hem Date nesnesini hem de string formatını saat dilimi farkı olmadan ayrıştırmalı", () => {
        const fromStr = parseDateParts("2026-09-09");
        expect(fromStr).toEqual({ y: 2026, m: 8, d: 9 });

        const fromDate = parseDateParts(new Date(2026, 8, 9));
        expect(fromDate).toEqual({ y: 2026, m: 8, d: 9 });
    });

    it("aynı tarih için her zaman deterministik olarak aynı metin dönmeli", () => {
        const dateStr = "2026-09-09";
        const first = getBotPlaceholderComment(dateStr);
        const second = getBotPlaceholderComment(dateStr);
        const third = getBotPlaceholderComment(new Date(2026, 8, 9));

        expect(first).toBe(second);
        expect(first).toBe(third);
    });

    it("14 günlük bir periyot boyunca 14 farklı metnin tamamı tam olarak birer kez dönmeli", () => {
        // Herhangi bir chunk başlangıcı bul (epochDay % 14 === 0)
        let testEpoch = 20000;
        while (testEpoch % 14 !== 0) {
            testEpoch++;
        }

        const seenTexts = new Set();
        for (let i = 0; i < 14; i++) {
            const date = new Date((testEpoch + i) * 86400000);
            const text = getBotPlaceholderComment(date);
            seenTexts.add(text);
        }

        expect(seenTexts.size).toBe(14);
        for (const placeholder of BOT_EMPTY_PLACEHOLDERS) {
            expect(seenTexts.has(placeholder)).toBe(true);
        }
    });

    it("ardışık iki 14 günlük blok farklı permütasyon sıralamasına sahip olmalı", () => {
        let testEpoch = 20000;
        while (testEpoch % 14 !== 0) {
            testEpoch++;
        }

        const chunk1 = [];
        for (let i = 0; i < 14; i++) {
            const date = new Date((testEpoch + i) * 86400000);
            chunk1.push(getBotPlaceholderComment(date));
        }

        const chunk2 = [];
        for (let i = 0; i < 14; i++) {
            const date = new Date((testEpoch + 14 + i) * 86400000);
            chunk2.push(getBotPlaceholderComment(date));
        }

        expect(chunk1).not.toEqual(chunk2);
    });

    it("31 Aralık - 1 Ocak yıl geçişinde fonksiyon hata vermemeli ve deterministik işlemeli", () => {
        const dec31 = getBotPlaceholderComment("2025-12-31");
        const jan1 = getBotPlaceholderComment("2026-01-01");

        expect(typeof dec31).toBe("string");
        expect(typeof jan1).toBe("string");
    });
});
