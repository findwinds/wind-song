use anyhow::{anyhow, Context, Result};
use rodio::{OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tokio::fs;

pub async fn play_audio_by_index(index: usize) -> Result<()> {
    let file_path = Path::new("downloads/info.txt");
    if !file_path.exists() {
        return Err(anyhow!("没有已下载的音频信息"));
    }

    let contents = fs::read_to_string(file_path).await.context("读取信息文件失败")?;
    let lines: Vec<&str> = contents.lines().collect();
    if index == 0 || index > lines.len() {
        return Err(anyhow!("无效的序号"));
    }

    let line = lines[index - 1];
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() != 3 {
        return Err(anyhow!("信息文件格式错误"));
    }

    let audio_path = parts[2];
    play_audio_file(audio_path).await?;
    Ok(())
}

pub async fn play_audio_file(audio_path: &str) -> Result<()> {
    let file = File::open(audio_path).context("打开音频文件失败")?;
    let reader = BufReader::new(file);

    let (_stream, stream_handle) = OutputStream::try_default().context("创建音频输出流失败")?;
    let sink = Sink::try_new(&stream_handle).context("创建音频播放器失败")?;

    sink.append(rodio::Decoder::new(reader).context("解析音频文件失败")?);

    while !sink.empty() {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    Ok(())
}
