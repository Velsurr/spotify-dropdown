Unicode True
!include "MUI2.nsh"

!ifndef APP_VERSION
  !error "APP_VERSION is required"
!endif
!ifndef PROJECT_ROOT
  !error "PROJECT_ROOT is required"
!endif

Name "Spotify Dropdown ${APP_VERSION}"
!ifdef QA_ONLY
  OutFile "${PROJECT_ROOT}\artifacts\SpotifyDropdown-Setup-QA.exe"
  InstallDir "${PROJECT_ROOT}\artifacts\installer-qa"
!else
  OutFile "${PROJECT_ROOT}\dist\v${APP_VERSION}\SpotifyDropdown-Setup-${APP_VERSION}.exe"
  InstallDir "$LOCALAPPDATA\Programs\SpotifyDropdown"
!endif
RequestExecutionLevel user
SetCompressor /SOLID lzma
VIProductVersion "${APP_VERSION}.0"
VIAddVersionKey /LANG=1046 "ProductName" "Spotify Dropdown"
VIAddVersionKey /LANG=1046 "FileDescription" "Instalador do Spotify Dropdown"
VIAddVersionKey /LANG=1046 "FileVersion" "${APP_VERSION}"
VIAddVersionKey /LANG=1046 "ProductVersion" "${APP_VERSION}"
VIAddVersionKey /LANG=1046 "LegalCopyright" "Projeto pessoal; não afiliado ao Spotify."

!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_RUN "$INSTDIR\Spotify Dropdown.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Abrir Spotify Dropdown"
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "PortugueseBR"

Section "Spotify Dropdown" Main
  SetShellVarContext current
  SetOutPath "$INSTDIR"
  SetOverwrite on
  ClearErrors
  File "${PROJECT_ROOT}\dist\v${APP_VERSION}\Spotify Dropdown.exe"
  IfErrors 0 +3
    MessageBox MB_OK|MB_ICONSTOP "Feche o Spotify Dropdown e execute o instalador novamente."
    Abort
  File "${PROJECT_ROOT}\README.md"
  File "${PROJECT_ROOT}\CHANGELOG.md"
  WriteUninstaller "$INSTDIR\Desinstalar.exe"
  !ifndef QA_ONLY
    CreateDirectory "$SMPROGRAMS\Spotify Dropdown"
    CreateShortcut "$SMPROGRAMS\Spotify Dropdown\Spotify Dropdown.lnk" "$INSTDIR\Spotify Dropdown.exe"
    CreateShortcut "$SMPROGRAMS\Spotify Dropdown\Desinstalar.lnk" "$INSTDIR\Desinstalar.exe"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SpotifyDropdown" "DisplayName" "Spotify Dropdown"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SpotifyDropdown" "DisplayVersion" "${APP_VERSION}"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SpotifyDropdown" "InstallLocation" "$INSTDIR"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SpotifyDropdown" "DisplayIcon" "$INSTDIR\Spotify Dropdown.exe"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SpotifyDropdown" "UninstallString" '$\"$INSTDIR\Desinstalar.exe$\"'
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SpotifyDropdown" "NoModify" 1
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SpotifyDropdown" "NoRepair" 1
  !endif
SectionEnd

Section "Uninstall"
  SetShellVarContext current
  ClearErrors
  Delete "$INSTDIR\Spotify Dropdown.exe"
  IfErrors 0 +3
    MessageBox MB_OK|MB_ICONSTOP "Feche o Spotify Dropdown e tente desinstalar novamente."
    Abort
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\CHANGELOG.md"
  Delete "$INSTDIR\Desinstalar.exe"
  !ifndef QA_ONLY
    Delete "$SMPROGRAMS\Spotify Dropdown\Spotify Dropdown.lnk"
    Delete "$SMPROGRAMS\Spotify Dropdown\Desinstalar.lnk"
    RMDir "$SMPROGRAMS\Spotify Dropdown"
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SpotifyDropdown"
  !endif
  ; Keep settings.ini and any other user-created files. Never delete directories recursively.
  RMDir "$INSTDIR"
SectionEnd
