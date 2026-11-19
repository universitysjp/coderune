[Setup]
AppId=edu.sjp.coderune
AppName=Coderune
AppVersion={#AppVersion}
DefaultDirName={localappdata}\Programs\Coderune
DefaultGroupName=Coderune
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SetupIconFile=..\assets\icon.ico
UninstallDisplayIcon={app}\Coderune.ico
OutputDir=..\..\..\dist\desktop
OutputBaseFilename=Coderune-windows-x64-setup

[Files]
Source: "..\..\..\target\release\coderune.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\icon.ico"; DestDir: "{app}"; DestName: "Coderune.ico"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Coderune"; Filename: "{app}\coderune.exe"; IconFilename: "{app}\Coderune.ico"
