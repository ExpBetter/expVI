# RTX 3060 12 GB + 16 GB RAM tailored launcher (on top of play-foxhound.ps1).
#   - Uses Series S profile by default (lighter on 15.8 GB system RAM; pass -SeriesX for Series X 1080p->1440p)
#   - Uses DLSS Quality on RTX 3060 Tensor Cores, with FrameGen disabled (-NoFrameGenReady saves ~1.5-2 GB VRAM)
#   - Simulates 1080p display (VICTORY_DISPLAY_MODE=1920x1080) & caps window to 1080p
#   - Skips intro Bink videos (VICTORY_SKIP_BINK=1) and enables F3 overlay (VICTORY_PANEL=1)
#   - Keeps texture VRAM ceiling at 86% so 12 GB VRAM doesn't spill into 15.8 GB system RAM
param(
    [string]$Exe = 'D:\Games\Game\game_scarlett_master.exe',
    [switch]$SeriesX,
    [ValidateSet('off', 'dlaa', 'quality', 'balanced', 'performance')][string]$Dlss = 'quality',
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
        Write-Host '[2/2] Pre-compiling recorded pipelines for NVIDIA (--jobs 4 for 16 GB RAM)...'
        Push-Location $repo
        try { & $victory warm --jobs 4 } finally { Pop-Location }
    }
}

$extra = @{
    VICTORY_DISPLAY_MODE         = '1920x1080'
    VICTORY_WINDOW_MAX           = '1920x1080'
    # Clean NVIDIA graphics (from victory-corgi): no f16 banding, big dynamic heap, single recording thread
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
    NoFrameGenReady = $true
    More            = $extra
}
if (-not $SeriesX) { $argsHash.SeriesS = $true }

& "$PSScriptRoot\play-foxhound.ps1" @argsHash
