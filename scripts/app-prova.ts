// Avvia l'app di debug isolata: cartella dati di prova, WebView a parte e una porta CDP libera.
//   bun run app:prova              avvia e stampa porta CDP, pid e cartelle
//   bun run app:prova stop <porta>  chiude ciò che ha avviato e cancella le sue cartelle
// Vedi docs/sviluppo/cartella-dati-di-prova.md.
import { spawn, spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { createConnection } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { setTimeout } from "node:timers/promises";
import { valuta } from "./cdp.ts";

const ROOT = resolve(import.meta.dir, "..");
const EXE = join(ROOT, "src-tauri", "target", "debug", "memotape.exe");
const VITE_PORT = 1420;
// Fuori dalla 9222, quella che si usava a mano e che altre sessioni possono tenere.
const CDP_FIRST = 9300;
const CDP_COUNT = 100;

interface Stato {
  app: number;
  cartella: string;
  porta: number;
  vite: number | null;
}

const cartellaDi = (porta: number) => join(tmpdir(), `memotape-prova-${porta}`);

function risponde(host: string, port: number): Promise<boolean> {
  return new Promise((done) => {
    const socket = createConnection({ host, port });
    socket.once("connect", () => {
      socket.destroy();
      done(true);
    });
    socket.once("error", () => done(false));
  });
}

// Vite ascolta su `localhost`, che può essere solo IPv6.
async function inUso(port: number): Promise<boolean> {
  return (await risponde("127.0.0.1", port)) || (await risponde("::1", port));
}

// Riprova ogni 250 ms, in sequenza, finché la condizione vale o scade il tempo.
async function aspetta(
  condizione: () => Promise<boolean>,
  secondi: number
): Promise<boolean> {
  if (await condizione()) {
    return true;
  }
  if (secondi <= 0) {
    return false;
  }
  await setTimeout(250);
  return aspetta(condizione, secondi - 0.25);
}

// Le porte si provano una alla volta, partendo da un punto a caso dell'intervallo.
async function portaLibera(
  start = Math.floor(Math.random() * CDP_COUNT),
  provate = 0
): Promise<number> {
  if (provate === CDP_COUNT) {
    throw new Error(
      `nessuna porta CDP libera tra ${CDP_FIRST} e ${CDP_FIRST + CDP_COUNT - 1}`
    );
  }
  const porta = CDP_FIRST + ((start + provate) % CDP_COUNT);
  if (!((await inUso(porta)) || existsSync(cartellaDi(porta)))) {
    return porta;
  }
  return portaLibera(start, provate + 1);
}

function staccato(
  cmd: string,
  args: string[],
  cwd: string,
  env: NodeJS.ProcessEnv
) {
  const child = spawn(cmd, args, { cwd, detached: true, env, stdio: "ignore" });
  child.unref();
  if (child.pid === undefined) {
    throw new Error(`${cmd} non è partito`);
  }
  return child.pid;
}

async function avvia() {
  if (!existsSync(EXE)) {
    throw new Error(
      `manca ${EXE}: compila prima con cargo build --manifest-path src-tauri/Cargo.toml`
    );
  }
  const porta = await portaLibera();
  const cartella = cartellaDi(porta);
  mkdirSync(join(cartella, "dati"), { recursive: true });

  let vite: number | null = null;
  if (await inUso(VITE_PORT)) {
    console.warn(
      `Vite risponde già sulla ${VITE_PORT}: lo uso, ma può servire il frontend di un altro checkout.`
    );
  } else {
    vite = staccato("bun", ["run", "dev"], ROOT, process.env);
    if (!(await aspetta(() => inUso(VITE_PORT), 30))) {
      throw new Error(`Vite non risponde sulla ${VITE_PORT} dopo 30 s`);
    }
  }

  const app = staccato(
    join("target", "debug", "memotape.exe"),
    [],
    join(ROOT, "src-tauri"),
    {
      ...process.env,
      MEMOTAPE_DATA_DIR: join(cartella, "dati"),
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${porta}`,
      WEBVIEW2_USER_DATA_FOLDER: join(cartella, "webview"),
    }
  );
  const stato: Stato = { app, cartella, porta, vite };
  writeFileSync(join(cartella, "stato.json"), JSON.stringify(stato, null, 2));
  if (!(await aspetta(() => inUso(porta), 30))) {
    throw new Error(
      `la porta CDP ${porta} non risponde dopo 30 s (app pid ${app})`
    );
  }
  // Un exe compilato prima di MEMOTAPE_DATA_DIR userebbe i dati veri: l'indice di prova lo smaschera.
  const indice = join(cartella, "dati", "indice");
  if (!(await aspetta(() => Promise.resolve(existsSync(indice)), 10))) {
    chiudi(app);
    chiudi(vite);
    throw new Error(
      `l'app non usa la cartella di prova (manca ${indice}): ricompila con cargo build --manifest-path src-tauri/Cargo.toml`
    );
  }
  if (!(await montaFrontend(porta))) {
    throw new Error(
      `il frontend non si monta (porta ${porta}): chiudi con bun run app:prova stop ${porta}`
    );
  }
  console.log(JSON.stringify(stato, null, 2));
  console.log(
    `Pilota l'app con: bun scripts/cdp.ts ${porta} eval "document.title"`
  );
  console.log(`Chiudi con: bun run app:prova stop ${porta}`);
}

const MONTATA = "document.getElementById('root')?.childElementCount > 0";

// Un Vite appena avviato può ottimizzare le dipendenze per più di un minuto, e la prima pagina resta
// sul preload: dopo 20 s la si ricarica una volta, poi si aspetta fino a 3 minuti.
async function montaFrontend(porta: number): Promise<boolean> {
  const montata = async () => (await cdp(porta, MONTATA)) === true;
  if (await aspetta(montata, 20)) {
    return true;
  }
  console.warn("Il frontend non è ancora montato: ricarico e aspetto Vite…");
  await cdp(porta, "location.reload()");
  return aspetta(montata, 180);
}

// Il valore di `js` nella pagina; `null` se la pagina non risponde ancora.
async function cdp(porta: number, js: string): Promise<unknown> {
  try {
    return await valuta(porta, js);
  } catch {
    return null;
  }
}

// L'albero intero: `bun run dev` lancia Vite come processo figlio.
function chiudi(pid: number | null) {
  if (pid !== null) {
    spawnSync("taskkill", ["/PID", String(pid), "/T", "/F"], {
      stdio: "ignore",
    });
  }
}

async function ferma(porta: number) {
  const cartella = cartellaDi(porta);
  const file = join(cartella, "stato.json");
  if (!existsSync(file)) {
    throw new Error(`nessuna app di prova avviata sulla porta ${porta}`);
  }
  const stato = JSON.parse(readFileSync(file, "utf8")) as Stato;
  chiudi(stato.app);
  chiudi(stato.vite);
  // Se l'app era già uscita, i processi WebView2 restano orfani e tengono aperta la cartella.
  spawnSync(
    "powershell",
    [
      "-NoProfile",
      "-Command",
      `Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'" | Where-Object { $_.CommandLine -like '*${cartella}*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }`,
    ],
    { stdio: "ignore" }
  );
  // La WebView rilascia i suoi file qualche istante dopo la chiusura dei processi.
  const cancellata = await aspetta(() => {
    try {
      rmSync(cartella, { force: true, recursive: true });
      return Promise.resolve(true);
    } catch {
      return Promise.resolve(false);
    }
  }, 10);
  console.log(
    cancellata
      ? `chiusa e cancellata ${cartella}`
      : `chiusa; ${cartella} non si cancella ancora`
  );
}

const [comando, argomento] = process.argv.slice(2);
if (comando === "stop") {
  await ferma(Number(argomento));
} else {
  await avvia();
}
