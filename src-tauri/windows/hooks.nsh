; BeamLink installer hooks (Tauri NSIS).
;
; After the files are copied, BeamLink sets itself up headlessly: it finds
; BeamNG.drive and its user folder, downloads the official BeamMP-Launcher
; and the BeamMP client mod (checksum-verified), and puts the client mod and
; the BeamLink companion mod in <user folder>\current\mods. If the game is not
; found, the app's own setup screen takes over on first launch.

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Installing BeamMP and the BeamLink companion into BeamNG.drive..."
  nsExec::ExecToLog '"$INSTDIR\${MAINBINARYNAME}.exe" --setup'
  Pop $0
  ${If} $0 != 0
    DetailPrint "BeamNG.drive setup will finish when BeamLink first opens."
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Removing BeamLink's mods from BeamNG.drive..."
  nsExec::ExecToLog '"$INSTDIR\${MAINBINARYNAME}.exe" --remove-mods'
  Pop $0
!macroend
