import { readdir } from "node:fs/promises";
import { join, relative } from "node:path";

const root = process.cwd();
const baseline = "C:/Users/inerba/.codex/visualizations/2026/10/06/01a11078-9a68-7d91-b4aa-1d056bb836f9/qol-baseline";
const changed: string[] = [];
let diff = "";
async function walk(dir: string) {
  for (const entry of await readdir(join(root, dir), { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) { await walk(path); }
    else { await compare(path); }
  }
}
async function compare(path: string) {
  const old = Bun.file(join(baseline, path));
  const current = Bun.file(join(root, path));
  if (await old.exists() && (await old.text()).replace(/\r\n/g, "\n") === (await current.text()).replace(/\r\n/g, "\n")) { return; }
  changed.push(path);
  const result = Bun.spawnSync(["git", "diff", "--no-index", "--", await old.exists() ? join(baseline, path) : "NUL", join(root, path)]);
  diff += result.stdout.toString();
}
await walk("src");
await walk("src-tauri/src");
await compare("biome.jsonc");
await compare("DESIGN.md");
await compare("docs/adr/0017-correzioni-manuali-della-trascrizione.md");
await compare(".scratch/trascrizione-qol/spec.md");
await Bun.write(".scratch/trascrizione-qol/implementation.diff", diff);
await Bun.write(".scratch/trascrizione-qol/changed-files.json", JSON.stringify(changed, null, 2));
console.log(changed.join("\n"));
