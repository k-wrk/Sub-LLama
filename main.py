import sys
import os

def print_help():
    print("Sub-LLama - Local Subtitle Extractor and Translator")
    print("\nUsage:")
    print("  sub-llama <video.mkv> [target_language] [options]")
    print("  sub-llama -f, --file <subtitle.srt> [original_language] [target_language] [options]")
    print("  sub-llama -eo, --embed-only <video_file> <subtitle_file> [language] [-i]")
    print("  sub-llama -lt, --list-tracks <video_file>")
    print("\nOptions:")
    print("  -h, --help               Show this help message and exit")
    print("  -l, --languages          List all languages with automatic VLC naming support")
    print("  -lt, --list-tracks       List all subtitle tracks in a video file and exit")
    print("  -f, --file               Translate a subtitle file directly instead of extracting from MKV")
    print("  -eo, --embed-only        Mux/embed an existing subtitle file into a video without translating")
    print("  -t, --track <index>      Specify which subtitle track index to extract from MKV (default: 0)")
    print("  -n, -ne, --no-embed      Do not mux/embed the translated subtitle into the video container")
    print("  -i, --in-place           Mux/embed the subtitle directly into the original video file (overwrites it)")
    print("  -m, --model <model>      Ollama model to use for translation (default: kaelri/hy-mt2:1.8b)")
    print("  -b, --batch-size <size>  Number of subtitle lines to translate per batch (default: 30)")
    print("  -w, --workers <count>    Number of concurrent translation workers (default: 3)")
    print("  --host <url>             Ollama server URL/host (default: http://localhost:11434)")
    print("\nLanguages:")
    print("  You must pass languages in English (e.g. \"Brazilian Portuguese\" or Spanish).")
    print("  If a language name contains spaces, enclose it in double quotes, e.g.: \"Brazilian Portuguese\".")
    print("\nExamples:")
    print("  sub-llama movie.mkv Spanish")
    print("  sub-llama movie.mkv Spanish -i")
    print("  sub-llama movie.mkv Spanish -n")
    print("  sub-llama movie.mkv \"Brazilian Portuguese\" -t 1 -w 4 -b 40")
    print("  sub-llama -f subtitle.srt English \"Brazilian Portuguese\"")
    print("  sub-llama -eo movie.mp4 subtitle.srt \"Brazilian Portuguese\" -i")
    print("  sub-llama -lt movie.mkv")
    print("  sub-llama -l")

def print_languages():
    from extract import LANGUAGE_CODES
    print("Languages with automatic VLC naming support (suffix):")
    print("-" * 55)
    for lang, suffix in sorted(LANGUAGE_CODES.items()):
        display_name = " ".join(word.capitalize() for word in lang.split())
        print(f"  {display_name:<25} -> .{suffix}.srt")
    print("-" * 55)
    print("\nNote: You can pass any other language. The translation model")
    print("supports many more languages. If a language is not on this list,")
    print("it will fallback to a slugified name suffix (e.g., '.finnish.srt').")

