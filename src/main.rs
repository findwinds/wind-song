use anyhow::Context;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::Value;
use std::{fs};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Deserialize)]
struct ApiResponse {
    code: i32,
    message: String,
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
    println!("欢迎使用文件管理工具。输入 'help' 查看可用命令。");

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
                println!("  download <BID> - 下载文件");
                println!("  exit - 退出程序");
            }
            "exit" => {
                println!("退出程序。");
                break;
            }
            cmd if cmd.starts_with("download ") => {
                let bid = cmd.split_whitespace().nth(1).unwrap_or("");
                if bid.is_empty() {
                    println!("请提供有效的 BID");
                    continue;
                }
                match download_on_bid(&bid).await {
                    Ok(()) => {
                        println!("下载完成");
                    }
                    Err(err) => println!("下载文件时出错: {}", err)
                }
            }
            _ => {
                println!("未知命令。输入 'help' 查看可用命令。");
            }
        }
        print!("> ");
    }
    Ok(())
}

async fn download_on_bid(bid: &str) -> Result<()> {
    let m4s_file_name = download(&bid).await?;
    let input_output_path = build_path(m4s_file_name, bid.parse()?)?;
    let input_path = input_output_path.0;
    let output_path = input_output_path.1;
    wind_song::convert_input_to_mp3(&*input_path, &*output_path);
    delete_download_file(input_path)?;
    println!("保存的文件路径: {}", output_path);
    Ok(())
}

fn delete_download_file(input_path: String) -> Result<()> {
    //删除缓存文件
    fs::remove_file(input_path).context("删除缓存文件错误")?;
    Ok(())
}

async fn download(bid: &str) -> Result<String> {
    // 第一步：获取视频信息
    let url = format!("https://api.bilibili.com/x/web-interface/view?bvid={}", bid);
    let response = reqwest::get(&url).await.context("请求视频信息失败")?;
    if !response.status().is_success() {
        return Err(anyhow!("请求视频信息失败: {}", response.status()));
    }
    let api_response: ApiResponse = response.json().await.context("解析视频信息失败")?;
    println!("api_response: {:#?}", api_response);
    let video_data = api_response.data;
    println!("video_data: {:#?}", video_data);
    // 第二步：获取音频流 URL
    let audio_url = format!(
        "https://api.bilibili.com/x/player/playurl?fnval=16&bvid={}&cid={}",
        video_data.bvid, video_data.cid
    );
    println!("audio_url: {:#?}", audio_url);
    let response = reqwest::get(&audio_url)
        .await
        .context("请求音频流 URL 失败")?;
    if !response.status().is_success() {
        return Err(anyhow!("请求音频流 URL 失败: {}", response.status()));
    }
    let json: Value = response
        .json()
        .await
        .context("解析音频流 URL 失败")?;
    let audio_array = json["data"]["dash"]["audio"]
        .as_array()
        .context("无法获取音频流数组")?;
    println!("音频流数组长度: {}", audio_array.len());

    let last_audio = audio_array.last().context("音频流数组为空")?;
    let final_audio_url = last_audio["baseUrl"]
        .as_str()
        .context("无法获取音频流 URL")?
        .to_string();

    println!("final_audio_url: {:#?}", final_audio_url);

    // 第三步：下载音频文件
    let response = reqwest::get(&final_audio_url)
        .await
        .context("下载音频文件失败")?;
    if !response.status().is_success() {
        return Err(anyhow!("下载音频文件失败: {:#?}",  response.status()))
    }
    let content = response.bytes().await.context("读取音频文件内容失败")?;

    // 创建下载目录
    let download_dir = PathBuf::from("downloads");
    fs::create_dir_all(&download_dir).context("创建下载目录失败")?;

    // 保存文件
    let file_name = format!("{}.m4s", video_data.bvid);
    let file_path = download_dir.join(&file_name);
    let mut file = File::create(file_path.clone()).context("创建文件失败")?;
    file.write_all(&content).context("写入文件失败")?;
    println!("音频文件已保存为: {}", file_path.display());
    Ok(file_name)
}

fn build_path(m4s_file_name: String, bid: String) -> Result<(String, String)> {
    let download_dir = PathBuf::from("downloads"); // 相对路径
    let output_dir = PathBuf::from("output"); // 相对路径

    // 创建下载目录
    fs::create_dir_all(&download_dir).context("创建目录失败")?;
    // 创建输出目录
    fs::create_dir_all(&output_dir).context("创建目录失败")?;

    // 构建输入路径和输出路径
    let input_path = download_dir.join(&m4s_file_name);
    let output_path = output_dir.join(format!("{}.mp3", bid));

    // 将路径转换为字符串
    let input_path = input_path.to_string_lossy().into_owned();
    let output_path = output_path.to_string_lossy().into_owned();

    Ok((input_path, output_path))
}
