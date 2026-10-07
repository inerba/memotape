import { expect, test } from "bun:test";
import type { AppError } from "@/bindings";
import { recordAfterSettings } from "./start";

test("Annulla durante il salvataggio risponde subito e impedisce la chiamata record tardiva", async () => {
  const saving = Promise.withResolvers<AppError | null>();
  const controller = new AbortController();
  let calls = 0;
  const result = recordAfterSettings(
    () => saving.promise,
    () => {
      calls += 1;
      return Promise.resolve({
        error: { code: "microphoneMissing" },
        status: "error",
      });
    },
    controller.signal
  );
  controller.abort();
  expect(await result).toEqual({
    error: { code: "cancelled" },
    status: "error",
  });
  expect(calls).toBe(0);
  saving.resolve(null);
  await saving.promise;
  expect(calls).toBe(0);
});
