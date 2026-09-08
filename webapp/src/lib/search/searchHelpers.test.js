import { describe, it, expect } from "vitest";
import {
  cleanLeadParentheses,
  getYoutubeId,
  getYoutubeThumbnail,
} from "./searchHelpers.js";

describe("searchHelpers - cleanLeadParentheses", () => {
  it("removes pronunciation and IPA notations", () => {
    const raw = "Python (İngilizce telaffuz: [ˈpaɪθɑːn]), nesne yönelimli, yorumsal bir dildir.";
    expect(cleanLeadParentheses(raw)).toBe(
      "Python, nesne yönelimli, yorumsal bir dildir."
    );
  });

  it("removes foreign language names from lead sentence", () => {
    const raw = "Londra (İngilizce: London), Birleşik Krallık'ın başkentidir.";
    expect(cleanLeadParentheses(raw)).toBe(
      "Londra, Birleşik Krallık'ın başkentidir."
    );
  });

  it("removes birth-death date ranges from lead sentence", () => {
    const raw =
      "Albert Einstein (14 Mart 1879, Ulm - 18 Nisan 1955, Princeton), Yahudi asıllı Alman teorik fizikçidir.";
    expect(cleanLeadParentheses(raw)).toBe(
      "Albert Einstein, Yahudi asıllı Alman teorik fizikçidir."
    );

    const rawShort = "Mustafa Kemal Atatürk (1881 – 10 Kasım 1938), Türkiye Cumhuriyeti'nin kurucusudur.";
    expect(cleanLeadParentheses(rawShort)).toBe(
      "Mustafa Kemal Atatürk, Türkiye Cumhuriyeti'nin kurucusudur."
    );
  });

  it("preserves explanatory non-language/non-date parentheses", () => {
    const raw = "X (eski adıyla Twitter), bir sosyal medya platformudur.";
    expect(cleanLeadParentheses(raw)).toBe(
      "X (eski adıyla Twitter), bir sosyal medya platformudur."
    );
  });

  it("handles empty or null text safely", () => {
    expect(cleanLeadParentheses("")).toBe("");
    expect(cleanLeadParentheses(null)).toBe("");
  });
});

describe("searchHelpers - YouTube utilities", () => {
  it("extracts YouTube video id from standard and short URLs", () => {
    expect(
      getYoutubeId("https://www.youtube.com/watch?v=zd8IFDgQCUc")
    ).toBe("zd8IFDgQCUc");
    expect(getYoutubeId("https://youtu.be/zd8IFDgQCUc")).toBe(
      "zd8IFDgQCUc"
    );
    expect(getYoutubeId("https://example.com/not-youtube")).toBeNull();
  });

  it("generates correct YouTube thumbnail URL", () => {
    expect(
      getYoutubeThumbnail("https://www.youtube.com/watch?v=zd8IFDgQCUc")
    ).toBe("https://i.ytimg.com/vi/zd8IFDgQCUc/hqdefault.jpg");
    expect(getYoutubeThumbnail("https://example.com")).toBeNull();
  });
});
