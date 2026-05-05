import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const pageFiles = [
  "src/pages/DownloadPage.vue",
  "src/pages/BatchPage.vue",
  "src/pages/HistoryPage.vue",
  "src/pages/ToolsPage.vue"
];

describe("table pages", () => {
  for (const file of pageFiles) {
    it(`uses vxe-table in ${file}`, () => {
      const content = readFileSync(resolve(process.cwd(), file), "utf8");

      expect(content).toContain("<vxe-table");
      expect(content).not.toContain("<el-table");
    });
  }
});