def main():
    raw_args = sys.argv[1:]
    
    if not raw_args:
        print("Error: You must provide a video file or use the -f/--file option.")
        print("Run 'sub-llama --help' for options and list of common languages.")
        sys.exit(1)
        
    if raw_args[0] in ("-h", "--help"):
        print_help()
        sys.exit(0)
        
    if raw_args[0] in ("-l", "--languages"):
        print_languages()
        sys.exit(0)

    # Parsing flags and arguments
    args = []
    file_mode = False
    embed_only_mode = False
    list_tracks_mode = False
    embed_after_translation = True
    in_place = False
    track_index = 0
    model = "kaelri/hy-mt2:1.8b"
    batch_size = 30
    workers = 3
    host = None

    i = 0
    while i < len(raw_args):
        arg = raw_args[i]

        if arg in ("-h", "--help"):
            print_help()
            sys.exit(0)
        elif arg in ("-l", "--languages"):
            print_languages()
            sys.exit(0)
        elif arg in ("-lt", "--list-tracks"):
            list_tracks_mode = True
        elif arg in ("-f", "--file"):
            file_mode = True
        elif arg in ("-eo", "--embed-only"):
            embed_only_mode = True
        elif arg in ("-n", "-ne", "--no-embed"):
            embed_after_translation = False
        elif arg in ("-i", "--in-place", "--original"):
            in_place = True
        elif arg in ("--embed", "-e"):
            # Deprecated flag, ignore gracefully
            pass
        elif arg in ("-t", "--track"):
            if i + 1 >= len(raw_args):
                print("Error: Option -t/--track requires an integer argument.")
                sys.exit(1)
            try:
                track_index = int(raw_args[i + 1])
                i += 1
            except ValueError:
                print(f"Error: Invalid track index '{raw_args[i + 1]}'. Must be an integer.")
                sys.exit(1)
        elif arg in ("-m", "--model"):
            if i + 1 >= len(raw_args):
                print("Error: Option -m/--model requires a model name argument.")
                sys.exit(1)
            model = raw_args[i + 1]
            i += 1
        elif arg in ("-b", "--batch-size"):
            if i + 1 >= len(raw_args):
                print("Error: Option -b/--batch-size requires an integer argument.")
                sys.exit(1)
            try:
                batch_size = int(raw_args[i + 1])
                i += 1
            except ValueError:
                print(f"Error: Invalid batch size '{raw_args[i + 1]}'. Must be an integer.")
                sys.exit(1)
        elif arg in ("-w", "--workers"):
            if i + 1 >= len(raw_args):
                print("Error: Option -w/--workers requires an integer argument.")
                sys.exit(1)
            try:
                workers = int(raw_args[i + 1])
                i += 1
            except ValueError:
                print(f"Error: Invalid workers count '{raw_args[i + 1]}'. Must be an integer.")
                sys.exit(1)
        elif arg == "--host":
            if i + 1 >= len(raw_args):
                print("Error: Option --host requires a URL argument.")
                sys.exit(1)
            host = raw_args[i + 1]
            i += 1
        else:
            args.append(arg)
        i += 1

    if list_tracks_mode:
        if not args:
            print("Error: You must specify a video file path after -lt/--list-tracks.")
            sys.exit(1)
        video_path = args[0]
        if not os.path.exists(video_path):
            print(f"Error: The file '{video_path}' was not found.")
            sys.exit(1)
            
        from extract import list_subtitle_tracks
        streams = list_subtitle_tracks(video_path)
        if streams:
            print(f"\n📽️ Subtitle tracks found in: {video_path}")
            for idx, s in enumerate(streams):
                title = s.get('tags', {}).get('title', 'No Title')
                lang = s.get('tags', {}).get('language', 'unknown')
                print(f"  Track {idx}: [{lang}] {title}")
            print()
        else:
            print("\n❌ No subtitle tracks found in this video or the file is invalid.")
        sys.exit(0)

    if embed_only_mode:
        if len(args) < 2:
            print("Error: You must specify a video file and a subtitle file after -eo/--embed-only.")
            print("Example: sub-llama -eo video.mp4 subtitle.srt [language] [-i]")
            sys.exit(1)
        video_path = args[0]
        subtitle_path = args[1]
        language = args[2] if len(args) > 2 else "Brazilian Portuguese"
        
        from extract import embed_subtitle
        embed_subtitle(video_path, subtitle_path, language, in_place=in_place)
        sys.exit(0)

    if file_mode:
        if not args:
            print("Error: You must specify a file path after -f/--file.")
            print("Example: sub-llama -f subtitle.srt [original_language] [target_language]")
            sys.exit(1)
        
        file_path = args[0]
        original_language = None
        target_language = "Brazilian Portuguese"
        
        if len(args) == 2:
            target_language = args[1]
        elif len(args) > 2:
            original_language = args[1]
            target_language = args[2]
            
        if not os.path.exists(file_path):
            print(f"Error: The file '{file_path}' was not found.")
            sys.exit(1)
            
        source_info = f" (Source: {original_language})" if original_language else ""
        print(f"📄 Processing file... {file_path}{source_info} (Target language: {target_language})")
        from extract import translate_file
        translate_file(file_path, target_language, original_language, model=model, batch_size=batch_size, workers=workers, host=host)
    else:
        if not args:
            print("Error: You must provide a video file.")
            print("Run 'sub-llama --help' for usage options.")
            sys.exit(1)

        video_path = args[0]
        language = args[1] if len(args) > 1 else "Brazilian Portuguese"
            
        if not os.path.exists(video_path):
            print(f"Error: The file '{video_path}' was not found.")
            sys.exit(1)

        print(f"📽️ Processing... {video_path} (Target language: {language})")
        from extract import translate_mkv
        output_sub = translate_mkv(video_path, language, track_index, model=model, batch_size=batch_size, workers=workers, host=host)
        if embed_after_translation and output_sub:
            from extract import embed_subtitle
            embed_subtitle(video_path, output_sub, language, in_place=in_place)


if __name__ == "__main__":
    main()
