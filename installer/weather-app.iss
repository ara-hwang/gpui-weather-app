; 한국 날씨 설치 프로그램 스크립트 (Inno Setup 6)
; 빌드: pwsh -File installer\build-installer.ps1

#define AppName "한국 날씨"
#define AppVersion "0.1.0"
#define AppExeName "weather-app.exe"

; 실행 파일 폴더 (ISCC /DBuildDir=... 로 변경 가능).
#ifndef BuildDir
  #define BuildDir "..\target\release"
#endif

; 출력 폴더 (ISCC /DOutputDir=... 로 변경 가능).
#ifndef OutputDir
  #define OutputDir "..\dist"
#endif

; 앱 로컬 런타임 DLL 폴더 (앱 폴더에 나란히 복사해 관리자 권한 없이 실행).
#ifndef RuntimeDir
  #define RuntimeDir "..\target\installer\runtime"
#endif

[Setup]
; 업그레이드/제거용 고유 식별자 (유지 필수).
AppId={{74DDA8FF-5783-4D9D-AEEF-0461FE8396F8}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
VersionInfoVersion={#AppVersion}
VersionInfoDescription={#AppName} 설치 프로그램
VersionInfoProductName={#AppName}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
UninstallDisplayName={#AppName}
UninstallDisplayIcon={app}\{#AppExeName}
OutputDir={#OutputDir}
OutputBaseFilename=한국날씨-Setup-{#AppVersion}
SetupIconFile=..\assets\icon.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
; 사용자 폴더 설치 (관리자 권한 불필요).
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "korean"; MessagesFile: "compiler:Languages\Korean.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "{#BuildDir}\{#AppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#RuntimeDir}\*"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExeName}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent
