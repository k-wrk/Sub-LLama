use anyhow::{bail, Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub struct TrackInfo {
    pub index: usize,
    pub codec: String,
    pub language: Option<String>,
    pub title: Option<String>,
}

pub fn list_tracks(video_path: &Path) -> Result<Vec<TrackInfo>> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "s",
            "-show_entries",
            "stream=index,codec_name:stream_tags=language,title",
            "-of",
            "json",
        ])
        .arg(video_path)
        .output()
        .context("Failed to run ffprobe. Please verify that ffmpeg/ffprobe is installed and available in PATH.")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("ffprobe error: {}", err);
    }

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let mut tracks = Vec::new();

    if let Some(streams) = parsed.get("streams").and_then(|s| s.as_array()) {
        for (i, stream) in streams.iter().enumerate() {
            let codec = stream
                .get("codec_name")
                .and_then(|c| c.as_str())
                .unwrap_or("unknown")
                .to_string();
            let tags = stream.get("tags");
            let language = tags
                .and_then(|t| t.get("language"))
                .and_then(|l| l.as_str())
                .map(|s| s.to_string());
            let title = tags
                .and_then(|t| t.get("title"))
                .and_then(|l| l.as_str())
                .map(|s| s.to_string());

            tracks.push(TrackInfo {
                index: i,
                codec,
                language,
                title,
            });
        }
    }

    Ok(tracks)
}

pub fn has_language_track(video_path: &Path, match_codes: &[&str]) -> bool {
    if let Ok(tracks) = list_tracks(video_path) {
        for t in tracks {
            if let Some(ref lang) = t.language {
                let lang_lower = lang.trim().to_lowercase();
                if match_codes.iter().any(|&c| c == lang_lower) {
                    return true;
                }
            }
            if let Some(ref title) = t.title {
                let title_lower = title.trim().to_lowercase();
                if match_codes.iter().any(|&c| title_lower.contains(c)) {
                    return true;
                }
            }
        }
    }
    false
}

/// Extrai a legenda diretamente para uma String na memória (sem tocar no disco).
/// Se falhar, usa arquivo temporário em /tmp como fallback.
pub fn extract_subtitle_to_memory(video_path: &Path, track_index: usize) -> Result<String> {
    // Tentativa 1: Pipe direto stdout
    let pipe_output = Command::new("ffmpeg")
        .args([
            "-y",
            "-v",
            "error",
            "-i",
        ])
        .arg(video_path)
        .args([
            "-map",
            &format!("0:s:{}", track_index),
            "-f",
            "srt",
            "-",
        ])
        .stdout(Stdio::piped())
        .output();

    if let Ok(out) = pipe_output {
        if out.status.success() && !out.stdout.is_empty() {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            return Ok(text);
        }
    }

    // Fallback: use temporary file in /tmp
    let tmp_path = std::env::temp_dir().join(format!("sub_llama_extract_{}.srt", std::process::id()));
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-v",
            "error",
            "-i",
        ])
        .arg(video_path)
        .args([
            "-map",
            &format!("0:s:{}", track_index),
        ])
        .arg(&tmp_path)
        .status()
        .context("Failed to run ffmpeg to extract subtitle fallback.")?;

    if !status.success() {
        let _ = fs::remove_file(&tmp_path);
        bail!("ffmpeg failed to extract subtitle track {}", track_index);
    }

    let text = fs::read_to_string(&tmp_path)
        .context("Failed to read temporary extracted subtitle")?;
    let _ = fs::remove_file(&tmp_path);

    Ok(text)
}

