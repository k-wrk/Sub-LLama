import subprocess
import os
import sys
import json
import urllib.request

LANGUAGE_CODES = {
    # Americas & Western Europe
    "english": "en",
    "spanish": "es",
    "portuguese": "pt",
    "brazilian portuguese": "pt-BR",
    "french": "fr",
    "german": "de",
    "italian": "it",
    "dutch": "nl",
    "greek": "el",
    "irish": "ga",
    "icelandic": "is",
    
    # Nordic & Baltic Europe
    "swedish": "sv",
    "norwegian": "no",
    "danish": "da",
    "finnish": "fi",
    "estonian": "et",
    "latvian": "lv",
    "lithuanian": "lt",
    
    # Eastern Europe
    "russian": "ru",
    "polish": "pl",
    "ukrainian": "uk",
    "czech": "cs",
    "romanian": "ro",
    "hungarian": "hu",
    "slovak": "sk",
    "bulgarian": "bg",
    "croatian": "hr",
    "serbian": "sr",
    "slovenian": "sl",
    
    # East Asia
    "chinese": "zh",
    "simplified chinese": "zh-Hans",
    "traditional chinese": "zh-Hant",
    "japanese": "ja",
    "korean": "ko",
    
    # Southeast Asia
    "vietnamese": "vi",
    "thai": "th",
    "indonesian": "id",
    "malay": "ms",
    "filipino": "fil",
    "tagalog": "tl",
    
    # Central & South Asia
    "hindi": "hi",
    "bengali": "bn",
    "punjabi": "pa",
    "marathi": "mr",
    "telugu": "te",
    "tamil": "ta",
    "gujarati": "gu",
    "kannada": "kn",
    "malayalam": "ml",
    
    # Middle East & Others
    "arabic": "ar",
    "turkish": "tr",
    "hebrew": "he",
    "persian": "fa",
    "farsi": "fa",
    "urdu": "ur"
}

def get_lang_suffix(language):
    lang_lower = language.strip().lower()
    if lang_lower in LANGUAGE_CODES:
        return LANGUAGE_CODES[lang_lower]
    return lang_lower.replace(" ", "_")


def get_ollama_url(host=None):
    if not host:
        host = os.environ.get("OLLAMA_HOST", "http://localhost:11434")
    if not host.startswith("http://") and not host.startswith("https://"):
        host = f"http://{host}"
    return host.rstrip("/") + "/api/chat"

import re
from concurrent.futures import ThreadPoolExecutor, as_completed
import threading

def is_untranslatable(text):
    """Check if a line contains only music notes, sound effect tags, punctuation, or numbers."""
    cleaned = text.strip()
    if not cleaned:
        return True
    # Only numbers and basic punctuation
    if re.fullmatch(r'[\d\s\.,;:!?\-\—\–\(\)\[\]"\'\/\\]+', cleaned):
        return True
    # Sound tags like [music], (applause), ♪ music ♪, etc.
    if re.fullmatch(r'[\[\(\{\<♪♫#\*].*?[\]\)\}\>♪♫#\*]', cleaned):
        return True
    return False

def translate_text_ollama(text, language="Brazilian Portuguese", original_language=None, model="kaelri/hy-mt2:1.8b", host=None):
    if is_untranslatable(text):
        return text

    url = get_ollama_url(host)
    source_phrase = f" from {original_language}" if original_language else ""
    data = {
        "model": model,
        "keep_alive": "15m",
        "options": {
            "temperature": 0.1
        },
        "messages": [
            {
                "role": "system",
                "content": f"Translate the input text{source_phrase} into {language}.\n- Output only the translation.\n- Preserve original formatting exactly (line breaks, spacing, paragraphs).\n- Do not modify HTML tags, placeholders, code, URLs, or special tokens.\n- Do not add explanations, comments, or extra text.\n- Keep meaning faithful and complete."
            },
            {
                "role": "user",
                "content": text
            }
        ],
        "stream": False
    }
    
    req = urllib.request.Request(
        url,
        data=json.dumps(data).encode('utf-8'),
        headers={'Content-Type': 'application/json'}
    )
    
    try:
        with urllib.request.urlopen(req) as response:
            res_data = json.loads(response.read().decode('utf-8'))
            return res_data['message']['content'].strip()
    except Exception as e:
        print(f"\n⚠️ Error calling Ollama for '{text}': {e}. Using original text.")
        return text

