; PYRAX Desktop NSIS Installer Hooks
; Auto-installs Visual C++ Redistributable if not present

!macro NSIS_HOOK_PREINSTALL
  ; Show installation start message
  DetailPrint "Preparing PYRAX Desktop installation..."
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Check if Visual C++ 2015-2022 Redistributable x64 is installed
  ReadRegDWord $0 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Installed"
  ${If} $0 == 1
    DetailPrint "Visual C++ Redistributable (x64) already installed"
    Goto vcredist_x64_done
  ${EndIf}
  
  ; Try alternative registry location
  ReadRegDWord $0 HKLM "SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Installed"
  ${If} $0 == 1
    DetailPrint "Visual C++ Redistributable (x64) already installed"
    Goto vcredist_x64_done
  ${EndIf}
  
  ; Install x64 from bundled EXE if not installed
  ${If} ${FileExists} "$INSTDIR\resources\vc_redist.x64.exe"
    DetailPrint "Installing Visual C++ Redistributable (x64)..."
    CopyFiles "$INSTDIR\resources\vc_redist.x64.exe" "$TEMP\vc_redist.x64.exe"
    ExecWait '"$TEMP\vc_redist.x64.exe" /install /passive /norestart' $0
    ${If} $0 == 0
      DetailPrint "Visual C++ Redistributable (x64) installed successfully"
    ${ElseIf} $0 == 1638
      ; 1638 = Another version already installed
      DetailPrint "Visual C++ Redistributable (x64) - compatible version exists"
    ${ElseIf} $0 == 3010
      ; 3010 = Success, reboot required (but we use /norestart)
      DetailPrint "Visual C++ Redistributable (x64) installed (reboot may be needed)"
    ${Else}
      MessageBox MB_ICONEXCLAMATION "Visual C++ installation returned code $0. If PYRAX Desktop doesn't start, please install Visual C++ Redistributable manually from Microsoft."
    ${EndIf}
    Delete "$TEMP\vc_redist.x64.exe"
    Delete "$INSTDIR\resources\vc_redist.x64.exe"
  ${EndIf}
  
  vcredist_x64_done:
  
  ; Check if Visual C++ 2015-2022 Redistributable x86 is installed (needed for some dependencies)
  ReadRegDWord $0 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x86" "Installed"
  ${If} $0 == 1
    DetailPrint "Visual C++ Redistributable (x86) already installed"
    Goto vcredist_x86_done
  ${EndIf}
  
  ReadRegDWord $0 HKLM "SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\x86" "Installed"
  ${If} $0 == 1
    DetailPrint "Visual C++ Redistributable (x86) already installed"
    Goto vcredist_x86_done
  ${EndIf}
  
  ; Install x86 from bundled EXE if not installed
  ${If} ${FileExists} "$INSTDIR\resources\vc_redist.x86.exe"
    DetailPrint "Installing Visual C++ Redistributable (x86)..."
    CopyFiles "$INSTDIR\resources\vc_redist.x86.exe" "$TEMP\vc_redist.x86.exe"
    ExecWait '"$TEMP\vc_redist.x86.exe" /install /passive /norestart' $0
    ${If} $0 == 0
      DetailPrint "Visual C++ Redistributable (x86) installed successfully"
    ${ElseIf} $0 == 1638
      DetailPrint "Visual C++ Redistributable (x86) - compatible version exists"
    ${ElseIf} $0 == 3010
      DetailPrint "Visual C++ Redistributable (x86) installed (reboot may be needed)"
    ${Else}
      MessageBox MB_ICONEXCLAMATION "Visual C++ (x86) installation returned code $0. If PYRAX Desktop doesn't start, please install Visual C++ Redistributable manually from Microsoft."
    ${EndIf}
    Delete "$TEMP\vc_redist.x86.exe"
    Delete "$INSTDIR\resources\vc_redist.x86.exe"
  ${EndIf}
  
  vcredist_x86_done:
  
  DetailPrint "PYRAX Desktop installation complete!"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Preparing to uninstall PYRAX Desktop..."
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DetailPrint "PYRAX Desktop uninstalled successfully"
!macroend
