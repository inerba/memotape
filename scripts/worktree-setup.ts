// Prepara un worktree di git per i sei controlli, riusando la build Rust del checkout principale.
//   bun scripts/worktree-setup.ts [branch di base]
// - scrive `.cargo/config.toml` (ignorato da git, quindi assente nel worktree) partendo da quello del
//   checkout principale, con `LOCALAPPDATA` assoluto e `target-dir` sulla sua `src-tauri\target`:
//   transcribe-cpp non si ricompila da capo (vedi docs/sviluppo/prerequisiti.md);
// - con un branch di base, si ferma se il worktree non ne discende;
// - lancia `bun install`.
// Le build Rust restano una alla volta: il worktree condivide `target` con il checkout principale.
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

const RIGA_LOCALAPPDATA = /^\s*LOCALAPPDATA\s*=/;
const SEZIONE_BUILD = /\n\[build\][^[]*/;

function git(...args: string[]): string {
  const r = spawnSync("git", args, { encoding: "utf8" });
  if (r.status !== 0) {
    throw new Error(`git ${args.join(" ")}: ${r.stderr.trim()}`);
  }
  return r.stdout.trim();
}

/** La config del worktree: quella principale con la cartella di build condivisa. */
export function configCondivisa(principale: string, target: string): string {
  const righe = principale
    .split("\n")
    .filter((r) => !RIGA_LOCALAPPDATA.test(r));
  const env = righe.findIndex((r) => r.trim() === "[env]");
  if (env === -1) {
    throw new Error("la config principale non ha la sezione [env]");
  }
  righe.splice(
    env + 1,
    0,
    `LOCALAPPDATA = { value = '${target}', force = true }`
  );
  const senzaBuild = righe.join("\n").replace(SEZIONE_BUILD, "\n");
  return `${senzaBuild.trimEnd()}\n\n[build]\ntarget-dir = '${target}'\n`;
}

if (import.meta.main) {
  const worktree = git("rev-parse", "--show-toplevel");
  const principale = dirname(
    resolve(git("rev-parse", "--path-format=absolute", "--git-common-dir"))
  );
  if (resolve(worktree) === resolve(principale)) {
    throw new Error(
      "sei nel checkout principale: lo script serve solo nei worktree"
    );
  }
  const [, , base] = process.argv;
  if (
    base &&
    spawnSync("git", ["merge-base", "--is-ancestor", base, "HEAD"]).status !== 0
  ) {
    throw new Error(
      `il worktree non discende da ${base}: riportalo lì con git reset --hard ${base}, poi rilancia`
    );
  }
  const sorgente = join(principale, ".cargo", "config.toml");
  if (!existsSync(sorgente)) {
    throw new Error(`manca ${sorgente}: vedi docs/sviluppo/prerequisiti.md`);
  }
  const target = join(principale, "src-tauri", "target");
  mkdirSync(join(worktree, ".cargo"), { recursive: true });
  writeFileSync(
    join(worktree, ".cargo", "config.toml"),
    configCondivisa(
      readFileSync(sorgente, "utf8").replaceAll("\r\n", "\n"),
      target
    )
  );
  const install = spawnSync("bun", ["install"], {
    cwd: worktree,
    stdio: "inherit",
  });
  if (install.status !== 0) {
    process.exit(install.status ?? 1);
  }
  console.log(`worktree pronto: build Rust in ${target}, una alla volta`);
}