def translate_batch_ollama(lines, language="Brazilian Portuguese", original_language=None, model="kaelri/hy-mt2:1.8b", host=None):
    # If all lines in the batch are untranslatable, return directly
    if all(is_untranslatable(l) for l in lines):
        return list(lines)

    prompt_lines = [f"{i+1}: {line}" for i, line in enumerate(lines)]
    prompt_content = "\n".join(prompt_lines)
    
    url = get_ollama_url(host)
    source_phrase = f" from {original_language}" if original_language else ""
    data = {
        "model": model,
        "keep_alive": "15m",
        "options": {
            "temperature": 0.1
        },
        "messages": [
            {
                "role": "system",
                "content": (
                    f"Translate the following numbered lines{source_phrase} into {language}.\n"
                    "- Maintain the exact numbering format (e.g., '1: Translation') in your output.\n"
                    "- Output only the numbered translations, one per line.\n"
                    "- Do not add any extra text, explanations, or introductory remarks.\n"
                    "- Keep translations faithful and complete."
                )
            },
            {
                "role": "user",
                "content": prompt_content
            }
        ],
        "stream": False
    }
    
    req = urllib.request.Request(
        url,
        data=json.dumps(data).encode('utf-8'),
        headers={'Content-Type': 'application/json'}
    )
    
    with urllib.request.urlopen(req) as response:
        res_data = json.loads(response.read().decode('utf-8'))
        response_text = res_data['message']['content'].strip()
        
    # Parse the numbered lines
    translated_map = {}
    for line in response_text.split('\n'):
        line = line.strip()
        if not line:
            continue
        if ':' in line:
            parts = line.split(':', 1)
            try:
                num = int(parts[0].strip())
                text = parts[1].strip()
                translated_map[num] = text
            except ValueError:
                continue
                
    # Reassemble results checking if all lines exist
    results = []
    for i in range(1, len(lines) + 1):
        if i in translated_map:
            results.append(translated_map[i])
        else:
            raise ValueError(f"Missing line index {i} in response")
            
    return results


def run_translation_pipeline(original_subs, language="Brazilian Portuguese", original_language=None, model="kaelri/hy-mt2:1.8b", batch_size=30, workers=3, host=None):
    total_subs = len(original_subs)
    batch_size = max(1, batch_size)
    workers = max(1, workers)

    # Prepare batches
    batches = []
    idx = 0
    while idx < total_subs:
        end_idx = min(idx + batch_size, total_subs)
        lines = [leg.content.replace('\n', ' ') for leg in original_subs[idx:end_idx]]
        batches.append((idx, end_idx, lines))
        idx = end_idx

    translated_count = 0
    lock = threading.Lock()

    def process_batch(batch_info):
        start_i, end_i, lines_to_translate = batch_info
        try:
            translations = translate_batch_ollama(
                lines_to_translate,
                language=language,
                original_language=original_language,
                model=model,
                host=host
            )
        except Exception as e:
            # Fallback to line-by-line translation if the batch fails
            translations = []
            for line in lines_to_translate:
                if is_untranslatable(line):
                    translations.append(line)
                else:
                    translations.append(
                        translate_text_ollama(
                            line,
                            language=language,
                            original_language=original_language,
                            model=model,
                            host=host
                        )
                    )
        return start_i, end_i, translations

    # Execute in parallel with progress updates
    with ThreadPoolExecutor(max_workers=workers) as executor:
        futures = [executor.submit(process_batch, b) for b in batches]
        for future in as_completed(futures):
            start_i, end_i, translations = future.result()
            for i, trans in enumerate(translations):
                original_subs[start_i + i].content = trans

            with lock:
                translated_count += len(translations)
                progress = (translated_count / total_subs) * 100
                bar_length = 30
                filled = int(round(bar_length * progress / 100))
                bar = '█' * filled + '░' * (bar_length - filled)
                print(f"\r[{bar}] {progress:.1f}% ({translated_count}/{total_subs} lines translated)...", end='', flush=True)

    print()


