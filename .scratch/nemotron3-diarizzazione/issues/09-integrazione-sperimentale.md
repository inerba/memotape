# 09: Integrare Nemotron 3 come opzione sperimentale

**What to build:** Completare la presentazione della funzione già implementata: scelta esplicita, Sortformer predefinito, avvisi sperimentali e GPU Vulkan consigliata per i Parlanti dal vivo. Nessuna garanzia di otto Parlanti o promozione automatica.

**Status:** done

**Autorizzazione:** «integrala allora», 6 ottobre 2026, dopo la proposta di adozione sperimentale.

- [x] Opzione e modello locale distinti dall’ASR, percorso esistente file/Tape/Registrazioni conservato.
- [x] Avviso sperimentale e indicazione GPU/CPU in Impostazioni, attribuzioni provvisorie e analisi finale esplicite.
- [x] Nota sui limiti nel menu Trascrivi visibile e accessibile, sei lingue aggiornate.
- [x] PRODUCT e ADR documentano adozione sperimentale distinta dalla validazione del ticket 08.
- [x] Sei controlli verdi, build frontend, binding rigenerati identici e due revisioni senza finding aperte.
- [x] Modifiche accumulate dei ticket precedenti preservate; nessun commit o pubblicazione.

## Comments

[Resoconto e limiti](../experimental-integration.md), [diff della sola integrazione](../experimental-integration.diff). 106 frontend, 235 Rust/12 ignored; collaudo UI nativo non attestato. Il ticket 08 resta ready-for-human.
