// html_report.rs
// Modified by Berkay (2025): Enhanced path handling for UNC paths
// - Uses safe_canonicalize to avoid UNC path issues

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use url::Url;
use crate::path_utils;

pub fn generate_html_report(
    output_dir: &Path,
    total: usize,
    photos: usize,
    videos: usize,
    whatsapp: usize,
    screenshots: usize,
    unknown: usize,
    mkv: usize,
    errors: usize,
    csv_files: &[&str],
    log_files: &[&str],
    metadata_fields: &[&str],
) {
    let html_path = output_dir.join("MetaSort_Summary.html");
    let mut file = match File::create(&html_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[WARNING] Failed to create HTML report: {}", e);
            return;
        }
    };

    // Helper to get file:// URL with safe canonicalization for UNC paths
    fn file_url(path: &PathBuf) -> String {
        let abs = path_utils::safe_canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        Url::from_file_path(&abs).unwrap().to_string()
    }

    let csv_links = csv_files.iter().map(|f| {
        let path = output_dir.join("Technical Files").join("CSV Report").join(f);
        let url = file_url(&path);
        format!("<li><a href='{}' target='_blank'>{}</a></li>", url, f)
    }).collect::<String>();
    let log_links = log_files.iter().map(|f| {
        let path = output_dir.join("Technical Files").join("logs").join(f);
        let url = file_url(&path);
        format!("<li><a href='{}' target='_blank'>{}</a></li>", url, f)
    }).collect::<String>();
    let meta_links = metadata_fields.iter().map(|f| format!("<li>{}</li>", f)).collect::<String>();

    let html = format!("\
<!DOCTYPE html>\
<html lang='en'>\
<head>\
<meta charset='UTF-8'>\
<title>MetaSort Summary Report</title>\
<link href='https://fonts.googleapis.com/css2?family=Inter:wght@400;700&display=swap' rel='stylesheet'>\
<style>\
body {{ font-family: 'Inter', Arial, sans-serif; background: linear-gradient(120deg, #f9f9f9 60%, #e0e7ff 100%); color: #222; margin: 0; padding: 0; }}\
.header {{ background: linear-gradient(90deg, #2b7a78 60%, #3aafa9 100%); color: #fff; padding: 2em 0 1em 0; text-align: center; border-radius: 0 0 24px 24px; box-shadow: 0 4px 16px rgba(43,122,120,0.08); }}\
.card {{ background: #fff; margin: 2em auto; border-radius: 18px; box-shadow: 0 4px 24px rgba(43,122,120,0.10); width: 90%; max-width: 800px; padding: 2em 2em 1em 2em; }}\
h1 {{ font-size: 2.2em; margin: 0 0 0.5em 0; letter-spacing: 1px; }}\
.section-title {{ font-size: 1.3em; margin-top: 2em; color: #2b7a78; display: flex; align-items: center; gap: 0.5em; }}\
table {{ border-collapse: separate; border-spacing: 0; width: 100%; margin: 1.5em 0; background: #f8fafc; border-radius: 12px; overflow: hidden; box-shadow: 0 2px 8px rgba(43,122,120,0.04); }}\
th, td {{ border: none; padding: 0.8em 1em; text-align: left; }}\
th {{ background: #def2f1; color: #222; font-weight: 700; }}\
tr:nth-child(even) {{ background: #f1f5f9; }}\
tr:hover {{ background: #e0e7ff; transition: background 0.2s; }}\
.badge {{ display: inline-block; background: #3aafa9; color: #fff; border-radius: 12px; padding: 0.2em 0.8em; font-size: 1em; margin-left: 0.5em; font-weight: 700; }}\
.emoji {{ font-size: 1.3em; margin-right: 0.3em; }}\
ul {{ list-style: none; padding: 0; }}\
li {{ margin: 0.5em 0; }}\
a {{ color: #2b7a78; text-decoration: none; font-weight: 500; transition: color 0.2s; }}\
a:hover {{ color: #3aafa9; text-decoration: underline; }}\
.footer {{ text-align: center; color: #888; margin: 3em 0 1em 0; font-size: 1em; }}\
.note {{ color: #2b7a78; background: #e0e7ff; border-radius: 8px; padding: 0.7em 1em; margin: 1em 0 2em 0; text-align: center; font-size: 1.1em; }}\
</style>\
</head>\
<body>\
<div class='header'>\
  <img src='assets/logo.png' alt='MetaSort Logo' style='width:120px;display:block;margin:0 auto 1em auto;'>\
  <h1>📊 MetaSort <span style='font-size:0.7em;font-weight:400;'>Summary Report</span></h1>\
  <div style='font-size:1.2em;'>Your Google Photos Takeout, beautifully organized! ✨</div>\
</div>\
<div class='card'>\
  <div class='note'>Click a file to open it in your default app or reveal it in Finder.</div>\
  <table>\
    <tr><th class='emoji'>📦</th><th>Total Files Processed</th><td><span class='badge'>{}</span></td></tr>\
    <tr><th class='emoji'>🖼️</th><th>Photos</th><td><span class='badge'>{}</span></td></tr>\
    <tr><th class='emoji'>🎬</th><th>Videos</th><td><span class='badge'>{}</span></td></tr>\
    <tr><th class='emoji'>💬</th><th>WhatsApp Images</th><td><span class='badge'>{}</span></td></tr>\
    <tr><th class='emoji'>📱</th><th>Screenshots</th><td><span class='badge'>{}</span></td></tr>\
    <tr><th class='emoji'>❓</th><th>Unknown Time</th><td><span class='badge'>{}</span></td></tr>\
    <tr><th class='emoji'>🎞️</th><th>MKV Files</th><td><span class='badge'>{}</span></td></tr>\
    <tr><th class='emoji'>⚠️</th><th>Errors</th><td><span class='badge'>{}</span></td></tr>\
  </table>\
  <div class='section-title'><span class='emoji'>📑</span>CSV Reports</div>\
  <ul>{}</ul>\
  <div class='section-title'><span class='emoji'>📝</span>Log Files</div>\
  <ul>{}</ul>\
  <div class='section-title'><span class='emoji'>🔍</span>Metadata Fields Extracted/Embedded</div>\
  <ul>{}</ul>\
</div>\
<div class='footer'>\
  Made with <span style='color:#e25555;'>♥</span> by Sanmith S | MetaSort · <a href='https://github.com/iamsanmith/' target='_blank'>GitHub</a>\
</div>\
</body>\
</html>\
", 
        total, photos, videos, whatsapp, screenshots, unknown, mkv, errors,
        csv_links, log_links, meta_links
    );
    let _ = writeln!(file, "{}", html);
    println!("\n[SUCCESS] HTML summary report written to: {:?}", html_path);
} 