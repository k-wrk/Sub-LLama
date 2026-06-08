# Sub-LLama

A CLI tool to extract and translate subtitles directly from MKV video files into
any target language using a local Ollama instance running the
**kaelri/hy-mt2:1.8b** translation model.

---

## Features

- **Local & Private:** Zero external API calls for translation. Everything runs
  on your machine.
- **Auto-Extraction:** Uses FFmpeg to extract embedded English subtitles from
  MKV files.
- **Optimized Translation:** Uses the fast and efficient `kaelri/hy-mt2:1.8b`
  model designed specifically for multilingual translations.
- **Accurate Reconstruction:** Automatically compiles the translated subtitle
  lines back into standard `.srt` format, preserving original timings.

---

## Prerequisites

Before using `Sub-LLama`, make sure you have the following installed on your
system:

1. **FFmpeg** (required to extract subtitles from MKV files):
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

---

## Usage

Once installed, you can run the translation script using the registered command.

To view all options, examples, and the list of common languages, run:

```bash
sub-llama --help
```

### 1. Translating from Video Files (MKV)

Extracts subtitles automatically and translates them:

```bash
sub-llama path/to/your/video.mkv [target_language]
```

By default, the target language is `Brazilian Portuguese`. You can pass any
target language in English (e.g., `Spanish`, `French`, `German`):

```bash
# Translates to Brazilian Portuguese by default (saves to video_brazilian_portuguese.srt)
sub-llama path/to/your/video.mkv

# Translates to Spanish (saves to video_spanish.srt)
sub-llama path/to/your/video.mkv Spanish
```

This will:

1. Extract the original English subtitle tracks to `path/to/your/video_en.srt`.
2. Translate all subtitle lines to your target language.
3. Save the translated subtitles into `path/to/your/video_<language>.srt`.

### 2. Translating Subtitle Files Directly

You can also translate an existing subtitle file (`.srt`) directly:

```bash
sub-llama --file path/to/subtitle.srt [original_language] [target_language]
```

If only one language is specified after the filename, it is treated as the target language:

```bash
# Translates to Brazilian Portuguese by default (saves to subtitle_brazilian_portuguese.srt)
sub-llama --file path/to/subtitle.srt

# Translates to Spanish (saves to subtitle_spanish.srt)
sub-llama --file path/to/subtitle.srt Spanish

# Translates from French to Spanish (saves to subtitle_spanish.srt)
sub-llama --file path/to/subtitle.srt French Spanish
```

---

## Disclaimer

This project is developed for educational and personal productivity purposes.
The author is not responsible for any misuse of this software, including the
unauthorized translation or distribution of copyrighted materials. Users are
solely responsible for ensuring they have the legal right to process and
translate the media files they input into this tool.
