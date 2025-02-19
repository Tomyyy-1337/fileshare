; Script generated by the Inno Setup Script Wizard.
; SEE THE DOCUMENTATION FOR DETAILS ON CREATING INNO SETUP SCRIPT FILES!

#define MyAppName "Fileshare"
#define MyAppVersion "1.10"
#define MyAppPublisher "Tom Paßberg"
#define MyAppURL "https://github.com/Tomyyy-1337/PasswordManager"
#define MyAppExeName "fileshare.exe"

[Setup]
; NOTE: The value of AppId uniquely identifies this application. Do not use the same AppId value in installers for other applications.
; (To generate a new GUID, click Tools | Generate GUID inside the IDE.)
AppId={{C7B909DF-AC5B-40B1-9618-61945E73861E}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
;AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
UninstallDisplayIcon={app}\{#MyAppExeName}
DisableProgramGroupPage=yes
; Uncomment the following line to run in non administrative install mode (install for current user only).
;PrivilegesRequired=lowest
OutputDir=.\target\inno
OutputBaseFilename=fileshare_installer
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: ".\target\optimized\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: ".\static\*"; DestDir: "{app}\static"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: ".\template\*"; DestDir: "{app}\template"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: ".\assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs
; NOTE: Don't use "Flags: ignoreversion" on any shared system files

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\icon.ico"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon; IconFilename: "{app}\assets\icon.ico"

