# expVI — VIctory for NVIDIA (RTX 3060 12 GB & 16 GB RAM Edition)

Custom Windows compatibility port of **VIctory** tuned specifically for **NVIDIA GeForce RTX 3060 (12 GB VRAM) + 16 GB System RAM** (with presets for DLSS, No-DLSS, Series X, and RTX 40+ Frame Generation).

> **This repository contains no game files, code or assets.** You need your own legally obtained dump of the title's Xbox Series build `v1.0.2.1` (`game_scarlett_master.exe`). See [docs/asset-policy.md](docs/asset-policy.md).

---

## How to Run (Step-by-Step)

### 1. Prerequisites
- **Windows 10 or 11 (x64)** with an **NVIDIA GeForce RTX GPU** (recent driver) and **16 GB+ RAM** (SSD recommended)
- **[Rust](https://rustup.rs)**
- **[Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)** with **"Desktop development with C++"** checked
- Your own legal dump of the game (`game_scarlett_master.exe` + `.rpf` files)

### 2. Clone, Build & Point to Your Game
Open **PowerShell** (normal user window, not Administrator) and run:
```powershell
git clone https://github.com/ExpBetter/expVI
cd expVI
cargo build --release --locked -p victory
.\Get-NVIDIA-DLSS.cmd
```
Then tell `expVI` where your game folder is (outside the `expVI` folder) using **either** file:
- Create `game_dir.txt` with your game folder path on line 1 (e.g. `D:\Games\Game`), **OR**
- Copy `victory.example.toml` to `victory.toml` and set `game_dir = 'D:\Games\Game'`

### 3. Launch
Double-click **`Play-RTX3060.cmd`** (or run `.\Play-RTX3060.cmd` in PowerShell) and pick a preset:

| Mode | Profile | Upscaler | Frame Gen | Display | Best For |
|---|---|---|---|---|---|
| **1** | Series S | DLSS Quality | Off | 1080p | **RTX 3060 / 8–12 GB VRAM + 16 GB RAM (Recommended)** |
| **2** | Series S | DLSS Performance | Off | 1080p | Maximum FPS |
| **3** | Series S | Title Upscaler (No DLSS) | Off | 1080p | Running without NVIDIA DLSS |
| **4** | Series X | DLSS Quality | Off | 1080p→1440p | Sharper visuals (needs more VRAM/RAM) |
| **5** | Series S | DLSS Quality | 2x | 1080p | RTX 40 / 50 series cards |
| **6** | — | — | — | — | Pre-translate shaders & warm NVIDIA pipelines only |

- **First run:** Automatically translates all game shaders (`victory shaders`) and warms recorded NVIDIA pipelines (`victory warm --jobs 4`) before opening the game. This takes a few minutes once and prevents missing graphics/stutters.
- **CLI shortcuts:** `.\Play-RTX3060.cmd -Mode 1`, `.\Play-RTX3060.cmd -Mode 3` (No DLSS), `.\Play-RTX3060.cmd -Exe "E:\MyGame\game_scarlett_master.exe"`, `.\Play-RTX3060.cmd -SkipWarm`.

---

## What `expVI` Customizes for RTX 3060 (12 GB) + 16 GB RAM

- **Pre-launch shader translation & pipeline warm-up (`[1/2]` & `[2/2]`):**
  - Automatically runs `victory.exe shaders` once to translate all game shaders into `%LOCALAPPDATA%\victory\cache`.
  - Runs `victory.exe warm --jobs 4` before launch to pre-compile recorded pipelines into NVIDIA's driver cache (capped to 4 parallel jobs so 16 GB RAM doesn't exhaust during compilation).
