# 01: Lock su file tra processi per le riscritture del Tape

**What to build:** prefactor senza cambi di comportamento visibili. Il lock delle riscritture del Tape smette di essere un mutex interno al processo dell'app e diventa un lock esclusivo su file nella cartella `.memotape` della Libreria (per un Tape fuori dalla Libreria resta il lock di processo), così il server MCP e l'app non si sovrappongono. Rinomina e Cestino della Libreria si dividono in operazione sul file e riallineamento dell'indice, così il server può usare la prima senza scrivere l'indice. Vedi `../spec.md` (Implementation Decisions: lock su file, il server non scrive mai l'indice) e ADR-0031.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Ogni riscrittura del Tape (correzioni, unione, nomi, data, Taglio, ritrascrizione, Forma d'onda) prende il lock su file quando il Tape sta nella Libreria
- [ ] Un test prova che due riscritture concorrenti dello stesso Tape non perdono nessuna delle due modifiche (come `correzioni_contemporanee_dello_stesso_tape_restano_tutte`) anche passando dal lock su file
- [ ] Rinomina e Cestino del Tape esistono anche senza `sync`; i comandi dell'app continuano a fare `sync` come prima
- [ ] I sei controlli verdi
