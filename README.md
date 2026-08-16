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
- **Subtitle Embedding (Muxing):** Mux the translated subtitle directly into the video container as a selectable soft subtitle track.
- **Multi-Track Support:** Detect and select specific subtitle tracks from MKV videos with multiple subtitle streams.

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

## Usage

Once installed, you can run the translation script using the registered command.

### Getting Help & Listing Languages

- **Help Menu**: To view all options, run:
  ```bash
  sub-llama --help
  ```
- **List Mapped Languages**: To view all 50+ languages with automatic VLC naming support, run:
  ```bash
  sub-llama --languages
  ```

---

### Command Examples

#### 1. List Subtitle Tracks inside a Video (Without Translating)
```bash
sub-llama --list-tracks path/to/your/video.mkv
```

#### 2. Translating and Automatically Embedding
Extracts subtitles, translates them, and muxes them back into the video:
```bash
sub-llama path/to/your/video.mkv --embed
```

#### 3. Select a Specific Subtitle Track to Translate
If the video has multiple subtitle tracks (like forced, full, SDH) and you want to select a specific track index (e.g. index 1) and embed it:
```bash
sub-llama path/to/your/video.mkv "Brazilian Portuguese" --track 1 --embed
```

#### 4. Embedding an Existing Subtitle File (Muxing Only)
Embed an existing `.srt` file into a video container without re-encoding (instantaneous, with progress bar):
```bash
sub-llama --embed-only path/to/your/video.mp4 path/to/your/subtitle.srt "Brazilian Portuguese"
```

#### 5. Translating Subtitle Files Directly
You can also translate an existing subtitle file (`.srt`) directly:
```bash
sub-llama --file path/to/subtitle.srt [original_language] [target_language]
```

---

## Disclaimer

This project is developed for educational and personal productivity purposes.
The author is not responsible for any misuse of this software, including the
unauthorized translation or distribution of copyrighted materials. Users are
solely responsible for ensuring they have the legal right to process and
translate the media files they input into this tool.
