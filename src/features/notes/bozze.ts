/**
 * Le bozze delle Note: per Tape, il testo non ancora su disco di ogni Nota che ne ha uno. Vivono
 * nella route, non nella vista del Tape, che si rimonta: restano passando da una Nota o da una
 * scheda all'altra, e un'uscita dal Tape con bozze passa da «Salvare le modifiche alle Note?».
 */
export type Bozze = Readonly<Record<string, Readonly<Record<string, string>>>>;

export type BozzeAction =
  /** L'editor della Nota `nome` ha `testo`; su disco c'è `salvato` (`null`: la Nota non c'è). */
  | {
      type: "scritta";
      path: string;
      nome: string;
      testo: string;
      salvato: string | null;
    }
  /** Su disco c'è ora `testo` della Nota `nome`. */
  | { type: "salvata"; path: string; nome: string; testo: string }
  | { type: "rinominata"; path: string; nome: string; nuovo: string }
  | { type: "eliminata"; path: string; nome: string }
  /** Non salvare: le bozze del Tape si buttano. */
  | { type: "scartate"; path: string }
  | { type: "tapeMoved"; from: string; to: string }
  /** Il Tape non c'è più (nel Cestino o sparito dal disco). */
  | { type: "tapeGone"; path: string };

const NESSUNA: Readonly<Record<string, string>> = {};

/** Le bozze delle Note del Tape `path`, per nome: sempre lo stesso oggetto se non ce ne sono. */
export function bozzeDi(stato: Bozze, path: string): Record<string, string> {
  return stato[path] ?? NESSUNA;
}

export function bozze(stato: Bozze, action: BozzeAction): Bozze {
  switch (action.type) {
    case "scritta": {
      const { nome, path, salvato, testo } = action;
      const { [nome]: _, ...altre } = bozzeDi(stato, path);
      return conBozze(
        stato,
        path,
        testo === (salvato ?? "") ? altre : { ...altre, [nome]: testo }
      );
    }
    case "salvata": {
      const { nome, path, testo } = action;
      const { [nome]: bozza, ...altre } = bozzeDi(stato, path);
      return bozza === testo ? conBozze(stato, path, altre) : stato;
    }
    case "rinominata": {
      const { nome, nuovo, path } = action;
      const { [nome]: bozza, ...altre } = bozzeDi(stato, path);
      return bozza === undefined
        ? stato
        : conBozze(stato, path, { ...altre, [nuovo]: bozza });
    }
    case "eliminata": {
      const { [action.nome]: _, ...altre } = bozzeDi(stato, action.path);
      return conBozze(stato, action.path, altre);
    }
    case "scartate":
    case "tapeGone":
      return conBozze(stato, action.path, NESSUNA);
    case "tapeMoved":
      return conBozze(
        conBozze(stato, action.from, NESSUNA),
        action.to,
        bozzeDi(stato, action.from)
      );
    default:
      return action satisfies never;
  }
}

/**
 * Cosa fare prima di lasciare il Tape `mostrato` (`null`: non c'è un Tape al centro) per `verso`,
 * un altro Tape, o per un'altra vista, un'Attività, un Taglio o la chiusura della finestra: `vai`
 * senza bozze o riaprendo lo stesso Tape; con bozze `salva` con Salva da solo acceso, altrimenti
 * `chiedi` «Salvare le modifiche alle Note?».
 */
export function uscita(
  stato: Bozze,
  {
    mostrato,
    salvaDaSolo,
    verso,
  }: { mostrato: string | null; salvaDaSolo: boolean; verso?: string }
): "vai" | "salva" | "chiedi" {
  if (mostrato === null || verso === mostrato || !(mostrato in stato)) {
    return "vai";
  }
  return salvaDaSolo ? "salva" : "chiedi";
}

/** `stato` con `di` come bozze del Tape `path`; senza bozze il Tape sparisce. */
function conBozze(
  stato: Bozze,
  path: string,
  di: Record<string, string>
): Bozze {
  const { [path]: _, ...altri } = stato;
  return Object.keys(di).length > 0 ? { ...altri, [path]: di } : altri;
}
