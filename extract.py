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
        # Fallback to original text if translation fails
        print(f"\n⚠️ Error calling Ollama for '{text}': {e}. Using original text.")
        return text

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
    print(f"\n3. Translating {total_subs} lines to {language} using Ollama (kaelri/hy-mt2:1.8b)...")
    
    for idx, leg in enumerate(original_subs):
        original_text = leg.content.replace('\n', ' ')
        translation = translate_text_ollama(original_text, language=language)
        leg.content = translation
        
        # Show progress in the terminal
        progress = ((idx + 1) / total_subs) * 100
        print(f"-> Progress: {progress:.1f}% ({idx + 1}/{total_subs} lines translated)...", end='\r')
        sys.stdout.flush()

    print(f"\n\n4. Writing new subtitle in {language}...")
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(srt.compose(original_subs))
        
    print(f"\nSuccess! Subtitle in {language} saved to: {output_file}")



