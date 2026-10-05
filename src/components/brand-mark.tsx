/**
 * Il simbolo di Memotape: le due bobine di una musicassetta unite dal nastro, su un quadrato salvia.
 * Le geometrie sono quelle di `docs/brand/svg` (`memotape-simbolo-piccolo` e `memotape-simbolo`). I
 * colori sono fissi, non token: il logo non cambia con il tema (DESIGN.md).
 */

const SAGE = "#576b3c";
const INK = "#241e1a";
const PAPER = "#fcfaf6";

/** Sotto i 48 px: quadrato a tutta tela, nastro più grande, mozzi tondi senza denti. */
const SMALL = {
  hubs: "M51,122A19,19 0 1 1 89,122A19,19 0 1 1 51,122Z M167,122A19,19 0 1 1 205,122A19,19 0 1 1 167,122Z",
  square:
    "M44,0H212A44,44 0 0 1 256,44V212A44,44 0 0 1 212,256H44A44,44 0 0 1 0,212V44A44,44 0 0 1 44,0Z",
  tape: "M116.65,140A50,50 0 1 0 70,172L186,172A50,50 0 1 0 139.35,140Z",
};

const hub = (x: number) =>
  `M${x - 3.5},106.34L${x - 3.5},114.63A10,10 0 0 1 ${x + 3.5},114.63L${x + 3.5},106.34A18,18 0 0 1 ${x + 13.54},112.14L${x + 6.36},116.29A10,10 0 0 1 ${x + 9.86},122.35L${x + 17.04},118.2A18,18 0 0 1 ${x + 17.04},129.8L${x + 9.86},125.65A10,10 0 0 1 ${x + 6.36},131.71L${x + 13.54},135.86A18,18 0 0 1 ${x + 3.5},141.66L${x + 3.5},133.37A10,10 0 0 1 ${x - 3.5},133.37L${x - 3.5},141.66A18,18 0 0 1 ${x - 13.54},135.86L${x - 6.36},131.71A10,10 0 0 1 ${x - 9.86},125.65L${x - 17.04},129.8A18,18 0 0 1 ${x - 17.04},118.2L${x - 9.86},122.35A10,10 0 0 1 ${x - 6.36},116.29L${x - 13.54},112.14A18,18 0 0 1 ${x - 3.5},106.34Z`;

/** Da 48 px in su: i mozzi dentati. */
const LARGE = {
  hubs: `${hub(78)} ${hub(178)}`,
  square:
    "M52,16H204A36,36 0 0 1 240,52V204A36,36 0 0 1 204,240H52A36,36 0 0 1 16,204V52A36,36 0 0 1 52,16Z",
  tape: "M115.47,138A40,40 0 1 0 78,164L178,164A40,40 0 1 0 140.53,138Z",
};

export function BrandMark({
  className,
  small = false,
}: {
  className?: string;
  small?: boolean;
}) {
  const shape = small ? SMALL : LARGE;
  return (
    <svg
      aria-hidden
      className={`pointer-events-none shrink-0 ${className ?? ""}`}
      viewBox="0 0 256 256"
    >
      <path d={shape.square} fill={SAGE} />
      <path d={shape.tape} fill={INK} />
      <path d={shape.hubs} fill={PAPER} />
    </svg>
  );
}
