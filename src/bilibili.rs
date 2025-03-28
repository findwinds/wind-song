use crate::utils::{delete_temporary_file, save_audio_info};
use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::BufReader as AsyncBufReader;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: VideoData,
}

#[derive(Debug, Deserialize)]
pub struct VideoData {
    pub bvid: String,
    pub title: String,
    pub cid: i64,
}

pub async fn download_audio_from_bvid(bvid: &str, debug_mode: bool) -> Result<String> {
    let video_info = fetch_video_info(bvid, debug_mode).await?;

    if is_already_downloaded(&video_info.bvid).await? {
        if debug_mode {
            println!("视频 {} 已经下载过，跳过处理", video_info.bvid);
        }
        return Err(anyhow::anyhow!("视频 {} 已经下载过", video_info.bvid));
    }

    let (audio_url, playurl) =
        fetch_audio_url(&video_info.bvid, video_info.cid, 192, debug_mode).await?;
    let m4s_file_name =
        download_audio_file(&audio_url, &playurl, &video_info.bvid, debug_mode).await?;
    let output_path = convert_audio_to_mp3(&m4s_file_name, &video_info.bvid, debug_mode)?;
    delete_temporary_file(&m4s_file_name)?;
    save_audio_info(&output_path, &video_info.title, bvid)?;
    Ok(output_path)
}

pub async fn is_already_downloaded(bvid: &str) -> Result<bool> {
    let file_path = Path::new("downloads/info.txt");

    if !file_path.exists() {
        return Ok(false);
    }

    let file = File::open(&file_path).await.context("打开信息文件失败")?;
    let reader = AsyncBufReader::new(file);

    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await.context("读取信息文件行失败")? {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 3 && parts[0] == bvid {
            return Ok(true);
        }
    }

    Ok(false)
}

pub async fn fetch_video_info(bvid: &str, debug_mode: bool) -> Result<VideoData> {
    let url = format!(
        "https://api.bilibili.com/x/web-interface/view?bvid={}",
        bvid
    );
    let response = reqwest::get(&url).await.context("请求视频信息失败")?;
    if !response.status().is_success() {
        return Err(anyhow!("请求视频信息失败: {}", response.status()));
    }
    let api_response: ApiResponse = response.json().await.context("解析视频信息失败")?;
    if debug_mode {
        println!("获取到的视频信息: {:#?}", api_response);
    }
    Ok(api_response.data)
}

pub async fn fetch_audio_url(
    bvid: &str,
    cid: i64,
    quality: i32,
    debug_mode: bool,
) -> Result<(String, String)> {
    let headers = create_request_headers();
    let url = format!("https://api.bilibili.com/x/player/wbi/playurl?bvid={}&cid={}&qn={}&fnver=0&fnval=4048&fourk=1", bvid, cid, quality);
    if debug_mode {
        println!("请求音频 URL: {:#?}", url);
    }

    let client = Client::new();
    let response = client
        .get(&url)
        .headers(headers)
        .send()
        .await
        .context("请求音频流 URL 失败")?;
    if !response.status().is_success() {
        return Err(anyhow!("请求音频流 URL 失败: {}", response.status()));
    }
    let json: Value = response.json().await.context("解析音频流 URL 失败")?;
    if debug_mode {
        println!("解析到的 JSON 数据: {:#?}", json);
    }

    let audio_array = json["data"]["dash"]["audio"]
        .as_array()
        .context("无法获取音频流数组")?;
    if debug_mode {
        println!("音频流数组长度: {}", audio_array.len());
    }
    let last_audio = audio_array.first().context("音频流数组为空")?;
    let audio_url = last_audio["baseUrl"]
        .as_str()
        .context("无法获取音频流 URL")?
        .to_string();
    Ok((audio_url, url))
}

pub fn create_request_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "User-Agent",
        HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36"),
    );
    headers.insert(
        "Origin",
        HeaderValue::from_static("https://www.bilibili.com"),
    );
    headers.insert(
        "Referer",
        HeaderValue::from_static("https://www.bilibili.com"),
    );
    headers
}

pub async fn download_audio_file(
    audio_url: &str,
    playurl: &str,
    file_name: &str,
    debug_mode: bool,
) -> Result<String> {
    let mut headers = create_request_headers();
    headers.insert("Referer", HeaderValue::from_str(playurl)?);
    let client = Client::new();
    let response = client
        .get(audio_url)
        .headers(headers)
        .send()
        .await
        .context("请求音频流 URL 失败")?;
    if !response.status().is_success() {
        return Err(anyhow!("下载音频文件失败: {:#?}", response.status()));
    }
    let content = response.bytes().await.context("读取音频文件内容失败")?;
    let download_dir = PathBuf::from("downloads");
    fs::create_dir_all(&download_dir).context("创建下载目录失败")?;
    let file_path = download_dir.join(file_name);
    let mut file = File::create(&file_path).await.context("创建文件失败")?;
    file.write_all(&content).await.context("写入文件失败")?;
    if debug_mode {
        println!("音频文件已保存为: {}", file_path.display());
    }
    Ok(file_path.to_string_lossy().into_owned())
}

pub fn convert_audio_to_mp3(input_file: &str, bvid: &str, debug_mode: bool) -> Result<String> {
    let download_dir = PathBuf::from("downloads");
    let output_dir = PathBuf::from("output");

    fs::create_dir_all(&download_dir).context("创建下载目录失败")?;
    fs::create_dir_all(&output_dir).context("创建输出目录失败")?;

    let input_path = input_file;
    let output_path = output_dir.join(format!("{}.mp3", bvid));

    wind_song::convert_input_to_mp3(input_path, &output_path.to_string_lossy(), debug_mode);
    Ok(output_path.to_string_lossy().into_owned())
}
