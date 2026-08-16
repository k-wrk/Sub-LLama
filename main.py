import sys, os
from extract import translate_mkv, translate_file

def print_help():
    print("Sub-LLama - Local Subtitle Extractor and Translator")
    print("\nUsage:")
    print("  sub-llama <video.mkv> [target_language] [--embed] [--track <index>]")
    print("  sub-llama --file <subtitle.srt> [original_language] [target_language]")
    print("  sub-llama --embed-only <video_file> <subtitle_file> [language]")
    print("  sub-llama --list-tracks <video_file>")
    print("\nOptions:")
    print("  -h, --help        Show this help message and exit")
    print("  -l, --languages    List all languages with automatic VLC naming support")
    print("  --file            Translate a subtitle file directly instead of extracting from MKV")
    print("  --embed, -e       Mux/embed the translated subtitle into the video container at the end")
    print("  --embed-only      Mux/embed an existing subtitle file into a video without translating")
    print("  --track, -t       Specify which subtitle track index to extract from MKV (default is 0)")
    print("  --list-tracks, -lt List all subtitle tracks in a video file and exit")
    print("\nLanguages:")
    print("  You must pass languages in English (e.g. \"Brazilian Portuguese\" or Spanish).")
    print("  If a language name contains spaces, enclose it in double quotes, e.g.: \"Brazilian Portuguese\".")
    print("\nExamples:")
    print("  sub-llama movie.mkv Spanish --embed")
    print("  sub-llama --file subtitle.srt English \"Brazilian Portuguese\"")
    print("  sub-llama --embed-only movie.mp4 subtitle.srt \"Brazilian Portuguese\"")
    print("  sub-llama --languages (to view all supported languages)")

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
    if len(sys.argv) < 2:
        print("Error: You must provide a video file or use the --file option.")
        print("Run 'sub-llama --help' for options and list of common languages.")
        sys.exit(1)
        
    if sys.argv[1] in ("-h", "--help"):
        print_help()
        sys.exit(0)
        
    if sys.argv[1] in ("-l", "--languages"):
        print_languages()
        sys.exit(0)
        
    if sys.argv[1] in ("--list-tracks", "-lt"):
        if len(sys.argv) < 3:
            print("Error: You must specify a video file path after --list-tracks.")
            sys.exit(1)
        video_path = sys.argv[2]
        if not os.path.exists(video_path):
            print(f"Error: The file '{video_path}' was not found.")
            sys.exit(1)
            
        from extract import list_subtitle_tracks
        streams = list_subtitle_tracks(video_path)
        if streams:
            print(f"\n📽️ Subtitle tracks found in: {video_path}")
            for i, s in enumerate(streams):
                title = s.get('tags', {}).get('title', 'No Title')
                lang = s.get('tags', {}).get('language', 'unknown')
                print(f"  Track {i}: [{lang}] {title}")
            print()
        else:
            print("\n❌ No subtitle tracks found in this video or the file is invalid.")
        sys.exit(0)
        
    if sys.argv[1] == "--embed-only":
        if len(sys.argv) < 4:
            print("Error: You must specify a video file and a subtitle file after --embed-only.")
            print("Example: sub-llama --embed-only video.mp4 subtitle.srt [language]")
            sys.exit(1)
        video_path = sys.argv[2]
        subtitle_path = sys.argv[3]
        language = sys.argv[4] if len(sys.argv) > 4 else "Brazilian Portuguese"
        
        from extract import embed_subtitle
        embed_subtitle(video_path, subtitle_path, language)
        sys.exit(0)
        
    embed_after_translation = False
    args = sys.argv[1:]
    if "--embed" in args:
        embed_after_translation = True
        args.remove("--embed")
    if "-e" in args:
        embed_after_translation = True
        args.remove("-e")

    track_index = 0
    if "--track" in args:
        idx = args.index("--track")
        if idx + 1 < len(args):
            try:
                track_index = int(args[idx + 1])
                args.pop(idx + 1)
            except ValueError:
                print("Error: Track index must be an integer.")
                sys.exit(1)
        args.pop(idx)
    elif "-t" in args:
        idx = args.index("-t")
        if idx + 1 < len(args):
            try:
                track_index = int(args[idx + 1])
                args.pop(idx + 1)
            except ValueError:
                print("Error: Track index must be an integer.")
                sys.exit(1)
        args.pop(idx)

    if args[0] == "--file":
        if len(args) < 2:
            print("Error: You must specify a file path after --file.")
            print("Example: sub-llama --file subtitle.srt [original_language] [target_language]")
            sys.exit(1)
        
        file_path = args[1]
        original_language = None
        target_language = "Brazilian Portuguese"
        
        if len(args) == 3:
            target_language = args[2]
        elif len(args) > 3:
            original_language = args[2]
            target_language = args[3]
            
        if not os.path.exists(file_path):
            print(f"Error: The file '{file_path}' was not found.")
            sys.exit(1)
            
        source_info = f" (Source: {original_language})" if original_language else ""
        print(f"📄 Processing file... {file_path}{source_info} (Target language: {target_language})")
        translate_file(file_path, target_language, original_language)
    else:
        video_path = args[0]
        language = args[1] if len(args) > 1 else "Brazilian Portuguese"
            
        if not os.path.exists(video_path):
            print(f"Error: The file '{video_path}' was not found.")
            sys.exit(1)

        print(f"📽️ Processing... {video_path} (Target language: {language})")
        output_sub = translate_mkv(video_path, language, track_index)
        if embed_after_translation and output_sub:
            from extract import embed_subtitle
            embed_subtitle(video_path, output_sub, language)


if __name__ == "__main__":
    main()


