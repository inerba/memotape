import grande from "../../docs/brand/svg/memotape-simbolo.svg";
import grandeNegativo from "../../docs/brand/svg/memotape-simbolo-negativo.svg";
import piccolo from "../../docs/brand/svg/memotape-simbolo-piccolo.svg";
import piccoloNegativo from "../../docs/brand/svg/memotape-simbolo-piccolo-negativo.svg";

/**
 * Il simbolo di Memotape: il mozzo di una bobina visto da vicino, con le strisce di una musicassetta,
 * su un quadrato salvia. Sono le SVG di `docs/brand/svg` (generate da `docs/brand/genera.py`); nel
 * tema scuro la bobina è bruna, per staccarsi dal fondo. I colori sono fissi, non token (DESIGN.md).
 */
export function BrandMark({
  className,
  small = false,
}: {
  className?: string;
  /** Sotto i 48 px: una striscia sola e il mozzo più grande. */
  small?: boolean;
}) {
  const [light, dark] = small
    ? [piccolo, piccoloNegativo]
    : [grande, grandeNegativo];
  return (
    <span className={`pointer-events-none block shrink-0 ${className ?? ""}`}>
      <img
        alt=""
        className="block size-full dark:hidden"
        draggable={false}
        height={256}
        src={light}
        width={256}
      />
      <img
        alt=""
        className="hidden size-full dark:block"
        draggable={false}
        height={256}
        src={dark}
        width={256}
      />
    </span>
  );
}
