use anyhow::{bail, Context, Result};
use ct2rs::tokenizers::sentencepiece::Tokenizer as SpmTokenizer;
use ct2rs::{Config, Device, TranslationOptions, Translator};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use crate::languages::LanguageDefinition;

pub struct Ct2Translator {
    translator: Translator<SpmTokenizer>,
    lang_def: &'static LanguageDefinition,
}

impl Ct2Translator {
    pub fn download_files(lang_def: &'static LanguageDefinition, model_dir: &Path) -> Result<()> {
        fs::create_dir_all(model_dir)?;

        if lang_def.code == "pt" {
            // Repositório CTranslate2 INT8 oficial pré-empacotado para opus-mt-tc-big-en-pt
            let archive_path = model_dir.join("opus-mt-tc-big-en-pt-ct2.tar.gz");
            if !model_dir.join("model.bin").exists() {
                println!(
                    "\nDownloading optimized CTranslate2 INT8 model: [{}]",
                    lang_def.ct2_repo.unwrap_or(lang_def.hf_repo)
                );
                let url = "https://huggingface.co/playto-mt/opus-mt-tc-big-en-pt-ct2/resolve/main/opus-mt-tc-big-en-pt-ct2.tar.gz";
                download_with_progress(url, &archive_path)?;

                println!("  Extracting model files...");
                let tar_gz = File::open(&archive_path)?;
                let tar = flate2::read::GzDecoder::new(tar_gz);
                let mut archive = tar::Archive::new(tar);

                // Extract flattened files directly into model_dir
                for entry in archive.entries()? {
                    let mut entry = entry?;
                    let path = entry.path()?;
                    if let Some(file_name) = path.file_name() {
                        let dest = model_dir.join(file_name);
                        let mut out = File::create(&dest)?;
                        std::io::copy(&mut entry, &mut out)?;
                    }
                }
                let _ = fs::remove_file(&archive_path);
            }
        } else if let Some(ct2_repo) = lang_def.ct2_repo {
            let base_files = [
                "config.json",
                "model.bin",
                "source.spm",
                "target.spm",
            ];
            println!(
                "\nDownloading CTranslate2 model files: [{}]",
                ct2_repo
            );

            for (idx, &file_name) in base_files.iter().enumerate() {
                let file_path = model_dir.join(file_name);
                if file_path.exists() && fs::metadata(&file_path)?.len() > 0 {
                    continue;
                }
                println!("  [{}/{}] {}", idx + 1, base_files.len() + 1, file_name);
                let url = format!("https://huggingface.co/{}/resolve/main/{}", ct2_repo, file_name);
                download_with_progress(&url, &file_path)?;
            }

            // Handle vocabulary file (can be shared_vocabulary.json or vocab.json)
            let shared_vocab_path = model_dir.join("shared_vocabulary.json");
            let vocab_path = model_dir.join("vocab.json");
            if !shared_vocab_path.exists() && !vocab_path.exists() {
                let shared_url = format!("https://huggingface.co/{}/resolve/main/shared_vocabulary.json", ct2_repo);
                println!("  [{}/{}] vocabulary", base_files.len() + 1, base_files.len() + 1);
                if download_with_progress(&shared_url, &shared_vocab_path).is_err() {
                    let fallback_url = format!("https://huggingface.co/{}/resolve/main/vocab.json", ct2_repo);
                    download_with_progress(&fallback_url, &vocab_path)?;
                    // Duplicate/link as shared_vocabulary.json so CTranslate2 sentencepiece tokenizers find it consistently
                    let _ = fs::copy(&vocab_path, &shared_vocab_path);
                }
            } else if vocab_path.exists() && !shared_vocab_path.exists() {
                let _ = fs::copy(&vocab_path, &shared_vocab_path);
            }
        } else {
            bail!("No CTranslate2 repository configured for language: {}", lang_def.code);
        }

        Ok(())
    }

    pub fn load(model_dir: &Path, lang_def: &'static LanguageDefinition) -> Result<Self> {
        let tokenizer = SpmTokenizer::new(model_dir)
            .context("Failed to initialize SentencePiece tokenizer for CTranslate2")?;

        let config = Config {
            device: Device::CPU,
            ..Default::default()
        };

        let translator = Translator::with_tokenizer(model_dir, tokenizer, &config)
            .context("Failed to load CTranslate2 engine")?;

        Ok(Self {
            translator,
            lang_def,
        })
    }

    pub fn translate_batch(&self, texts: &[&str]) -> Result<Vec<String>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // If the model requires special target language token (e.g. >>por<<)
        let tagged_texts: Vec<String> = if let Some(tag) = self.lang_def.target_tag {
            texts.iter().map(|&t| format!("{} {}", tag, t.trim())).collect()
        } else {
            texts.iter().map(|&t| t.trim().to_string()).collect()
        };

        let slice_refs: Vec<&str> = tagged_texts.iter().map(|s| s.as_str()).collect();
        let options = TranslationOptions::default();

        let results = self
            .translator
            .translate_batch(&slice_refs, &options, None)
            .map_err(|e| anyhow::anyhow!("CTranslate2 error: {}", e))?;

        let translations = results.into_iter().map(|(trans, _)| trans).collect();
        Ok(translations)
    }
}

fn download_with_progress(url: &str, output_path: &Path) -> Result<()> {
    let tmp_path = output_path.with_extension("tmp_download");
    let resp = ureq::get(url).call().with_context(|| format!("Failed to download {}", url))?;
    let total_size = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  [{bar:35.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA: {eta})")
            .unwrap()
            .progress_chars("━╸ "),
    );

    let mut reader = resp.into_body().into_reader();
    let mut file = File::create(&tmp_path)?;
    let mut buffer = [0u8; 65536];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        std::io::Write::write_all(&mut file, &buffer[..bytes_read])?;
        pb.inc(bytes_read as u64);
    }
    pb.finish_and_clear();
    fs::rename(&tmp_path, output_path)?;
    Ok(())
}
