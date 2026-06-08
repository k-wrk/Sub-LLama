import sys, os
from extract import translate_mkv

def main():
    if len(sys.argv) < 2:
        print("Error: You must provide the path to the MKV file.")
        print("Example: sub-llama video.mkv [language]")
        print("Default language: Brazilian Portuguese")
        sys.exit(1)
        
    video_path = sys.argv[1]
    language = sys.argv[2] if len(sys.argv) > 2 else "Brazilian Portuguese"
        
    if not os.path.exists(video_path):
        print(f"Error: The file '{video_path}' was not found.")
        sys.exit(1)

    print(f"📽️ Processing... {video_path} (Target language: {language})")
    translate_mkv(video_path, language)


if __name__ == "__main__":
    main()


