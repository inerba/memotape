import type { SVGProps } from "react";

type IconProps = Omit<SVGProps<SVGSVGElement>, "viewBox">;

/** Il Microfono pieno: la capsula piena, l'arco e l'asta a tratto, come lucide a 24 px. */
export function MicFill(props: IconProps) {
  return (
    <svg
      aria-hidden
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      viewBox="0 0 24 24"
      {...props}
    >
      <rect
        fill="currentColor"
        height="12.5"
        rx="3.75"
        stroke="none"
        width="7.5"
        x="8.25"
        y="1.75"
      />
      <path d="M5.25 10.75a6.75 6.75 0 0 0 13.5 0" />
      <path d="M12 17.5v3.75M8.75 21.25h6.5" />
    </svg>
  );
}

/** L'Audio di sistema pieno: la cassa piena con tweeter e woofer ritagliati. */
export function SpeakerFill(props: IconProps) {
  return (
    <svg aria-hidden fill="currentColor" viewBox="0 0 24 24" {...props}>
      <path
        d="M7.25 1.75h9.5a3.25 3.25 0 0 1 3.25 3.25v14a3.25 3.25 0 0 1-3.25 3.25h-9.5A3.25 3.25 0 0 1 4 19V5a3.25 3.25 0 0 1 3.25-3.25ZM12 9.4a4.6 4.6 0 1 0 0 9.2a4.6 4.6 0 1 0 0-9.2ZM12 4.6a1.45 1.45 0 1 0 0 2.9a1.45 1.45 0 1 0 0-2.9ZM12 12.4a1.6 1.6 0 1 1 0 3.2a1.6 1.6 0 1 1 0-3.2Z"
        fillRule="evenodd"
      />
    </svg>
  );
}

/** Il segno di divieto sopra l'icona di un Ingresso in Muto. */
export function BanMark(props: IconProps) {
  return (
    <svg
      aria-hidden
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeWidth={2.25}
      viewBox="0 0 24 24"
      {...props}
    >
      <circle cx="12" cy="12" r="10" />
      <path d="m4.9 4.9 14.2 14.2" />
    </svg>
  );
}