def list_subtitle_tracks(video_path):
    cmd = [
        'ffprobe', '-v', 'error',
        '-show_entries', 'stream=index:stream_tags=language,title',
        '-select_streams', 's',
        '-of', 'json',
        video_path
    ]
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        data = json.loads(result.stdout)
        return data.get('streams', [])
    except Exception:
        return []


def translate_mkv(mkv_path, language="Brazilian Portuguese", track_index=0, model="kaelri/hy-mt2:1.8b", batch_size=30, workers=3, host=None):
    base_name = os.path.splitext(mkv_path)[0]
    en_file = f"{base_name}.en.srt"
    
    # Define the suffix based on the language in standard format (e.g. video.es.srt or video.pt-BR.srt)
    lang_suffix = get_lang_suffix(language)
    output_file = f"{base_name}.{lang_suffix}.srt"
    
    streams = list_subtitle_tracks(mkv_path)
    if streams:
        print("\n📽️ Subtitle tracks found in video:")
        for i, s in enumerate(streams):
            title = s.get('tags', {}).get('title', 'No Title')
            lang = s.get('tags', {}).get('language', 'unknown')
            print(f"  Track {i}: [{lang}] {title}")
        print(f"-> Using Track {track_index} (Use -t or --track to select another)\n")
    
    print("1. Extracting original subtitle with FFmpeg...")
    ffmpeg_cmd = ['ffmpeg', '-y', '-i', mkv_path, '-map', f'0:s:{track_index}', en_file]

    try:
        subprocess.run(ffmpeg_cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        print(f"-> English subtitle extracted: {en_file}")
    except subprocess.CalledProcessError:
        print("\n❌ Error: No internal subtitle track was found in this video.")
        print("💡 .mp4 files rarely contain embedded subtitles. Try downloading the English .srt subtitle separately.")
        sys.exit(1)
    except FileNotFoundError:
        print("\n❌ Error: FFmpeg is not installed on the system.")
        sys.exit(1)
    
    print("\n2. Reading subtitle file...")
    import srt
    with open(en_file, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_subs = list(srt.parse(content))
    total_subs = len(original_subs)
    
    print(f"\n3. Translating {total_subs} lines to {language} using Ollama (model: {model}, batch size: {batch_size}, workers: {workers})...")
    run_translation_pipeline(
        original_subs,
        language=language,
        original_language=None,
        model=model,
        batch_size=batch_size,
        workers=workers,
        host=host
    )

    print(f"\n4. Writing new subtitle in {language}...")
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(srt.compose(original_subs))
        
    print(f"\nSuccess! Subtitle in {language} saved to: {output_file}")
    return output_file


def translate_file(file_path, target_language="Brazilian Portuguese", original_language=None, model="kaelri/hy-mt2:1.8b", batch_size=30, workers=3, host=None):
    if not os.path.exists(file_path):
        print(f"Error: The file '{file_path}' was not found.")
        sys.exit(1)
        
    base_name = os.path.splitext(file_path)[0]
    lang_suffix = get_lang_suffix(target_language)
    output_file = f"{base_name}.{lang_suffix}.srt"
    
    print(f"1. Reading subtitle file '{file_path}'...")
    import srt
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_subs = list(srt.parse(content))
    total_subs = len(original_subs)
    
    source_info = f" from {original_language}" if original_language else ""
    print(f"\n2. Translating {total_subs} lines{source_info} to {target_language} using Ollama (model: {model}, batch size: {batch_size}, workers: {workers})...")
    run_translation_pipeline(
        original_subs,
        language=target_language,
        original_language=original_language,
        model=model,
        batch_size=batch_size,
        workers=workers,
        host=host
    )

    print(f"\n3. Writing new subtitle in {target_language}...")
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(srt.compose(original_subs))
        
    print(f"\nSuccess! Subtitle in {target_language} saved to: {output_file}")
    return output_file


def get_video_duration(video_path):
    cmd = [
        'ffprobe', '-v', 'error',
        '-show_entries', 'format=duration',
        '-of', 'default=noprint_wrappers=1:nokey=1',
        video_path
    ]
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        return float(result.stdout.strip())
    except Exception:
        return None


def embed_subtitle(video_path, subtitle_path, language="Brazilian Portuguese", in_place=False):
    if not os.path.exists(video_path):
        print(f"Error: The video file '{video_path}' was not found.")
        return False
    if not os.path.exists(subtitle_path):
        print(f"Error: The subtitle file '{subtitle_path}' was not found.")
        return False

    base, ext = os.path.splitext(video_path)
    output_path = f"{base}.embedded{ext}"
    
    lang_code = get_lang_suffix(language)
    s_codec = "mov_text" if ext.lower() == ".mp4" else "copy"
    
    total_seconds = get_video_duration(video_path)
    
    print(f"\n3. Embedding subtitle into {output_path} (Language: {language}, Code: {lang_code})...")
    
    original_subs = list_subtitle_tracks(video_path)
    new_sub_index = len(original_subs)

    ffmpeg_cmd = [
        'ffmpeg', '-y',
        '-hide_banner',
        '-loglevel', 'error',
        '-progress', '-',
        '-i', video_path,
        '-i', subtitle_path,
        '-map', '0:v:0', '-map', '0:a?', '-map', '0:s?', '-map', '1:s',
        '-c', 'copy',
        f'-c:s', s_codec,
        f'-metadata:s:s:{new_sub_index}', f'language={lang_code}',
        f'-metadata:s:s:{new_sub_index}', f'title={language}',
        f'-disposition:s:{new_sub_index}', 'default'
    ]

    for i in range(new_sub_index):
        ffmpeg_cmd.extend([f'-disposition:s:{i}', '0'])

    ffmpeg_cmd.append(output_path)
    
    try:
        process = subprocess.Popen(
            ffmpeg_cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )
        
        progress_info = {}
        for line in process.stdout:
            line = line.strip()
            if '=' in line:
                key, val = line.split('=', 1)
                progress_info[key] = val
                
            if line.startswith('progress='):
                out_time = progress_info.get('out_time', '00:00:00').split('.')[0]
                speed = progress_info.get('speed', '0.0x')
                
                if total_seconds:
                    try:
                        out_time_us = int(progress_info.get('out_time_us', 0))
                        current_seconds = out_time_us / 1000000.0
                        percent = min(100.0, max(0.0, (current_seconds / total_seconds) * 100))
                        
                        bar_length = 30
                        filled_length = int(round(bar_length * percent / 100))
                        bar = '█' * filled_length + '░' * (bar_length - filled_length)
                        
                        sys.stdout.write(f"\r[{bar}] {percent:.1f}% | Speed: {speed} | Time: {out_time}")
                    except Exception:
                        sys.stdout.write(f"\rMuxing... Speed: {speed} | Time: {out_time}")
                else:
                    sys.stdout.write(f"\rMuxing... Speed: {speed} | Time: {out_time}")
                sys.stdout.flush()
            
        process.wait()
        if process.returncode != 0:
            stderr_output = process.stderr.read()
            raise subprocess.CalledProcessError(process.returncode, ffmpeg_cmd, stderr=stderr_output)
            
        if in_place:
            try:
                os.replace(output_path, video_path)
                print(f"\n\nSuccess! Subtitled video saved in-place to: {video_path}")
            except Exception as e:
                print(f"\n\n⚠️ Error replacing original file: {e}. Embedded file saved to {output_path}")
                return False
        else:
            print(f"\n\nSuccess! Subtitled video saved to: {output_path}")
        return True
    except subprocess.CalledProcessError as e:
        print(f"\n❌ Error embedding subtitle: {e.stderr if hasattr(e, 'stderr') else e}")
        return False
    except FileNotFoundError:
        print("\n❌ Error: FFmpeg is not installed on the system.")
        return False







