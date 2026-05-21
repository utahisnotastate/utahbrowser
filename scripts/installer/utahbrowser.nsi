; Utah Browser — minimal NSIS installer template (requires NSIS 3.x on build machine)
; Build: makensis scripts/installer/utahbrowser.nsi

!include "MUI2.nsh"

Name "Utah Browser"
OutFile "UtahBrowserSetup.exe"
InstallDir "$APPDATA\UtahBrowser\app"
RequestExecutionLevel user

Page directory
Page instfiles
InstType "Full"

Section "Utah Browser"
  SetOutPath "$INSTDIR"
  File /r "..\..\dist\*.*"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  CreateShortcut "$DESKTOP\Utah Browser.lnk" "$INSTDIR\UtahBrowser.cmd"
  CreateShortcut "$SMPROGRAMS\Utah Browser.lnk" "$INSTDIR\UtahBrowser.cmd"
SectionEnd

Section "Uninstall"
  Delete "$DESKTOP\Utah Browser.lnk"
  Delete "$SMPROGRAMS\Utah Browser.lnk"
  RMDir /r "$INSTDIR"
SectionEnd
