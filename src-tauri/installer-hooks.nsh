; Hook dell'installer NSIS di Tauri (`bundle.windows.nsis.installerHooks`).

; `APP_UNASSOCIATE` rimette in `.bino` il valore salvato all'installazione: vuoto, o `Sbobino.Bino`
; se si è installato sopra un'installazione esistente (un aggiornamento). In entrambi i casi la
; chiave resta e punta a niente: si toglie, a meno che un altro programma non si sia associato.
!macro NSIS_HOOK_POSTUNINSTALL
  ReadRegStr $R0 SHCTX "Software\Classes\.bino" ""
  ${If} $R0 == ""
  ${OrIf} $R0 == "Sbobino.Bino"
    DeleteRegKey SHCTX "Software\Classes\.bino"
  ${EndIf}
!macroend
