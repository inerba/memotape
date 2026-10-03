# 09: Audio di sistema ed Entrambi

**What to build:** l'utente registra l'Audio di sistema (loopback del dispositivo di uscita) oppure Entrambi, microfono e sistema mixati in un solo file. Vede un indicatore di livello per ciascuna sorgente. Anche quando il PC è in silenzio, timer e file restano allineati.

**Blocked by:** 08 (Registrazione dal microfono)

**Status:** done

- [x] Loopback WASAPI aprendo uno stream di input sul dispositivo di uscita, con la config presa da `default_output_config()`
- [x] Il mixer posiziona i blocchi per timestamp QPC rispetto all'inizio della sessione e mette silenzio nei buchi. Somma con clamp. In stereo il microfono mono va su entrambi i canali
- [x] Sorgente di registrazione (Microfono, Audio di sistema, Entrambi) e dispositivo di uscita scelti nelle impostazioni, con default di sistema
- [x] Livello per sorgente nell'evento `recording-tick`
- [x] Test del mixer con buffer sintetici con timestamp: buchi del loopback, sovrapposizione, pause
- [x] Verificare su Windows 11 e annotare in `AGENTS.md`: il comportamento del loopback a riproduzione ferma e il drift in una sessione lunga
- [x] `AGENTS.md` / `PRODUCT.md` aggiornati, controlli verdi
