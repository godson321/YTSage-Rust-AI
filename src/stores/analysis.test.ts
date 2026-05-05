import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";
import { useAnalysisStore } from "./analysis";

beforeEach(() => {
  setActivePinia(createPinia());
});

describe("analysis store", () => {
  it("clears the current result", () => {
    const store = useAnalysisStore();
    store.result = { jobId: "job-1", items: [] };
    store.loading = true;

    store.clear();

    expect(store.result).toBeNull();
    expect(store.loading).toBe(false);
  });
});
