import { expect, test } from "bun:test";
import { spostaTempi, tempiNellaNota } from "@/features/notes/tempi";
// Gli stessi casi li prova `tempi_nota.rs` nel backend: la regola è una sola.
import casi from "@/features/notes/tempi.casi.json";

for (const { caso, tempi, testo } of casi.riconosce) {
  test(`riconosce i Tempi nella Nota: ${caso}`, () => {
    expect(
      tempiNellaNota(testo).map(({ a, da, ms }) => [testo.slice(da, a), ms])
    ).toEqual(tempi);
  });
}

for (const { atteso, caso, tagli, testo } of casi.sposta) {
  test(`sposta i Tempi nella Nota dopo un Taglio: ${caso}`, () => {
    expect(
      spostaTempi(
        testo,
        tagli.map(([inizioMs = 0, fineMs = 0]) => ({ fineMs, inizioMs }))
      )
    ).toBe(atteso);
  });
}
