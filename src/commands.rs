use crate::{audio, bilibili};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn show_help() {
    println!("可用命令：");
    println!("  download <BID> - 下载音频");
    println!("  debug - 切换调试模式");
    println!("  list - 列出所有已下载的音频");
    println!("  play <序号> - 播放指定序号的音频");
    println!("  exit - 退出程序");
}

pub async fn list_downloaded_audios() -> Result<()> {
    let file_path = Path::new("downloads/info.txt");
    if !file_path.exists() {
        println!("没有已下载的音频信息。");
        return Ok(());
    }

    let contents = fs::read_to_string(file_path).context("读取信息文件失败")?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut index = 1;
    for line in lines {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() == 3 {
            println!("{}. {}", index, parts[1]);
            index += 1;
        }
    }
    Ok(())
}

pub async fn play_audio_by_index(cmd: &str) -> Result<()> {
    let index_str = cmd.split_whitespace().nth(1).unwrap_or("");
    match index_str.parse::<usize>() {
        Ok(index) => match audio::play_audio_by_index(index).await {
            Ok(_) => println!("播放完成"),
            Err(err) => println!("播放音频时出错: {}", err),
        },
        Err(_) => println!("请输入有效的序号"),
    }
    Ok(())
}

pub async fn download_audio_from_bvid(cmd: &str, debug_mode: bool) -> Result<()> {
    let bvid = cmd.split_whitespace().nth(1).unwrap_or("");
    if bvid.is_empty() {
        println!("请提供有效的 BVID");
        return Ok(());
    }
    match bilibili::download_audio_from_bvid(bvid, debug_mode).await {
        Ok(output_path) => {
            println!("音频已保存到: {}", output_path);
        }
        Err(err) => println!("下载音频时出错: {}", err),
    }
    Ok(())
}
