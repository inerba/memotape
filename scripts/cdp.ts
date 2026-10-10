// Client CDP minimo per la WebView2 dell'app di debug (vedi docs/sviluppo/verifica-manuale.md).
//   bun scripts/cdp.ts <porta> eval "<js>"                       valuta e stampa il risultato
//   bun scripts/cdp.ts <porta> shot <file.png> [w h] [dark|light]  screenshot, con larghezza o tema emulati
// Da un altro script: `import { valuta } from "./cdp.ts"`, senza passare il JavaScript da riga di
// comando (su Windows le virgolette e `>` non arrivano intatti a un processo figlio).
import { writeFileSync } from "node:fs";
import { setTimeout } from "node:timers/promises";

type Invia = (
  method: string,
  params?: object
) => Promise<Record<string, unknown>>;

// Apre la pagina dell'app sulla porta, esegue `lavoro` e chiude la connessione.
async function conPagina<T>(
  porta: number | string,
  lavoro: (invia: Invia) => Promise<T>
): Promise<T> {
  const pagine = (await (
    await fetch(`http://127.0.0.1:${porta}/json`)
  ).json()) as { type: string; url: string; webSocketDebuggerUrl: string }[];
  const pagina = pagine.find(
    (p) => p.type === "page" && p.url.includes("localhost")
  );
  if (!pagina) {
    throw new Error(`nessuna pagina dell'app sulla porta ${porta}`);
  }
  const ws = new WebSocket(pagina.webSocketDebuggerUrl);
  await new Promise((aperto) => ws.addEventListener("open", aperto));
  let id = 0;
  const invia: Invia = (method, params = {}) =>
    new Promise((risolvi, rifiuta) => {
      id += 1;
      const mio = id;
      const ascolta = (e: MessageEvent) => {
        const msg = JSON.parse(String(e.data));
        if (msg.id === mio) {
          ws.removeEventListener("message", ascolta);
          if (msg.error) {
            rifiuta(new Error(JSON.stringify(msg.error)));
          } else {
            risolvi(msg.result);
          }
        }
      };
      ws.addEventListener("message", ascolta);
      ws.send(JSON.stringify({ id: mio, method, params }));
    });
  try {
    return await lavoro(invia);
  } finally {
    ws.close();
  }
}

/** Il valore di `js` valutato nella pagina, o i dettagli dell'eccezione. */
export function valuta(porta: number | string, js: string): Promise<unknown> {
  return conPagina(porta, async (invia) => {
    const r = (await invia("Runtime.evaluate", {
      awaitPromise: true,
      expression: js,
      returnByValue: true,
    })) as { result?: { value?: unknown }; exceptionDetails?: unknown };
    return r.exceptionDetails ?? r.result?.value ?? null;
  });
}

/** Screenshot della pagina in `file`, con larghezza e tema emulati se dati. */
export function fotografa(
  porta: number | string,
  file: string,
  dimensioni?: { altezza: number; larghezza: number },
  tema?: string
): Promise<void> {
  return conPagina(porta, async (invia) => {
    if (dimensioni) {
      await invia("Emulation.setDeviceMetricsOverride", {
        deviceScaleFactor: 1,
        height: dimensioni.altezza,
        mobile: false,
        width: dimensioni.larghezza,
      });
    }
    if (tema) {
      await invia("Emulation.setEmulatedMedia", {
        features: [{ name: "prefers-color-scheme", value: tema }],
      });
    }
    await setTimeout(400);
    const { data } = (await invia("Page.captureScreenshot", {
      format: "png",
    })) as { data: string };
    writeFileSync(file, Buffer.from(data, "base64"));
    await invia("Emulation.clearDeviceMetricsOverride");
    await invia("Emulation.setEmulatedMedia", { features: [] });
  });
}

if (import.meta.main) {
  const [porta, comando, argomento, larghezza, altezza, tema] =
    process.argv.slice(2);
  if (!(porta && comando && argomento)) {
    throw new Error(
      'uso: bun scripts/cdp.ts <porta> eval "<js>" | shot <file.png> [w h] [dark|light]'
    );
  }
  if (comando === "eval") {
    console.log(JSON.stringify(await valuta(porta, argomento), null, 1));
  } else if (comando === "shot") {
    const dimensioni = larghezza
      ? { altezza: Number(altezza), larghezza: Number(larghezza) }
      : undefined;
    await fotografa(porta, argomento, dimensioni, tema);
    console.log(argomento);
  }
}
