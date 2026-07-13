import sys, os
from extract import translate_mkv, translate_file

def print_help():
    print("Sub-LLama - Local Subtitle Extractor and Translator")
    print("\nUsage:")
    print("  sub-llama <video.mkv> [target_language]")
    print("  sub-llama --file <subtitle.srt> [original_language] [target_language]")
    print("\nOptions:")
    print("  -h, --help     Show this help message and exit")
    print("  -l, --languages List all languages with automatic VLC naming support")
    print("  --file         Translate a subtitle file directly instead of extracting from MKV")
    print("\nLanguages:")
    print("  You must pass languages in English (e.g. \"Brazilian Portuguese\" or Spanish).")
    print("  If a language name contains spaces, enclose it in double quotes, e.g.: \"Brazilian Portuguese\".")
    print("\nExamples:")
    print("  sub-llama movie.mkv Spanish")
    print("  sub-llama --file subtitle.srt English \"Brazilian Portuguese\"")
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
        
    if sys.argv[1] == "--file":
        if len(sys.argv) < 3:
            print("Error: You must specify a file path after --file.")
            print("Example: sub-llama --file subtitle.srt [original_language] [target_language]")
            sys.exit(1)
        
        file_path = sys.argv[2]
        original_language = None
        target_language = "Brazilian Portuguese"
        
        if len(sys.argv) == 4:
            target_language = sys.argv[3]
        elif len(sys.argv) > 4:
            original_language = sys.argv[3]
            target_language = sys.argv[4]
            
        if not os.path.exists(file_path):
            print(f"Error: The file '{file_path}' was not found.")
            sys.exit(1)
            
        source_info = f" (Source: {original_language})" if original_language else ""
        print(f"📄 Processing file... {file_path}{source_info} (Target language: {target_language})")
        translate_file(file_path, target_language, original_language)
    else:
        video_path = sys.argv[1]
        language = sys.argv[2] if len(sys.argv) > 2 else "Brazilian Portuguese"
            
        if not os.path.exists(video_path):
            print(f"Error: The file '{video_path}' was not found.")
            sys.exit(1)

        print(f"📽️ Processing... {video_path} (Target language: {language})")
        translate_mkv(video_path, language)


if __name__ == "__main__":
    main()


