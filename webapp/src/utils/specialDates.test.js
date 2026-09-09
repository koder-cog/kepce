import { describe, it, expect } from "vitest";
import {
    parseDateKey,
    isMourningDay,
    isScholarshipPeriod,
    getSpecialDayInfo,
} from "./specialDates.js";

describe("specialDates Utility", () => {
    it("parseDateKey hem ISO string hem Date nesnesini doğru ayrıştırmalı", () => {
        const strRes = parseDateKey("2026-10-06");
        expect(strRes.mmdd).toBe("10-06");
        expect(strRes.day).toBe(6);

        const dateRes = parseDateKey(new Date(2026, 9, 6));
        expect(dateRes.mmdd).toBe("10-06");
        expect(dateRes.day).toBe(6);
    });

    it("10 Kasım gününü doğru matem günü olarak tanımalı", () => {
        expect(isMourningDay("2026-11-10")).toBe(true);
        expect(isMourningDay("2026-11-09")).toBe(false);
        expect(isMourningDay(new Date(2026, 10, 10))).toBe(true);

        const info = getSpecialDayInfo("2026-11-10");
        expect(info?.isMourning).toBe(true);
        expect(info?.name).toContain("Atatürk");
    });

    it("6 Ekim'de İstanbul için yerel kurtuluş günü dönmeli, başka şehirde dönmemeli", () => {
        const ist = getSpecialDayInfo("2026-10-06", "istanbul");
        expect(ist?.name).toBe("İstanbul'un Kurtuluşu");
        expect(ist?.isLocal).toBe(true);

        const ank = getSpecialDayInfo("2026-10-06", "ankara");
        // 6 Ekim Ankara için yerel gün değil, ama ayın 6'sı olduğu için burs dönemine düşebilir
        expect(ank?.name).not.toBe("İstanbul'un Kurtuluşu");
    });

    it("9 Eylül'de İzmir için kurtuluş günü dönmeli", () => {
        const izm = getSpecialDayInfo("2026-09-09", "izmir");
        expect(izm?.name).toBe("İzmir'in Kurtuluşu");
        expect(izm?.isLocal).toBe(true);
    });

    it("29 Ekim milli bayram olarak kutlama bayrağıyla dönmeli", () => {
        const rep = getSpecialDayInfo("2026-10-29", "istanbul");
        expect(rep?.name).toBe("Cumhuriyet Bayramı");
        expect(rep?.isCelebration).toBe(true);
    });

    it("özel bir gün olmayan ayın 7'sinde burs dönemi olarak tanınmalı", () => {
        expect(isScholarshipPeriod("2026-03-07")).toBe(true);
        expect(isScholarshipPeriod("2026-03-12")).toBe(false);

        const bursInfo = getSpecialDayInfo("2026-03-07", "ankara");
        expect(bursInfo?.isScholarship).toBe(true);
        expect(bursInfo?.name).toContain("Burs");
    });

    it("sıradan bir günde null dönmeli", () => {
        const normal = getSpecialDayInfo("2026-02-15", "istanbul");
        expect(normal).toBeNull();
    });
});
