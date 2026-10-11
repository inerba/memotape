/**
 * Le bozze delle Note: per Tape, il testo non ancora su disco di ogni Nota che ne ha uno, con il
 * testo su disco da cui parte. Vivono nella route, non nella vista del Tape, che si rimonta:
 * restano passando da una Nota o da una scheda all'altra, e un'uscita dal Tape con bozze passa da
 * «Salvare le modifiche alle Note?».
 */
export interface BozzeDelTape {
  /** Il testo su disco da cui parte ogni bozza (`""` se la Nota non c'è): l'atteso del salvataggio. */
  readonly attesi: Readonly<Record<string, string>>;
  /** Il testo non ancora su disco, per nome. */
  readonly testi: Readonly<Record<string, string>>;
}

export type Bozze = Readonly<Record<string, BozzeDelTape>>;

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
const VUOTO: BozzeDelTape = { attesi: NESSUNA, testi: NESSUNA };

/** Le bozze delle Note del Tape `path`, per nome: sempre lo stesso oggetto se non ce ne sono. */
export function bozzeDi(stato: Bozze, path: string): Record<string, string> {
  return stato[path]?.testi ?? NESSUNA;
}

export function bozze(stato: Bozze, action: BozzeAction): Bozze {
  switch (action.type) {
    case "scritta": {
      const { nome, path, salvato, testo } = action;
      const del = stato[path] ?? VUOTO;
      const atteso = salvato ?? "";
      return conTape(
        stato,
        path,
        testo === atteso
          ? senza(del, nome)
          : {
              attesi: { ...del.attesi, [nome]: atteso },
              testi: { ...del.testi, [nome]: testo },
            }
      );
    }
    case "salvata": {
      const { nome, path, testo } = action;
      const del = stato[path];
      if (del?.testi[nome] === undefined) {
        return stato;
      }
      // Il testo scritto intanto resta, e ora parte da quello salvato.
      return conTape(
        stato,
        path,
        del.testi[nome] === testo
          ? senza(del, nome)
          : { ...del, attesi: { ...del.attesi, [nome]: testo } }
      );
    }
    case "rinominata": {
      const { nome, nuovo, path } = action;
      const del = stato[path];
      if (del?.testi[nome] === undefined) {
        return stato;
      }
      const resto = senza(del, nome);
      return conTape(stato, path, {
        attesi: { ...resto.attesi, [nuovo]: del.attesi[nome] ?? "" },
        testi: { ...resto.testi, [nuovo]: del.testi[nome] },
      });
    }
    case "eliminata":
      return conTape(
        stato,
        action.path,
        senza(stato[action.path] ?? VUOTO, action.nome)
      );
    case "scartate":
    case "tapeGone":
      return conTape(stato, action.path, VUOTO);
    case "tapeMoved":
      return conTape(
        conTape(stato, action.from, VUOTO),
        action.to,
        stato[action.from] ?? VUOTO
      );
    default:
      return action satisfies never;
  }
}

/**
 * Cosa fare prima di lasciare il Tape `mostrato` (`null`: non c'è un Tape al centro) per `verso`,
 * un altro Tape, o per un'altra vista, un'Attività o un Taglio: `vai` senza bozze o riaprendo lo
 * stesso Tape; con bozze `salva` con Salva da solo acceso, altrimenti `chiedi` «Salvare le modifiche
 * alle Note?». La `chiusura` della finestra guarda le bozze di ogni Tape, anche di quelli non più
 * mostrati (il Tape consultato che un'Attività finita ha chiuso).
 */
export function uscita(
  stato: Bozze,
  {
    chiusura = false,
    mostrato,
    salvaDaSolo,
    verso,
  }: {
    chiusura?: boolean;
    mostrato: string | null;
    salvaDaSolo: boolean;
    verso?: string;
  }
): "vai" | "salva" | "chiedi" {
  const conBozze = chiusura
    ? Object.keys(stato).length > 0
    : mostrato !== null && verso !== mostrato && mostrato in stato;
  if (!conBozze) {
    return "vai";
  }
  return salvaDaSolo ? "salva" : "chiedi";
}

/** Una bozza da salvare senza la vista del suo Tape. */
export interface BozzaDaSalvare {
  atteso: string;
  nome: string;
  path: string;
  testo: string;
}

/**
 * Le bozze di ogni Tape tranne `tranne` (quello mostrato, la cui vista le salva da sé), con il testo
 * su disco da cui partono: la route le salva da sé chiudendo la finestra.
 */
export function bozzeDaSalvare(
  stato: Bozze,
  tranne: string | null
): BozzaDaSalvare[] {
  return Object.entries(stato).flatMap(([path, { attesi, testi }]) =>
    path === tranne
      ? []
      : Object.entries(testi).map(([nome, testo]) => ({
          atteso: attesi[nome] ?? "",
          nome,
          path,
          testo,
        }))
  );
}

/** `del` senza la bozza della Nota `nome`. */
function senza(del: BozzeDelTape, nome: string): BozzeDelTape {
  const { [nome]: _, ...testi } = del.testi;
  const { [nome]: __, ...attesi } = del.attesi;
  return { attesi, testi };
}

/** `stato` con `del` come bozze del Tape `path`; senza bozze il Tape sparisce. */
function conTape(stato: Bozze, path: string, del: BozzeDelTape): Bozze {
  const { [path]: _, ...altri } = stato;
  return Object.keys(del.testi).length > 0 ? { ...altri, [path]: del } : altri;
}
