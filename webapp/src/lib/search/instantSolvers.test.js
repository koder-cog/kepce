import { describe, it, expect } from "vitest";
import {
  solveUnitConversion,
  suggestUnitCorrection,
  solveWorldTime,
  levenshteinDistance,
  findClosestUnit,
  solveTdkDefinition,
} from "./instantSolvers.js";

describe("instantSolvers - levenshteinDistance", () => {
  it("computes single-character substitution correctly", () => {
    expect(levenshteinDistance("lg", "kg")).toBe(1);
    expect(levenshteinDistance("cm", "km")).toBe(1);
  });

  it("handles identical and empty strings", () => {
    expect(levenshteinDistance("meter", "meter")).toBe(0);
    expect(levenshteinDistance("", "abc")).toBe(3);
  });
});

describe("instantSolvers - findClosestUnit", () => {
  it("finds closest unit in mass category for typo 'lg'", () => {
    const match = findClosestUnit("lg", "mass", "g");
    expect(match).not.toBeNull();
    expect(match.code).toBe("kg");
  });

  it("finds closest unit in length category for typo 'mtre'", () => {
    const match = findClosestUnit("mtre", "length", "km");
    expect(match).not.toBeNull();
    expect(match.code).toBe("m");
  });
});

describe("instantSolvers - suggestUnitCorrection", () => {
  it("corrects '50 g kaç lg' to '50 g kaç kg'", () => {
    const correction = suggestUnitCorrection("50 g kaç lg");
    expect(correction).not.toBeNull();
    expect(correction.correctedQuery).toBe("50 g kaç kg");
    expect(correction.suggested).toBe("kg");
    expect(correction.solved).not.toBeNull();
    expect(correction.solved.toAmount).toBe(0.05);
  });

  it("corrects '50 lg kaç g' to '50 kg kaç g'", () => {
    const correction = suggestUnitCorrection("50 lg kaç g");
    expect(correction).not.toBeNull();
    expect(correction.correctedQuery).toBe("50 kg kaç g");
    expect(correction.suggested).toBe("kg");
    expect(correction.solved).not.toBeNull();
    expect(correction.solved.toAmount).toBe(50000);
  });

  it("returns null when conversion is already valid", () => {
    expect(suggestUnitCorrection("50 g kaç kg")).toBeNull();
    expect(suggestUnitCorrection("100 km to m")).toBeNull();
  });

  it("returns null for non-unit queries", () => {
    expect(suggestUnitCorrection("istanbul hava durumu")).toBeNull();
    expect(suggestUnitCorrection("")).toBeNull();
  });
});

describe("instantSolvers - solveWorldTime", () => {
  it("resolves general 'saat kaç' queries to Turkey/Istanbul local time", () => {
    const ans1 = solveWorldTime("saat kaç");
    expect(ans1).not.toBeNull();
    expect(ans1.type).toBe("time");
    expect(ans1.city).toBe("İstanbul");
    expect(ans1.country).toBe("Türkiye");
    expect(ans1.diffText).toBe("Türkiye ile aynı saat diliminde");

    const ans2 = solveWorldTime("şu an saat kaç");
    expect(ans2).not.toBeNull();
    expect(ans2.city).toBe("İstanbul");

    const ans3 = solveWorldTime("saat kac?");
    expect(ans3).not.toBeNull();
    expect(ans3.city).toBe("İstanbul");
  });

  it("resolves city time queries like 'tokyo saati' and 'tokyo\\'da saat kaç'", () => {
    const ans1 = solveWorldTime("tokyo saati");
    expect(ans1).not.toBeNull();
    expect(ans1.city).toBe("Tokyo");
    expect(ans1.country).toBe("Japonya");
    expect(ans1.timezone).toBe("Asia/Tokyo");

    const ans2 = solveWorldTime("tokyo'da saat kaç");
    expect(ans2).not.toBeNull();
    expect(ans2.city).toBe("Tokyo");
  });

  it("resolves Turkish city queries like 'ankara saati' or 'türkiye saati'", () => {
    const ans1 = solveWorldTime("ankara saati");
    expect(ans1).not.toBeNull();
    expect(ans1.city).toBe("Ankara");
    expect(ans1.country).toBe("Türkiye");

    const ans2 = solveWorldTime("türkiye saati");
    expect(ans2).not.toBeNull();
    expect(ans2.city).toBe("İstanbul");
    expect(ans2.country).toBe("Türkiye");
  });

  it("resolves western capitals like 'londra saati' and 'new york saati'", () => {
    const london = solveWorldTime("londra saati");
    expect(london).not.toBeNull();
    expect(london.city).toBe("Londra");

    const ny = solveWorldTime("new york saati");
    expect(ny).not.toBeNull();
    expect(ny.city).toBe("New York");
  });

  it("dynamically resolves any of the 81 Turkish cities from CITY_MAP", () => {
    const trabzon = solveWorldTime("trabzon saati");
    expect(trabzon).not.toBeNull();
    expect(trabzon.city).toBe("Trabzon");
    expect(trabzon.country).toBe("Türkiye");
    expect(trabzon.timezone).toBe("Europe/Istanbul");

    const adana = solveWorldTime("adana'da saat kaç");
    expect(adana).not.toBeNull();
    expect(adana.city).toBe("Adana");
    expect(adana.country).toBe("Türkiye");
  });

  it("dynamically resolves world cities via IANA and Turkish aliases", () => {
    const kahire = solveWorldTime("kahire saati");
    expect(kahire).not.toBeNull();
    expect(kahire.city).toBe("Kahire");
    expect(kahire.timezone).toBe("Africa/Cairo");

    const viyana = solveWorldTime("viyana saati");
    expect(viyana).not.toBeNull();
    expect(viyana.city).toBe("Viyana");
    expect(viyana.timezone).toBe("Europe/Vienna");

    const seul = solveWorldTime("seul saati");
    expect(seul).not.toBeNull();
    expect(seul.city).toBe("Seul");
    expect(seul.timezone).toBe("Asia/Seoul");

    const chicago = solveWorldTime("chicago saati");
    expect(chicago).not.toBeNull();
    expect(chicago.timezone).toBe("America/Chicago");
  });

  it("returns null for unrelated queries", () => {
    expect(solveWorldTime("saat tamircisi")).toBeNull();
    expect(solveWorldTime("istanbul yemekleri")).toBeNull();
    expect(solveWorldTime("")).toBeNull();
  });
});

describe("instantSolvers - solveTdkDefinition", () => {
  it("resolves definition for 'tabldot nedir' via 'tabildot' alias", async () => {
    const res = await solveTdkDefinition("tabldot nedir");
    if (res) {
      expect(res.type).toBe("definition");
      expect(res.word).toBe("tabildot");
      expect(res.meanings.length).toBeGreaterThan(0);
    }
  });

  it("returns null for non-definition queries", async () => {
    expect(await solveTdkDefinition("istanbul hava durumu")).toBeNull();
    expect(await solveTdkDefinition("100 dolar kaç tl")).toBeNull();
  });
});
