# Sub-LLama

A CLI tool to extract and translate subtitles directly from MKV video files into
any target language using a local Ollama instance running the
**kaelri/hy-mt2:1.8b** translation model.

---

## Features

- **Local & Private:** Zero external API calls for translation. Everything runs
  on your machine.
- **Auto-Extraction:** Uses FFmpeg to extract embedded subtitles from MKV files.
- **Optimized Translation:** Uses the fast and efficient `kaelri/hy-mt2:1.8b`
  model designed specifically for multilingual translations.
- **Accurate Reconstruction:** Automatically compiles the translated subtitle
  lines back into standard `.srt` format, preserving original timings.
- **Progress Bar:** Real-time visual progress bar displayed in the terminal.
- **Subtitle Embedding (Muxing):** Mux the translated subtitle directly into the video container as a selectable soft subtitle track (enabled by default).
- **Multi-Track Support:** Detect and select specific subtitle tracks from MKV videos with multiple subtitle streams.
- **Customizable:** Configure translation model, batch sizes, and remote Ollama hosts.

---

## Prerequisites

Before using `Sub-LLama`, make sure you have the following installed on your
system:

1. **FFmpeg & FFprobe** (required to extract/embed subtitles and query video duration):
   - **macOS:** `brew install ffmpeg`
   - **Ubuntu/Debian:** `sudo apt install ffmpeg`
   - **Windows:** Download from the official website or install via
     `winget install gyan.ffmpeg`.

2. **Ollama**:
   - Install Ollama from [ollama.com](https://ollama.com).
   - Pull the translation model:
     ```bash
     ollama pull kaelri/hy-mt2:1.8b
     ```
   - Ensure the Ollama server is running (usually on `http://localhost:11434`).

---

## Installation

We recommend using [uv](https://github.com/astral-sh/uv) to manage dependencies
and environments easily.

1. **Clone the repository:**
   ```bash
   git clone https://github.com/yourusername/sub-llama.git
   cd sub-llama
   ```

2. **Install dependencies and create virtual environment:** Using `uv`:
   ```bash
   uv sync
   ```
   Or using standard `pip`:
   ```bash
   pip install -e .
   ```

3. **Global CLI Installation:**
   To make the `sub-llama` command available globally:
   ```bash
   uv tool install --editable .
   ```

---

## CLI Options & Flags

| Flag | Short | Description | Default |
| :--- | :--- | :--- | :--- |
| `--help` | `-h` | Show help message and exit | - |
| `--languages` | `-l` | List mapped languages with automatic VLC suffix naming | - |
| `--list-tracks` | `-lt` | List all subtitle tracks in a video file and exit | - |
| `--file` | `-f` | Translate a `.srt` subtitle file directly | - |
| `--embed-only` | `-eo` | Mux/embed an existing subtitle file into a video without translating | - |
| `--track` | `-t` | Subtitle track index to extract from MKV | `0` |
| `--no-embed` | `-n`, `-ne` | Disable embedding subtitle into video container (extract & translate only) | Disabled (Embed is ON) |
| `--in-place` | `-i` | Overwrite the original video file when embedding | `False` (`.embedded` file) |
| `--model` | `-m` | Ollama model to use for translation | `kaelri/hy-mt2:1.8b` |
| `--batch-size` | `-b` | Number of subtitle lines translated per batch | `30` |
| `--workers` | `-w` | Number of concurrent batch translation threads | `3` |
| `--host` | - | Ollama server URL/host | `http://localhost:11434` |

---

## Performance & Speed Optimizations

`Sub-LLama` includes several built-in optimizations for maximum translation speed:
- **Parallel Multi-threading (`-w` / `--workers`):** Dispatches multiple subtitle batches concurrently to Ollama.
- **Batched Requests (`-b` / `--batch-size`):** Translates 30+ lines per network roundtrip, significantly reducing prompt evaluation overhead.
- **Deterministic Sampling:** Fixed low temperature (`0.1`) for faster and faithful translation decoding.
- **Untranslatable Fast-path:** Automatically skips LLM inference for sound effects, music notes (`♪`), and empty/number-only lines.
- **Memory Keep-Alive:** Keeps the model loaded in Ollama VRAM between batches to eliminate reload latency.

---

## Usage Examples

### 1. Translating a Video (Default: Auto-embeds subtitle & multi-threaded)
Extracts the subtitle, translates to Brazilian Portuguese (default), and embeds it in a new `.embedded.mkv` file:
```bash
sub-llama movie.mkv
```

Translate to another target language:
```bash
sub-llama movie.mkv Spanish
```

### 2. High-Performance / Turbo Mode (`-w` and `-b`)
Maximize throughput on multi-core systems / GPUs with larger batches and more worker threads:
```bash
sub-llama movie.mkv "Brazilian Portuguese" -w 4 -b 40
```

### 3. Overwriting Original Video File (`-i` / `--in-place`)
Muxes the translated subtitle directly into the original video file:
```bash
sub-llama movie.mkv Spanish -i
```

### 4. Extract and Translate Only (`-n` / `--no-embed`)
Generates the `.srt` file alongside the video without embedding it back into the container:
```bash
sub-llama movie.mkv Spanish -n
```

### 5. Selecting a Specific Subtitle Track (`-t` / `--track`)
List tracks first:
```bash
sub-llama -lt movie.mkv
```
Translate using a specific track index (e.g., track 1):
```bash
sub-llama movie.mkv "Brazilian Portuguese" -t 1 -i
```

### 6. Translating Subtitle Files Directly (`-f` / `--file`)
Translate an existing `.srt` file:
```bash
sub-llama -f subtitle.srt "Brazilian Portuguese"
```
Specifying source and target languages:
```bash
sub-llama -f subtitle.srt English "Brazilian Portuguese"
```

### 7. Embedding an Existing Subtitle File Only (`-eo` / `--embed-only`)
Embed an existing `.srt` file into a video container without re-encoding:
```bash
sub-llama -eo movie.mp4 subtitle.srt "Brazilian Portuguese" -i
```

### 8. Custom Model, Batch Size & Ollama Host
```bash
sub-llama movie.mkv Spanish -m "kaelri/hy-mt2:1.8b" -b 40 -w 4 --host "http://localhost:11434"
```

---

## Disclaimer

This project is developed for educational and personal productivity purposes.
The author is not responsible for any misuse of this software, including the
unauthorized translation or distribution of copyrighted materials. Users are
solely responsible for ensuring they have the legal right to process and
translate the media files they input into this tool.
