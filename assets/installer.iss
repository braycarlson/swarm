#ifndef MyAppVersion
  #define MyAppVersion "0.0.1"
#endif

#define MyAppName "swarm"
#define MyAppPublisher "Brayden Carlson"
#define MyAppURL "https://www.braydencarlson.com/"
#define MyAppExeName "swarm.exe"
#define MyAppId "{{AF880013-CB11-4D3F-82E5-38502F92EDA0}"

[Setup]
AppId={#MyAppId}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DisableDirPage=yes
UninstallDisplayIcon={app}\{#MyAppExeName}
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputDir=Output
OutputBaseFilename=swarm-setup
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
; Context menu for directories
Root: HKCU; Subkey: "Software\Classes\Directory\shell\swarm"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Directory\shell\swarm"; ValueType: string; ValueName: ""; ValueData: "Open swarm here"
Root: HKCU; Subkey: "Software\Classes\Directory\shell\swarm"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\{#MyAppExeName}"
Root: HKCU; Subkey: "Software\Classes\Directory\shell\swarm\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%V"""

; Context menu for directory background
Root: HKCU; Subkey: "Software\Classes\Directory\Background\shell\swarm"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Directory\Background\shell\swarm"; ValueType: string; ValueName: ""; ValueData: "Open swarm here"
Root: HKCU; Subkey: "Software\Classes\Directory\Background\shell\swarm"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\{#MyAppExeName}"
Root: HKCU; Subkey: "Software\Classes\Directory\Background\shell\swarm\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%V"""

; Context menu for all files
Root: HKCU; Subkey: "Software\Classes\*\shell\swarm"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\*\shell\swarm"; ValueType: string; ValueName: ""; ValueData: "Open swarm here"
Root: HKCU; Subkey: "Software\Classes\*\shell\swarm"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\{#MyAppExeName}"
Root: HKCU; Subkey: "Software\Classes\*\shell\swarm\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%V"""

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch {#MyAppName}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "cmd.exe"; Parameters: "/c reg delete ""HKCU\Software\Classes\Directory\shell\swarm"" /f"; Flags: runhidden
Filename: "cmd.exe"; Parameters: "/c reg delete ""HKCU\Software\Classes\Directory\Background\shell\swarm"" /f"; Flags: runhidden
Filename: "cmd.exe"; Parameters: "/c reg delete ""HKCU\Software\Classes\*\shell\swarm"" /f"; Flags: runhidden
