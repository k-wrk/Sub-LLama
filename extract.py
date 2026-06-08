import subprocess
import os
import sys
import srt
import json
import urllib.request

def translate_text_ollama(text, language="Brazilian Portuguese", model="kaelri/hy-mt2:1.8b"):
    url = "http://localhost:11434/api/chat"
    data = {
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": f"Translate the input text into {language}.\n- Output only the translation.\n- Preserve original formatting exactly (line breaks, spacing, paragraphs).\n- Do not modify HTML tags, placeholders, code, URLs, or special tokens.\n- Do not add explanations, comments, or extra text.\n- Keep meaning faithful and complete."
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

def translate_batch_ollama(lines, language="Brazilian Portuguese", model="kaelri/hy-mt2:1.8b"):
    prompt_lines = [f"{i+1}: {line}" for i, line in enumerate(lines)]
    prompt_content = "\n".join(prompt_lines)
    
    url = "http://localhost:11434/api/chat"
    data = {
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": (
                    f"Translate the following numbered lines into {language}.\n"
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

def translate_mkv(mkv_path, language="Brazilian Portuguese"):
    base_name = os.path.splitext(mkv_path)[0]
    en_file = f"{base_name}_en.srt"
    
    # Define the suffix based on the language (e.g. _spanish.srt or _portuguese.srt)
    lang_suffix = language.lower().replace(" ", "_")
    output_file = f"{base_name}_{lang_suffix}.srt"
    
    print("1. Extracting original subtitle with FFmpeg...")
    ffmpeg_cmd = ['ffmpeg', '-y', '-i', mkv_path, '-map', '0:s:0', en_file]
    
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
    with open(en_file, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_subs = list(srt.parse(content))
    total_subs = len(original_subs)
    batch_size = 10
    
    print(f"\n3. Translating {total_subs} lines to {language} using Ollama in batches of {batch_size} (kaelri/hy-mt2:1.8b)...")
    
    idx = 0
    while idx < total_subs:
        current_batch = original_subs[idx : idx + batch_size]
        lines_to_translate = [leg.content.replace('\n', ' ') for leg in current_batch]
        
        try:
            # Try to translate the whole batch at once
            translations = translate_batch_ollama(lines_to_translate, language=language)
            for i, trans in enumerate(translations):
                current_batch[i].content = trans
        except Exception as e:
            # Fallback to line-by-line translation if the batch fails or is misformatted
            # Print on a new line to avoid messing up the status indicator
            print(f"\n⚠️ Batch translation failed (Index {idx} to {idx + len(current_batch)}): {e}. Retrying line-by-line...")
            for leg in current_batch:
                texto_original = leg.content.replace('\n', ' ')
                leg.content = translate_text_ollama(texto_original, language=language)
        
        idx += len(current_batch)
        progress = (idx / total_subs) * 100
        print(f"-> Progress: {progress:.1f}% ({idx}/{total_subs} lines translated)...", end='\r')
        sys.stdout.flush()

    print(f"\n\n4. Writing new subtitle in {language}...")
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(srt.compose(original_subs))
        
    print(f"\nSuccess! Subtitle in {language} saved to: {output_file}")



