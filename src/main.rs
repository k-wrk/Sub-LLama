mod config;
mod ct2_engine;
mod languages;
mod media;
mod srt;

use anyhow::{bail, Context, Result};
use clap::Parser;
use colored::Colorize;
use ct2_engine::Ct2Translator;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use languages::LanguageDefinition;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "sub-llama")]
#[command(
    about = "Ultra-fast local subtitle translator in Rust using CTranslate2 (INT8 MarianMT)",
    long_about = None
)]
struct Cli {
    /// Video or subtitle file (.mkv, .mp4, .srt) OR folder path
    #[arg(value_name = "INPUT_PATH")]
    input_path: Option<PathBuf>,

    /// Batch directory mode: translate all video/sub files in directory
    #[arg(short = 'd', long = "dir")]
    dir_mode: Option<PathBuf>,

    /// Recursive search in subdirectories when using directory mode
    #[arg(short = 'r', long = "recursive")]
    recursive: bool,

    /// Force translation even if target language already exists
    #[arg(long = "force")]
    force: bool,

    /// Target language code or name (e.g. pt, es, fr, ar, zh, ja). Default from config or 'pt'
    #[arg(long = "to")]
    to_lang: Option<String>,

    /// List all supported target languages and exit
    #[arg(long = "list-languages")]
    list_languages: bool,

    /// Subtitle file mode: translate directly without video extraction
    #[arg(short = 'f', long = "file")]
    file_mode: bool,

    /// List subtitle tracks in the video and exit
    #[arg(short = 'l', long = "list-tracks")]
    list_tracks: bool,

    /// Number of concurrent translation worker threads (default: 2)
    #[arg(short = 'w', long = "workers", default_value_t = 2)]
    workers: usize,

    /// Subtitle track index to extract (default: 0)
    #[arg(short = 't', long = "track")]
    track: Option<usize>,

    /// Do not embed translated subtitle into video container (generates .srt only)
    #[arg(short = 'n', long = "no-embed")]
    no_embed: bool,

    /// Keep the generated .srt subtitle file on disk (by default, embedded directly with zero leftover files)
    #[arg(long = "keep-subs")]
    keep_subs: bool,

    /// Generate a new '.translated.mkv' file instead of updating the original video in-place
    #[arg(long = "new-file")]
    new_file: bool,

    /// Custom models directory (overrides config.toml and defaults to ~/.cache/sub-llama/models)
    #[arg(long = "models-dir")]
    models_dir: Option<PathBuf>,
}

fn print_supported_languages() {
    println!("\n{}", "Supported target languages for direct translation (--to):".bold().cyan());
    println!("{:-<85}", "");
    println!(
        "  {:<8} | {:<24} | {:<14} | {:<10} | {}",
        "CODE".bold(),
        "NAME".bold(),
        "FAMILY".bold(),
        "VLC SUFFIX".bold(),
        "MODEL REPOSITORY".bold()
    );
    println!("{:-<85}", "");
    for lang in languages::SUPPORTED_LANGUAGES {
        println!(
            "  {:<8} | {:<24} | {:<14} | {:<10} | {}",
            lang.code.green(),
            lang.name,
            lang.family.magenta(),
            lang.vlc_suffix.yellow(),
            lang.ct2_repo.unwrap_or(lang.hf_repo).white()
        );
    }
    println!("{:-<85}", "");
    println!("Usage examples: sub-llama video.mkv --to es");
    println!("                sub-llama video.mkv --to fr");
    println!("                sub-llama --dir ./season/ --to pt");
    println!("                sub-llama -f subtitle.srt --to it\n");
}

fn resolve_models_dir(cli_dir: Option<PathBuf>, cfg_dir_str: &str) -> PathBuf {
    if let Some(p) = cli_dir {
        return config::expand_tilde(p);
    }
    if let Ok(env_val) = std::env::var("SUB_LLAMA_MODELS_DIR").or_else(|_| std::env::var("SUBS2PT_MODELS_DIR")) {
        if !env_val.is_empty() {
            return config::expand_tilde(PathBuf::from(env_val));
        }
    }
    config::expand_tilde(cfg_dir_str)
}

