use anyhow::{Context, Result};
use std::fs;
use std::io::Write; // 引入 Write 特性
use std::path::Path;

pub fn save_audio_info(output_path: &str, title: &str, bvid: &str) -> Result<()> {
    let file_path = Path::new("downloads/info.txt");

    if file_path.exists() {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(file_path)
            .context("打开信息文件失败")?;
        writeln!(file, "{}|{}|{}", bvid, title, output_path).context("追加记录到信息文件失败")?;
    } else {
        let mut file = fs::File::create(file_path).context("创建信息文件失败")?;
        writeln!(file, "{}|{}|{}", bvid, title, output_path).context("写入信息文件失败")?;
    }

    Ok(())
}

pub fn delete_temporary_file(file_path: &str) -> Result<()> {
    fs::remove_file(file_path).context("删除临时文件失败")?;
    Ok(())
}