- **16 GB RAM / pagefile OOM protection:**
  - `VICTORY_CARD_BUFFERS_MIN_KB = '1024'` and `VICTORY_CARD_BUFFERS_MB = '1536'` (avoids copying thousands of tiny buffers that caused `nvwgf2umx.dll` / virtual commit OOM crashes at 95% memory load).
  - `VICTORY_LEVELS_ON_DEMAND = '1'` (loads only viewed texture mip levels instead of full mip chains).
  - `VICTORY_TEXTURE_ROOM_PERCENT = '80'` (leaves VRAM headroom for NVIDIA's driver and DLSS).
  - `-NoFrameGenReady` when FrameGen is off (saves ~1.5–2 GB VRAM on Ampere/RTX 3060 cards).
- **Clean NVIDIA rendering defaults:**
  - `VICTORY_F16_NEAREST = '1'` (eliminates 16-bit float banding/blocky shading on NVIDIA).
  - `VICTORY_DYNAMIC_SLOTS = '524288'` (expands NVIDIA's capped descriptor heap).
  - `VICTORY_THIN = '0'` (single recording thread when FrameGen is off so NVIDIA's descriptor pool doesn't overflow and drop draws).
  - `VICTORY_COMPOSE = '1'` (Series S two-plane UI composition) and `VICTORY_SKIP_BINK = '1'` (skips intro Bink videos; pass `-KeepBink` to keep them).
- **1080p Display Simulation:**
  - `VICTORY_DISPLAY_MODE = '1920x1080'` and `VICTORY_WINDOW_MAX = '1920x1080'`.
- **In-game controls:**
  - **F1:** In-game settings menu (DLSS mode, FPS cap, console profile, etc., saved to `victory-settings.txt`).
  - **F3:** Performance & memory overlay (FPS, frame time, draws, dropped draws, RAM, VRAM).

---

## Observed Test Results (RTX 3060 12 GB + i5-14400F + 15.8 GB RAM)

Tested on **Windows 11**, **Intel Core i5-14400F**, **15.8 GB RAM**, **NVIDIA GeForce RTX 3060 12 GB** (`game_scarlett_master.exe` v1.0.2.1):

- **Boot & 3D World Entry:** Reaches the 3D world with DLSS Super Resolution active (`nvngx_dlss.dll` via Streamline v2.14.16).
- **First-run pipeline compilation:** Compiling 500–900+ unseen pipelines live during gameplay causes severe stalls (`0.1–0.2 fps`) and dropped draws (`500+` draws dropped, causing blocky/missing lighting); pre-translating with `victory shaders` + `victory warm --jobs 4` resolves this ahead of launch.
- **VRAM Budget Note:** Windows D3D12 subtracts VRAM used by other running apps (browsers, overlays, IDEs) from the 12 GB card's reported budget (`~9.1 GB` with background apps open vs `~11.1 GB` clean). Closing heavy GPU apps before launching gives `victory` maximum VRAM headroom.

---

## Building from Source

Install [Rust](https://rustup.rs) and [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) ("Desktop development with C++"), then:

```powershell
git clone https://github.com/ExpBetter/expVI
cd expVI
cargo build --release --locked -p victory
.\Get-NVIDIA-DLSS.cmd
.\Play-RTX3060.cmd
```

Logs are written to `%LOCALAPPDATA%\victory\logs`, and saves/caches to `%LOCALAPPDATA%\victory\data` and `%LOCALAPPDATA%\victory\cache`.

---

## Releases & Documentation

- **Releases & Updates:** [ExpBetter/expVI](https://github.com/ExpBetter/expVI/releases)
- **Technical Docs:**
  - [docs/README-foxhound.md](docs/README-foxhound.md)
  - [docs/architecture.md](docs/architecture.md)
  - [docs/findings.md](docs/findings.md)
  - [docs/dlss.md](docs/dlss.md)
  - [docs/fork-ray-tracing.md](docs/fork-ray-tracing.md)
  - [docs/asset-policy.md](docs/asset-policy.md)

## Credits

- **Custom RTX 3060 / 16 GB RAM port & launcher:** ExpBetter (customized with Google Gemini / Antigravity)
- **Upstream open-source foundations:**
  - [VIctory](https://github.com/Marvisak/VIctory) by Marvisak and contributors
  - [foxhound](https://github.com/readmet3xt-lab/foxhound)
  - [VIctory-NVIDIA](https://github.com/trustno1-coder/VIctory-NVIDIA)
  - [fxceless/victory-fork](https://github.com/fxceless/victory-fork), [avatarmanz/VIctory-7900XTX](https://github.com/avatarmanz/VIctory-7900XTX), [Belobug/VIctory-Belobug-AMD](https://github.com/Belobug/VIctory-Belobug-AMD)
- NVIDIA Streamline, DLSS, and Reflex are loaded at runtime from `nvidia-sdk\` under NVIDIA's licenses.

## License

GPL-3.0-or-later, as VIctory and foxhound. See [LICENSE](LICENSE) and [docs/asset-policy.md](docs/asset-policy.md).
