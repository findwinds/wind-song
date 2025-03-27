use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

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

#[tokio::main]
async fn main() -> Result<()> {
    println!("欢迎使用 Bilibili 音频下载工具。输入 'help' 查看可用命令。");
    let mut debug_mode = false; // 添加一个布尔变量来控制调试模式

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut buffer = String::new();

    loop {
        buffer.clear();
        print!("> ");
        io::stdout().flush().await?;
        reader.read_line(&mut buffer).await?;
        let command = buffer.trim();
        match command {
            "help" => {
                println!("可用命令：");
                println!("  download <BID> - 下载音频");
                println!("  debug - 切换调试模式");
                println!("  exit - 退出程序");
            }
            "exit" => {
                println!("退出程序。");
                break;
            }
            "debug" => {
                debug_mode = !debug_mode; // 切换调试模式
                println!("调试模式已{}", if debug_mode { "开启" } else { "关闭" });
            }
            cmd if cmd.starts_with("download ") => {
                let bvid = cmd.split_whitespace().nth(1).unwrap_or("");
                if bvid.is_empty() {
                    println!("请提供有效的 BVID");
                    continue;
                }
                match download_audio_from_bvid(&bvid, debug_mode).await {
                    Ok(output_path) => {
                        println!("音频已保存到: {}", output_path);
                    }
                    Err(err) => println!("下载音频时出错: {}", err),
                }
            }
            _ => {
                println!("未知命令。输入 'help' 查看可用命令。");
            }
        }
    }
    Ok(())
}

async fn download_audio_from_bvid(bvid: &str, debug_mode: bool) -> Result<String> {
    let video_info = fetch_video_info(bvid, debug_mode).await?;
    let (audio_url, playurl) =
        fetch_audio_url(&video_info.bvid, video_info.cid, 192, debug_mode).await?;
    let m4s_file_name =
        download_audio_file(&audio_url, &playurl, &video_info.bvid, debug_mode).await?;
    let output_path = convert_audio_to_mp3(&m4s_file_name, &video_info.bvid)?;
    delete_temporary_file(&m4s_file_name)?;
    Ok(output_path)
}

fn delete_temporary_file(file_path: &str) -> Result<()> {
    fs::remove_file(file_path).context("删除临时文件失败")?;
    Ok(())
}

async fn fetch_video_info(bvid: &str, debug_mode: bool) -> Result<VideoData> {
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

async fn fetch_audio_url(
    bvid: &str,
    cid: i64,
    quality: i32,
    debug_mode: bool,
) -> Result<(String, String)> {
    let headers = create_request_headers();
    let url = format!(
        "https://api.bilibili.com/x/player/wbi/playurl?bvid={}&cid={}&qn={}&fnver=0&fnval=4048&fourk=1",
        bvid, cid, quality
    );
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

fn create_request_headers() -> HeaderMap {
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

async fn download_audio_file(
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
    let mut file = File::create(&file_path).context("创建文件失败")?;
    file.write_all(&content).context("写入文件失败")?;
    if debug_mode {
        println!("音频文件已保存为: {}", file_path.display());
    }
    Ok(file_path.to_string_lossy().into_owned())
}

fn convert_audio_to_mp3(input_file: &str, bvid: &str) -> Result<String> {
    let download_dir = PathBuf::from("downloads");
    let output_dir = PathBuf::from("output");

    fs::create_dir_all(&download_dir).context("创建下载目录失败")?;
    fs::create_dir_all(&output_dir).context("创建输出目录失败")?;

    let input_path = input_file;
    let output_path = output_dir.join(format!("{}.mp3", bvid));

    wind_song::convert_input_to_mp3(input_path, &output_path.to_string_lossy());
    Ok(output_path.to_string_lossy().into_owned())
}