fn find_media_files_in_dir(dir: &Path, recursive: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_media_files(dir, recursive, &mut files);
    files.sort();
    files
}

fn collect_media_files(dir: &Path, recursive: bool, list: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && recursive {
                collect_media_files(&path, true, list);
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if ["mkv", "mp4", "srt", "avi", "webm"].contains(&ext_lower.as_str()) {
                        // Evita pegar arquivos temporários
                        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if !fname.starts_with('.') && !fname.contains(".tmp_embed_") && !fname.contains("tmp_") {
                            list.push(path);
                        }
                    }
                }
            }
        }
    }
}


/// Checks whether a file already contains the target language and should be skipped.
fn should_skip_file(path: &Path, lang_def: &LanguageDefinition, force: bool) -> (bool, &'static str) {
    if force {
        return (false, "Forced by flag");
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "srt" {
        // Loose subtitle file:
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // Do not translate files that are already translations (e.g. movie.pt-BR.srt)
        for other_lang in languages::SUPPORTED_LANGUAGES {
            if fname.ends_with(&format!(".{}.srt", other_lang.vlc_suffix))
                || fname.ends_with(&format!(".{}.srt", other_lang.code))
            {
                return (true, "Already a translated subtitle");
            }
        }

        // Check if destination translation already exists in the same folder
        let file_stem = path.file_stem().unwrap().to_str().unwrap();
        let expected_output = path.with_file_name(format!("{}.{}.srt", file_stem, lang_def.vlc_suffix));
        if expected_output.exists() {
            return (true, "Translated .srt file already exists");
        }
        (false, "")
    } else {
        // Video file (mkv, mp4, etc.):
        // 1. Check if it already contains a subtitle track matching the target language
        if media::has_language_track(path, lang_def.match_codes) {
            return (true, "Video already contains track in this language");
        }

        // 2. Check if an external .srt file exists alongside the video
        let file_stem = path.file_stem().unwrap().to_str().unwrap();
        let srt_alongside = path.with_file_name(format!("{}.{}.srt", file_stem, lang_def.vlc_suffix));
        if srt_alongside.exists() {
            return (true, "External subtitle already exists");
        }

        (false, "")
    }
}

fn process_single_file(
    file_path: &Path,
    translator: &Ct2Translator,
    lang_def: &LanguageDefinition,
    cli: &Cli,
    in_place_mode: bool,
) -> Result<()> {
    let is_srt_input = cli.file_mode
        || file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("srt"))
            .unwrap_or(false);

    let chosen_track = cli.track.unwrap_or(0);

    let srt_content = if is_srt_input {
        fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read subtitle file {}", file_path.display()))?
    } else {
        let extract_spinner = ProgressBar::new_spinner();
        extract_spinner.set_style(
            ProgressStyle::default_spinner()
                .template("  {spinner:.cyan} Extracting subtitle from video...")
                .unwrap(),
        );
        extract_spinner.enable_steady_tick(std::time::Duration::from_millis(80));

        let extracted = media::extract_subtitle_to_memory(file_path, chosen_track)?;
        extract_spinner.finish_with_message("  Extraction completed to memory!".green().to_string());
        extracted
    };

    let mut entries = srt::parse_srt(&srt_content);
    println!("  Lines to translate: {}", entries.len().to_string().cyan());

    let pb = ProgressBar::new(entries.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  [{elapsed_precise}] [{bar:35.cyan/blue}] {pos}/{len} ({per_sec}, ETA: {eta})")
            .unwrap()
            .progress_chars("━╸ "),
    );

    let start_translate = Instant::now();

    // Identificar índices que precisam de tradução
    let mut indices_to_translate = Vec::new();
    for (idx, entry) in entries.iter().enumerate() {
        if !srt::is_untranslatable(&entry.text) && !entry.text.trim().is_empty() {
            indices_to_translate.push(idx);
        } else {
            pb.inc(1);
        }
    }

    let batch_size = 32;
    for chunk in indices_to_translate.chunks(batch_size) {
        let texts: Vec<&str> = chunk.iter().map(|&idx| entries[idx].text.as_str()).collect();
        match translator.translate_batch(&texts) {
            Ok(translations) => {
                for (i, &idx) in chunk.iter().enumerate() {
                    entries[idx].text = translations[i].clone();
                    pb.inc(1);
                }
            }
            Err(e) => {
                eprintln!("\nWarning: error translating batch: {}. Retrying sequentially...", e);
                for &idx in chunk {
                    if let Ok(t) = translator.translate_batch(&[&entries[idx].text]) {
                        if let Some(first) = t.into_iter().next() {
                            entries[idx].text = first;
                        }
                    }
                    pb.inc(1);
                }
            }
        }
    }

    let translate_duration = start_translate.elapsed().as_secs_f32();
    pb.finish();
    println!(
        "  {}",
        format!(
            "Translation finished in {:.2}s ({:.1} lines/s)",
            translate_duration,
            entries.len() as f32 / translate_duration.max(0.001)
        )
        .green()
    );

    let serialized = srt::serialize_srt(&entries);

    // Save SRT file to disk if input was a standalone subtitle, or if explicitly requested (--keep-subs / --no-embed)
    if is_srt_input || cli.keep_subs || cli.no_embed {
        let file_stem = file_path.file_stem().unwrap().to_str().unwrap();
        let output_srt_path = file_path.with_file_name(format!("{}.{}.srt", file_stem, lang_def.vlc_suffix));
        fs::write(&output_srt_path, &serialized)
            .with_context(|| format!("Failed to save subtitle to {}", output_srt_path.display()))?;
        println!("  Saved subtitle to: {}", output_srt_path.display().to_string().cyan());
    }

    // Embed into video container
    if !is_srt_input && !cli.no_embed {
        let embed_spinner = ProgressBar::new_spinner();
        embed_spinner.set_style(
            ProgressStyle::default_spinner()
                .template("  {spinner:.magenta} Embedding subtitle into video via FFmpeg...")
                .unwrap(),
        );
        embed_spinner.enable_steady_tick(std::time::Duration::from_millis(80));

        if in_place_mode {
            media::embed_subtitle_in_place(
                file_path,
                &serialized,
                lang_def.code,
                lang_def.track_title,
            )?;
            embed_spinner.finish_with_message("  Original video updated successfully!".green().to_string());
        } else {
            let final_output = file_path.with_file_name(format!(
                "{}.translated.{}",
                file_path.file_stem().unwrap().to_str().unwrap(),
                file_path.extension().unwrap().to_str().unwrap()
            ));
            let tmp_srt = std::env::temp_dir().join(format!("sub_llama_{}.srt", std::process::id()));
            fs::write(&tmp_srt, &serialized)?;

            let res = (|| -> Result<()> {
                let mut cmd = std::process::Command::new("ffmpeg");
                cmd.args(["-y", "-v", "error", "-i"])
                    .arg(file_path)
                    .args(["-i"])
                    .arg(&tmp_srt)
                    .args([
                        "-map", "0:v:0", "-map", "0:a?", "-map", "0:s?", "-map", "1:s",
                        "-c", "copy",
                        "-metadata:s:s", &format!("language={}", lang_def.code),
                        "-metadata:s:s", &format!("title={}", lang_def.track_title),
                        "-disposition:s", "default",
                    ])
                    .arg(&final_output);
                let out = cmd.output()?;
                if !out.status.success() {
                    bail!("ffmpeg failed");
                }
                Ok(())
            })();
            let _ = fs::remove_file(&tmp_srt);
            res?;
            embed_spinner.finish_with_message(format!(
                "  New video saved to: {}",
                final_output.display().to_string().green()
            ));
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // --list-languages
    if cli.list_languages {
        print_supported_languages();
        return Ok(());
    }

    let theme = ColorfulTheme::default();

    // Carregar configuração TOML
    let app_cfg = config::load_or_create_config()?;

    // Resolve target language
    let selected_lang_code: String = match cli.to_lang {
        Some(ref l) => l.clone(),
        None => {
            let lang_labels: Vec<String> = languages::SUPPORTED_LANGUAGES
                .iter()
                .map(|l| format!("{:<5} - {} ({})", l.code, l.name, l.family))
                .collect();

            let default_idx = languages::SUPPORTED_LANGUAGES
                .iter()
                .position(|l| l.code == app_cfg.default_target_lang)
                .unwrap_or(0);

            let selection = Select::with_theme(&theme)
                .with_prompt("Which language would you like to translate to?")
                .default(default_idx)
                .items(&lang_labels)
                .interact()?;

            languages::SUPPORTED_LANGUAGES[selection].code.to_string()
        }
    };

    let lang_def = match languages::find_language(&selected_lang_code) {
        Some(l) => l,
        None => {
            bail!(
                "Language '{}' not recognized. Use --list-languages to view supported languages.",
                selected_lang_code
            );
        }
    };

    // Check directory mode (--dir or positional dir path)
    let explicit_dir = cli.dir_mode.clone().or_else(|| {
        if let Some(ref p) = cli.input_path {
            if p.is_dir() {
                return Some(p.clone());
            }
        }
        None
    });

    // LIST TRACKS MODE
    if cli.list_tracks {
        let target_video = match cli.input_path {
            Some(ref p) if p.is_file() => p,
            _ => {
                eprintln!("{}", "Error: Please specify a video file to list subtitle tracks.".red());
                std::process::exit(1);
            }
        };
        let tracks = media::list_tracks(target_video)?;
        println!("\nSubtitle tracks found in '{}':", target_video.display().to_string().cyan());
        println!("{:-<60}", "");
        for t in tracks {
            let lang = t.language.as_deref().unwrap_or("und");
            let title = t.title.as_deref().unwrap_or("Untitled");
            println!(
                "  [Track {}] Codec: {:<8} Language: {:<6} Title: {}",
                t.index.to_string().yellow(),
                t.codec,
                lang.green(),
                title
            );
        }
        println!("{:-<60}", "");
        return Ok(());
    }

    // Load CTranslate2 INT8 model
    let base_models_dir = resolve_models_dir(cli.models_dir.clone(), &app_cfg.models_dir);
    let model_folder_name = lang_def.ct2_repo.unwrap_or(lang_def.hf_repo).replace('/', "--");
    let model_dir = base_models_dir.join(&model_folder_name);

    println!("\n{} {} [{}]", "Target Language :".bold(), lang_def.name.cyan(), lang_def.code.yellow());
    println!("{} {}", "AI Model        :".bold(), lang_def.ct2_repo.unwrap_or(lang_def.hf_repo).white());

    let has_weights = model_dir.join("model.bin").exists();

    if !has_weights {
        Ct2Translator::download_files(lang_def, &model_dir)?;
    }

    let load_spinner = ProgressBar::new_spinner();
    load_spinner.set_style(
        ProgressStyle::default_spinner()
            .template("  {spinner:.yellow} Loading CTranslate2 model into memory...")
            .unwrap(),
    );
    load_spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let start_load = Instant::now();
    let translator = Ct2Translator::load(&model_dir, lang_def)
        .context("Failed to load CTranslate2 translation engine")?;
    load_spinner.finish_with_message(format!(
        "  {}",
        format!("Model loaded in {:.2}s!", start_load.elapsed().as_secs_f32()).green()
    ));

    // ==========================================
    // FLOW 1: DIRECTORY BATCH PROCESSING
    // ==========================================
    if let Some(ref dir_path) = explicit_dir {
        println!("\n{} {}", "Directory Mode:".bold().magenta(), dir_path.display().to_string().cyan());
        println!("{} {}\n", "Scanning media files for:".white(), lang_def.name.yellow());

        let all_files = find_media_files_in_dir(dir_path, cli.recursive);
        if all_files.is_empty() {
            println!("{}", "No video or subtitle files found in directory.".yellow());
            return Ok(());
        }

        let mut queue = Vec::new();
        let mut skipped_count = 0;

        for file in &all_files {
            let (skip, reason) = should_skip_file(file, lang_def, cli.force);
            let filename = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if skip {
                skipped_count += 1;
                println!("  {} {} {}", "[SKIPPED]".yellow(), filename, format!("({})", reason).dimmed());
            } else {
                queue.push(file);
                println!("  {} {}", "[QUEUED]".cyan(), filename);
            }
        }

        println!(
            "\n{} Total files: {} ({} to translate, {} already done)\n",
            "Summary:".bold(),
            all_files.len().to_string().bold(),
            queue.len().to_string().green(),
            skipped_count.to_string().yellow()
        );

        if queue.is_empty() {
            println!("{}", "All files already have the selected target language. Nothing to do!".green());
            return Ok(());
        }

        let total_in_queue = queue.len();
        let in_place_mode = !cli.new_file;

        for (idx, file) in queue.iter().enumerate() {
            println!(
                "{}",
                format!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").dimmed()
            );
            println!(
                "{} [{}/{}] {}",
                "Processing:".bold().cyan(),
                idx + 1,
                total_in_queue,
                file.file_name().unwrap().to_str().unwrap().bold()
            );

            if let Err(e) = process_single_file(file, &translator, lang_def, &cli, in_place_mode) {
                eprintln!("  {} Failed to process {}: {}", "[ERROR]".red().bold(), file.display(), e);
            }
        }

        println!(
            "{}",
            format!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").dimmed()
        );
        println!("\n{}", "Batch processing completed successfully!".bold().green());
        return Ok(());
    }

    // ==========================================
    // FLOW 2: SINGLE FILE OR INTERACTIVE WIZARD
    // ==========================================
    let input_path: PathBuf = match cli.input_path {
        Some(ref p) => p.clone(),
        None => {
            println!("\n{}\n", "Interactive Assistant:".bold().cyan());
            let local_files = find_media_files_in_dir(Path::new("."), false);

            if !local_files.is_empty() {
                let mut options: Vec<String> = local_files
                    .iter()
                    .map(|f| f.file_name().unwrap().to_str().unwrap().to_string())
                    .collect();
                options.push("Enter path manually...".to_string());

                let selection = Select::with_theme(&theme)
                    .with_prompt("Select file to translate:")
                    .default(0)
                    .items(&options)
                    .interact()?;

                if selection < local_files.len() {
                    local_files[selection].clone()
                } else {
                    let path_str: String = Input::with_theme(&theme)
                        .with_prompt("File path (video or .srt subtitle):")
                        .interact_text()?;
                    PathBuf::from(path_str.trim())
                }
            } else {
                let path_str: String = Input::with_theme(&theme)
                    .with_prompt("File path (video or .srt subtitle):")
                    .interact_text()?;
                PathBuf::from(path_str.trim())
            }
        }
    };

    if !input_path.exists() {
        eprintln!("\n{} File not found: {}", "Error:".red().bold(), input_path.display());
        std::process::exit(1);
    }

    let (skip, reason) = should_skip_file(&input_path, lang_def, cli.force);
    if skip {
        println!(
            "\n{} {} {}",
            "[SKIPPED]".yellow().bold(),
            input_path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            format!("({})", reason).dimmed()
        );
        println!("{}", "Use --force if you want to overwrite or re-translate this file.".cyan());
        return Ok(());
    }

    let in_place_mode = !cli.new_file;
    process_single_file(&input_path, &translator, lang_def, &cli, in_place_mode)?;

    println!("\n{}", "All done successfully!".bold().green());
    Ok(())
}
