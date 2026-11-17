; OffHorizon's Windows setup, built by Inno Setup in the release workflow.
;
; The release's main file stays the portable OffHorizon.exe: it needs no administrator rights and
; installs nothing by itself. This setup exists for players who want an install they can remove
; cleanly: a Start menu entry, an optional desktop icon, and an uninstaller that takes the program
; and the launcher's own %APPDATA%\offhorizon copy away, leaving worlds, settings and accounts
; (they live in horizon-launcher, shared with Horizon) untouched.

#define AppName "OffHorizon"
#define AppExe "OffHorizon.exe"
#define Version Trim(FileRead(FileOpen("..\..\version.txt")))

[Setup]
AppId={{7A0C8C2E-6F41-4F3A-9B7D-2F1A4E0C9B11}
AppName={#AppName}
AppVersion={#Version}
AppVerName={#AppName} {#Version}
AppPublisher=PiBOH
AppPublisherURL=https://github.com/PiBOH/offhorizon
AppSupportURL=https://github.com/PiBOH/offhorizon/issues
AppUpdatesURL=https://piboh.github.io/offhorizon
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
OutputDir=..\..\dist
OutputBaseFilename=OffHorizon-Setup-Windows-x64
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SetupIconFile=..\icons\offhorizon.ico
UninstallDisplayIcon={app}\{#AppExe}
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
CloseApplications=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\..\dist\OffHorizon-Launcher-Windows-x64.exe"; DestDir: "{app}"; DestName: "{#AppExe}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; The launcher's own copy of itself, and the installed folder: settings, accounts, logs and the
; prepared game stay in horizon-launcher, which is shared with Horizon.
Type: filesandordirs; Name: "{userappdata}\offhorizon"
Type: filesandordirs; Name: "{app}"
