# expVI — Custom RTX 3060 / NVIDIA Launcher (wraps play-foxhound.ps1)
param(
    [string]$Exe = '',
    [ValidateSet('', '1', '2', '3', '4', '5', '6')][string]$Mode = '',
    [switch]$SeriesX,
    [ValidateSet('off', 'dlaa', 'quality', 'balanced', 'performance')][string]$Dlss = 'quality',
    [switch]$FrameGen,
    [ValidateSet('1440', '1080to1440', '4k')][string]$Res = '1080to1440',
    [ValidateRange(1, 1000)][int]$Fps = 1000,
    [switch]$AutoStart,
    [switch]$KeepBink,
    [switch]$Precompile,
    [switch]$SkipWarm,
    [ValidateSet('diag', 'release')][string]$Build = 'release',
    [hashtable]$More = @{}
)

$repo = Split-Path $PSScriptRoot -Parent

# Auto-detect game_scarlett_master.exe from -Exe, game_dir.txt, victory.toml, or D:\Games\Game
if (-not $Exe) {
    $txt = Join-Path $repo 'game_dir.txt'
    if (Test-Path $txt) {
        $d = (Get-Content $txt -Encoding UTF8 -TotalCount 1).Trim().Trim('"')
        if ($d) { $Exe = Join-Path $d 'game_scarlett_master.exe' }
    }
}
if (-not $Exe -or -not (Test-Path -LiteralPath $Exe)) {
    $toml = Join-Path $repo 'victory.toml'
    if (Test-Path $toml) {
        $m = Select-String -Path $toml -Pattern "^\s*game_dir\s*=\s*['""](.+?)['""]" | Select-Object -First 1
        if ($m) { $Exe = Join-Path $m.Matches[0].Groups[1].Value 'game_scarlett_master.exe' }
    }
}
if (-not $Exe) { $Exe = 'D:\Games\Game\game_scarlett_master.exe' }

# Show interactive preset menu if no explicit mode/override flags were passed
$hasOverride = $Mode -or $PSBoundParameters.ContainsKey('Dlss') -or $PSBoundParameters.ContainsKey('FrameGen') -or $PSBoundParameters.ContainsKey('SeriesX') -or $Precompile
if (-not $hasOverride) {
    Write-Host ''
    Write-Host '================ expVI Launcher (RTX 3060 / NVIDIA) ================' -ForegroundColor Cyan
    Write-Host '  [1] RTX 3060 Recommended   (Series S | DLSS Quality | No FrameGen | 1080p)'
    Write-Host '  [2] Max FPS / Performance  (Series S | DLSS Perf    | No FrameGen | 1080p)'
    Write-Host '  [3] No DLSS (Native/Title) (Series S | DLSS Off     | No FrameGen | 1080p)'
    Write-Host '  [4] Series X Sharp         (Series X | DLSS Quality | No FrameGen | 1080->1440p)'
    Write-Host '  [5] DLSS + FrameGen 2x     (Series S | DLSS Quality | FrameGen 2x | RTX 40+)'
    Write-Host '  [6] Pre-translate Shaders & Warm Pipelines only'
    Write-Host '====================================================================' -ForegroundColor Cyan
    $pick = Read-Host 'Select mode [1-6] (Enter = 1)'
    if (-not $pick) { $pick = '1' }
    $Mode = $pick.Trim()
}

switch ($Mode) {
    '1' { $SeriesX = $false; $Dlss = 'quality';     $FrameGen = $false }
    '2' { $SeriesX = $false; $Dlss = 'performance'; $FrameGen = $false }
    '3' { $SeriesX = $false; $Dlss = 'off';         $FrameGen = $false }
    '4' { $SeriesX = $true;  $Dlss = 'quality';     $FrameGen = $false; $Res = '1080to1440' }
    '5' { $SeriesX = $false; $Dlss = 'quality';     $FrameGen = $true }
    '6' { $Precompile = $true }
}

if ($Mode -ne '6' -and -not (Test-Path -LiteralPath $Exe)) {
    Write-Host "Game not found at: $Exe" -ForegroundColor Red
    Write-Host 'Put your game folder path in game_dir.txt or victory.toml, or pass -Exe "path\to\game_scarlett_master.exe".'
    return
}

$victory = Join-Path $repo "target\$Build\victory.exe"
if (Test-Path $victory) {
    $cacheDir = if ($env:VICTORY_CACHE_DIR) { $env:VICTORY_CACHE_DIR } else { Join-Path $env:LOCALAPPDATA 'victory\cache' }
    $marker = Join-Path $cacheDir '.rtx3060_shaders_done'
    if ($Precompile -or -not (Test-Path $marker)) {
        Write-Host '[1/2] Translating all game shaders into cache (one-time, may take a few minutes)...'
        Push-Location $repo
        try { & $victory shaders } finally { Pop-Location }
        if ($LASTEXITCODE -eq 0) {
            New-Item -ItemType Directory -Force -Path $cacheDir | Out-Null
            Set-Content -Path $marker -Value (Get-Date -Format o)
        }
    }
    if (-not $SkipWarm) {
        $warmArgs = if ($Mode -eq '6') { @('warm', '--jobs', '8') } else { @('warm', '--used', '--jobs', '8') }
        Write-Host "[2/2] Pre-compiling NVIDIA pipelines ($($warmArgs -join ' '))..."
        Push-Location $repo
        try { & $victory @warmArgs } finally { Pop-Location }
    }
    if ($Mode -eq '6') {
        Write-Host 'Shader translation and pipeline warm-up complete.' -ForegroundColor Green
        return
    }
}

$extra = @{
    VICTORY_DISPLAY_MODE         = '1920x1080'
    VICTORY_WINDOW_MAX           = '1920x1080'
    # Clean NVIDIA graphics: no f16 banding, big dynamic heap, single recording thread
    VICTORY_F16_NEAREST          = '1'
    VICTORY_DYNAMIC_SLOTS        = '524288'
    VICTORY_THIN                 = '0'
    VICTORY_COMPOSE              = '1'
    # Keep 15.8 GB system RAM + pagefile from exhausting during shader compilation (nvwgf2umx.dll OOM):
    VICTORY_CARD_BUFFERS_MIN_KB  = '1024'
    VICTORY_CARD_BUFFERS_MB      = '1536'
    VICTORY_LEVELS_ON_DEMAND     = '1'
    VICTORY_TEXTURE_ROOM_PERCENT = '80'
    VICTORY_PANEL                = '1'
}
if ($FrameGen) {
    # FrameGen requires VICTORY_THIN=1 or VICTORY_ASYNC_SUBMIT=1
    $extra.VICTORY_ASYNC_SUBMIT = '1'
}
if (-not $KeepBink) { $extra.VICTORY_SKIP_BINK = '1' }
if ($AutoStart) {
    $extra.VICTORY_KEYS = '25:Enter,35:Enter,45:Enter,55:Enter,65:Enter,75:Enter,85:Enter,95:Enter,105:Enter,115:Enter'
}
foreach ($k in $More.Keys) { $extra[$k] = $More[$k] }

$argsHash = @{
    Exe             = $Exe
    Build           = $Build
    Dlss            = $Dlss
    Res             = $Res
    Fps             = $Fps
    NoFrameGenReady = (-not $FrameGen)
    More            = $extra
}
if ($FrameGen) { $argsHash.FrameGen = $true }
if (-not $SeriesX) { $argsHash.SeriesS = $true }

& "$PSScriptRoot\play-foxhound.ps1" @argsHash
