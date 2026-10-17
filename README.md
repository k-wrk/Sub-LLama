# Sub-LLama

A CLI tool written in Rust to extract, translate, and embed subtitles directly into video files (`.mkv`, `.mp4`) or standalone `.srt` files locally using CTranslate2 and MarianMT models.

---

## Features

- **Offline Local Translation:** Runs MarianMT models locally on CPU via CTranslate2 without external API calls.
- **In-Place Video Muxing:** Extracts subtitles in memory, translates them in batches, and embeds the output stream back into the video container using FFmpeg.
- **Track Deduplication:** Checks existing subtitle streams to avoid duplicating tracks of the same language.
- **Batch Directory Processing:** Traverses directories to translate multiple video files, skipping files that already contain the target subtitle track.
- **Multi-Language Support:** Supports translation from English to Portuguese, Spanish, French, German, Italian, Russian, Chinese, Japanese, Korean, Arabic, Turkish, Vietnamese, and Indonesian.
- **Interactive Mode:** Provides terminal prompts to select input files and target languages when run without flags.

---

## Prerequisites

1. **Rust & Cargo:** (Rust 1.80+ recommended)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **FFmpeg & FFprobe:**
   - **macOS:** `brew install ffmpeg cmake`
   - **Ubuntu/Debian:** `sudo apt install ffmpeg cmake build-essential`
   - **Arch Linux:** `sudo pacman -S ffmpeg cmake base-devel`

---

## Installation

### Via Homebrew (macOS & Linux)

```bash
brew tap k-wrk/tap
brew install sub-llama
```

### From Source (Cargo)

Clone the repository and compile the optimized release binary:

```bash
git clone https://github.com/k-wrk/Sub-LLama.git
cd Sub-LLama
cargo build --release
```

To install the binary globally into your system path (`~/.cargo/bin/sub-llama`):

```bash
cargo install --path .
```

---

## CLI Options

| Flag | Short | Description | Default |
| :--- | :--- | :--- | :--- |
| `[INPUT_PATH]` | - | Path to video (`.mkv`, `.mp4`) or subtitle (`.srt`) | Interactive Wizard |
| `--dir` | `-d` | Directory mode: process all videos in the target folder | - |
| `--recursive` | `-r` | Recursively scan subdirectories in directory mode | `false` |
| `--to` | - | Target language code (`pt`, `es`, `fr`, `de`, `it`, `ru`, `zh`, `ja`, `ko`, `ar`, `tr`, `vi`, `id`) | `pt` |
| `--force` | - | Force translation even if target language already exists | `false` |
| `--list-languages` | - | List all 13 supported languages and their model repositories | - |
| `--list-tracks` | `-l` | List existing subtitle tracks inside the video file and exit | - |
| `--track` | `-t` | Subtitle track index to extract | `0` |
| `--no-embed` | `-n` | Do not embed into video container (generates `.srt` only) | `false` |
| `--keep-subs` | - | Keep the `.srt` file saved alongside the video after embedding | `false` |
| `--new-file` | - | Generate a new `.translated.mkv` file instead of in-place muxing | `false` |
| `--workers` | `-w` | Number of worker threads for parallel jobs | `2` |
| `--models-dir` | - | Custom directory for downloaded models | `~/.cache/sub-llama/models/` |

---

## Usage Examples

### 1. Translate and embed into video in-place
```bash
sub-llama movie.mkv --to es
```

### 2. Batch process an entire TV season
Automatically skips episodes that already contain the target language track:
```bash
sub-llama -d /path/to/season/ --to pt
```

### 3. Recursively translate series folders
```bash
sub-llama -d /path/to/tv_shows/ -r --to fr
```

### 4. Translate a standalone subtitle file (`.srt`)
```bash
sub-llama -f subtitle.srt --to it
```

### 5. Inspect and select a specific subtitle track
```bash
# 1. List available tracks
sub-llama -l video.mkv

# 2. Translate using track index 1
sub-llama video.mkv -t 1 --to pt
```

### 6. Generate `.srt` only without touching the video
```bash
sub-llama video.mkv -n --to es
```

---

## Neural Translation Models

Models are based on **Helsinki-NLP (OPUS-MT)** architectures converted and optimized for the **CTranslate2** engine:

- **Portuguese (`pt`):** `Helsinki-NLP/opus-mt-tc-big-en-pt` (CTranslate2 INT8 via `playto-mt`)
- **Spanish (`es`):** `Helsinki-NLP/opus-mt-tc-big-en-es` (CTranslate2 INT8 via `CristianRojaas`)
- **French (`fr`):** `Helsinki-NLP/opus-mt-tc-big-en-fr` (CTranslate2 INT8 via `michaelfeil`)
- **German (`de`):** `Helsinki-NLP/opus-mt-en-de` (CTranslate2 INT8 via `michaelfeil`)
- **Italian (`it`):** `Helsinki-NLP/opus-mt-tc-big-en-it` (CTranslate2 via `ooeoeo`)
- **Russian (`ru`):** `Helsinki-NLP/opus-mt-en-ru` (CTranslate2 via `ooeoeo`)
- **Chinese (`zh`):** `Helsinki-NLP/opus-mt-en-zh` (CTranslate2 via `ooeoeo`)
- **Japanese (`ja`):** `Helsinki-NLP/opus-mt-en-ja` (CTranslate2 INT8 via `craftwise`)
- **Korean (`ko`):** `Helsinki-NLP/opus-mt-tc-big-en-ko` (CTranslate2 via `ooeoeo`)
- **Arabic (`ar`):** `Helsinki-NLP/opus-mt-tc-big-en-ar` (CTranslate2 via `ai-path`)
- **Turkish (`tr`):** `Helsinki-NLP/opus-mt-tc-big-en-tr` (CTranslate2 via `ooeoeo`)
- **Vietnamese (`vi`):** `Helsinki-NLP/opus-mt-en-vi` (CTranslate2 INT8 via `dekthedev`)
- **Indonesian (`id`):** `Helsinki-NLP/opus-mt-en-id` (CTranslate2 via `ooeoeo`)

Model weights and vocabularies are downloaded automatically on first execution and cached locally in `~/.cache/sub-llama/models/`.

---

## Disclaimer

This project is developed for educational and personal productivity purposes. The author is not responsible for any misuse of this software, including the unauthorized translation or distribution of copyrighted materials. Users are solely responsible for ensuring they have the legal right to process and translate the media files they input into this tool.

---

## License

Distributed under the **PolyForm Noncommercial License 1.0.0**.

- Free for personal, academic, educational, and non-commercial research use.
- Commercial use or use for commercial gain is **strictly prohibited** without prior written agreement.
- See [`LICENSE`](LICENSE) for the full license terms.

