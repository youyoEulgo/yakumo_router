# Install yakumo: use a binary next to this script when present (unpacked
# archive), otherwise download the latest release for this platform.
# It also adds yakumo to the user PATH and creates a default config.
$ErrorActionPreference = 'Stop'

# Windows PowerShell 5.1 needs TLS 1.2 explicitly for the GitHub API and assets.
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$Repo = 'youyoEulgo/yakumo_router'
$BinName = 'yakumo.exe'

$ScriptDir = $null
if ($MyInvocation.MyCommand.Path) {
    $ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
}
$LocalSource = if ($ScriptDir) { Join-Path $ScriptDir $BinName } else { $null }

function Get-Target {
    switch ($env:PROCESSOR_ARCHITECTURE) {
        'AMD64' { return 'x86_64-pc-windows-msvc' }
        default { throw "no prebuilt binary for Windows/$env:PROCESSOR_ARCHITECTURE" }
    }
}

function Fetch-Binary {
    $target = Get-Target

    $version = $env:YAKUMO_VERSION
    if ([string]::IsNullOrEmpty($version)) {
        $version = (Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest").tag_name
    }
    if ([string]::IsNullOrEmpty($version)) {
        throw 'could not determine the latest release'
    }

    $asset = "yakumo-$version-$target.zip"
    $url = "https://github.com/$Repo/releases/download/$version/$asset"
    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("yakumo-" + [System.Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null

    Write-Host "Downloading $asset ..."
    $zip = Join-Path $tmp $asset
    Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing
    Expand-Archive -Path $zip -DestinationPath $tmp -Force

    $found = Get-ChildItem -Path $tmp -Recurse -Filter $BinName | Select-Object -First 1
    if (-not $found) {
        throw "$BinName not found inside $asset"
    }

    return $found.FullName
}

$Source = if ($LocalSource -and (Test-Path $LocalSource)) { $LocalSource } else { Fetch-Binary }

$InstallDir = if ($env:YAKUMO_INSTALL_DIR) {
    $env:YAKUMO_INSTALL_DIR
} else {
    Join-Path $env:LOCALAPPDATA 'Programs\yakumo'
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$Target = Join-Path $InstallDir $BinName
Copy-Item -Force $Source $Target

$UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ([string]::IsNullOrEmpty($UserPath)) {
    $UserPath = ''
}

$skipPathEdit = $env:YAKUMO_NO_MODIFY_PATH -in @('1', 'true', 'yes')

$entries = $UserPath -split ';' | Where-Object { $_ -ne '' }
if ($entries -notcontains $InstallDir) {
    if ($skipPathEdit) {
        Write-Host "Add $InstallDir to your PATH to use yakumo."
    } else {
        $newPath = (@($entries) + $InstallDir) -join ';'
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
        $env:Path = "$env:Path;$InstallDir"
        Write-Host "Added $InstallDir to your user PATH."
    }
}

# Create a default config the first time; init is a no-op if one already exists.
try {
    & $Target init | Out-Null
} catch {
    # Ignore: the config can also be created from the web UI.
}

Write-Host "yakumo installed to $Target"
try {
    & $Target --version
} catch {
    # Ignore: older builds do not support --version.
}
Write-Host "Open a new terminal, then run: yakumo"
