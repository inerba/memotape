/** `Microfono (Anker PowerConf C200)`: il tipo generico di Windows e, tra parentesi, il dispositivo. */
const GENERIC_PREFIX = /^[^()]+\((.+)\)$/;

/** Il nome di un dispositivo senza il tipo generico che Windows mette davanti, per il menu. */
export function shortDeviceName(name: string): string {
  return name.match(GENERIC_PREFIX)?.[1]?.trim() || name;
}
