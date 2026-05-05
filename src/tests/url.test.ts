import { describe, expect, it } from "vitest";
import { extractUrlsFromText } from "../utils/url";

describe("url extraction", () => {
  it("extracts the youtube regression urls in order", () => {
    const text = [
      "https://www.youtube.com/watch?v=oyPhmcgVoSY",
      "https://www.youtube.com/watch?v=9E9y-suOleI",
      "https://www.youtube.com/watch?v=ZX_NwrgmYFk",
      "https://www.youtube.com/playlist?list=PLD3Ywi8n56O7MQLPjxEK52DegGaPFLKcQ"
    ].join(" ");

    expect(extractUrlsFromText(text)).toEqual([
      "https://www.youtube.com/watch?v=oyPhmcgVoSY",
      "https://www.youtube.com/watch?v=9E9y-suOleI",
      "https://www.youtube.com/watch?v=ZX_NwrgmYFk",
      "https://www.youtube.com/playlist?list=PLD3Ywi8n56O7MQLPjxEK52DegGaPFLKcQ"
    ]);
  });
});
