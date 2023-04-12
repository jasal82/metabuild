[CmdletBinding(DefaultParameterSetName = 'Default')]
param(
    [Parameter(Mandatory = $false)]
    [string]
    $MetabuildVersion = $env:metabuildVersion
)

if (-not $MetabuildVersion) {
    Write-Host "Determining latest metabuild version"
    $MetabuildVersion = ((Invoke-WebRequest -Uri "https://api.github.com/repos/jasal82/metabuild/releases/latest").Content | ConvertFrom-Json)[0].tag_name.TrimStart('v')
}

Write-Host "Downloading metabuild $MetabuildVersion"
$MetabuildDownloadUrl = "https://github.com/jasal82/metabuild/releases/download/v$MetabuildVersion/mb-v$MetabuildVersion-x86_64-pc-windows-msvc.exe"

$MetabuildInstallDir = Join-Path $env:LOCALAPPDATA -ChildPath "metabuild"

if (-not (Test-Path $MetabuildInstallDir -PathType Container)) {
    $null = New-Item -Path $MetabuildInstallDir -ItemType Directory
}

$file = Join-Path $MetabuildInstallDir "mb.exe"

Invoke-WebRequest -Uri $MetabuildDownloadUrl -OutFile $file

Write-Host "Setting PATH"
[Environment]::SetEnvironmentVariable(
    "Path",
    [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::Machine) + ";$MetabuildInstallDir",
    [EnvironmentVariableTarget]::User)

Write-Host "Done"