/// Embuti a legenda no arquivo de vídeo original.
/// Tenta pipe stdin primeiro; se falhar, usa fallback temporário em /tmp.
/// Ao final, atualiza de forma atômica o vídeo original sem deixar lixo no disco.
pub fn embed_subtitle_in_place(
    video_path: &Path,
    srt_content: &str,
    language_code: &str,
    track_title: &str,
) -> Result<()> {
    let existing_tracks = list_tracks(video_path).unwrap_or_default();
    
    // Filtra trilhas pré-existentes que sejam do mesmo idioma para não duplicar no container
    let mut keep_sub_indices = Vec::new();
    for (i, t) in existing_tracks.iter().enumerate() {
        let is_same_lang = t.language.as_ref().map(|l| l.eq_ignore_ascii_case(language_code) || (language_code == "pt" && ["por", "pt-br", "pt_br", "pob"].contains(&l.to_lowercase().as_str()))).unwrap_or(false);
        let is_same_title = t.title.as_ref().map(|title| title.eq_ignore_ascii_case(track_title)).unwrap_or(false);
        if !is_same_lang && !is_same_title {
            keep_sub_indices.push(i);
        }
    }
    let new_sub_idx = keep_sub_indices.len();

    let ext = video_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mkv")
        .to_lowercase();
    let s_codec = if ext == "mp4" { "mov_text" } else { "copy" };

    let temp_video_output = video_path.with_extension(format!("tmp_embed_{}.{}", std::process::id(), ext));

    // Monta argumentos de mapeamento
    let mut map_args = vec!["-map".to_string(), "0:v:0".to_string(), "-map".to_string(), "0:a?".to_string()];
    for &idx in &keep_sub_indices {
        map_args.push("-map".to_string());
        map_args.push(format!("0:s:{}", idx));
    }
    map_args.push("-map".to_string());
    map_args.push("1:s".to_string());

    // Tentativa 1: Injetar legenda via Pipe stdin
    let pipe_result = (|| -> Result<()> {
        let mut child = Command::new("ffmpeg")
            .args(["-y", "-v", "error", "-i"])
            .arg(video_path)
            .args(["-f", "srt", "-i", "pipe:0"])
            .args(&map_args)
            .args([
                "-c",
                "copy",
                "-c:s",
                s_codec,
                &format!("-metadata:s:s:{}", new_sub_idx),
                &format!("language={}", language_code),
                &format!("-metadata:s:s:{}", new_sub_idx),
                &format!("title={}", track_title),
                &format!("-disposition:s:{}", new_sub_idx),
                "default",
            ])
            .args(
                (0..new_sub_idx)
                    .flat_map(|i| vec![format!("-disposition:s:{}", i), "0".to_string()])
                    .collect::<Vec<_>>(),
            )
            .arg(&temp_video_output)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(srt_content.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            bail!("Stdin pipe failed: {}", err);
        }
        Ok(())
    })();

    if pipe_result.is_ok() {
        fs::rename(&temp_video_output, video_path)?;
        return Ok(());
    }

    // Fallback: If pipe fails, cleanup partial temp output if present
    if temp_video_output.exists() {
        let _ = fs::remove_file(&temp_video_output);
    }

    // Fallback: Create temp file in /tmp
    let tmp_srt = std::env::temp_dir().join(format!("sub_llama_embed_{}.srt", std::process::id()));
    fs::write(&tmp_srt, srt_content)?;

    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-y", "-v", "error", "-i"])
        .arg(video_path)
        .args(["-i"])
        .arg(&tmp_srt)
        .args(&map_args)
        .args([
            "-c",
            "copy",
            "-c:s",
            s_codec,
            &format!("-metadata:s:s:{}", new_sub_idx),
            &format!("language={}", language_code),
            &format!("-metadata:s:s:{}", new_sub_idx),
            &format!("title={}", track_title),
            &format!("-disposition:s:{}", new_sub_idx),
            "default",
        ]);

    for i in 0..new_sub_idx {
        cmd.args([&format!("-disposition:s:{}", i), "0"]);
    }

    cmd.arg(&temp_video_output);

    let output = cmd.output().context("Failed to run ffmpeg during embedding fallback.")?;
    let _ = fs::remove_file(&tmp_srt); // Cleanup in /tmp

    if !output.status.success() {
        if temp_video_output.exists() {
            let _ = fs::remove_file(&temp_video_output);
        }
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("ffmpeg failed to embed subtitle into video: {}", err);
    }

    // Replace original video atomically
    fs::rename(&temp_video_output, video_path)?;
    Ok(())
}
