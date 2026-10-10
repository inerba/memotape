// Regole meccaniche di DESIGN.md sui sorgenti del frontend (fuori da components/ui, che è di shadcn):
// - colori solo dai token di global.css, mai scritti nel codice;
// - una sola ombra, `shadow-float` (The One Shadow Rule; `shadow-none` la toglie).
// Uso: bun scripts/check-design.ts (lo lancia anche `bun run check`).
import { readdirSync, readFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";

export interface Violazione {
  colonna: number;
  regola: string;
  riga: number;
  testo: string;
}

const COLORE = /#[0-9a-fA-F]{3,8}\b|\b(?:rgba?|hsla?|oklch|oklab|lab|lch)\(/g;
const OMBRA = /\bshadow-(?!float\b|none\b)[\w[\]().,/%-]+/g;

/** Le violazioni nel testo di un sorgente. */
export function violazioni(sorgente: string): Violazione[] {
  const trovate: Violazione[] = [];
  for (const [i, riga] of sorgente.split("\n").entries()) {
    for (const [regex, regola] of [
      [COLORE, "colore scritto nel codice: usa un token di global.css"],
      [OMBRA, "ombra diversa da shadow-float (The One Shadow Rule)"],
    ] as const) {
      for (const m of riga.matchAll(regex)) {
        trovate.push({
          colonna: (m.index ?? 0) + 1,
          regola,
          riga: i + 1,
          testo: m[0],
        });
      }
    }
  }
  return trovate;
}

const ESCLUSI = [/components[\\/]ui[\\/]/, /bindings\.ts$/, /\.test\.tsx?$/];
const TS = /\.tsx?$/;

function sorgenti(cartella: string): string[] {
  return readdirSync(cartella, { recursive: true, withFileTypes: true })
    .filter((f) => f.isFile() && TS.test(f.name))
    .map((f) => join(f.parentPath, f.name))
    .filter((p) => !ESCLUSI.some((e) => e.test(p)));
}

if (import.meta.main) {
  const root = resolve(import.meta.dir, "..");
  const righe = sorgenti(join(root, "src")).flatMap((file) =>
    violazioni(readFileSync(file, "utf8")).map(
      (v) =>
        `${relative(root, file)}:${v.riga}:${v.colonna} ${v.testo}: ${v.regola}`
    )
  );
  if (righe.length > 0) {
    console.error(righe.join("\n"));
    console.error(
      `\ncheck-design: ${righe.length} violazioni (vedi DESIGN.md)`
    );
    process.exit(1);
  }
}
