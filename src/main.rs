use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

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
async fn main() {
    let bid = parse_cil();
    let m4s_file_name = download(&bid).await.ok().unwrap();
    let input_output_path = build_path(m4s_file_name,bid);
    let input_path = input_output_path.0;
    let output_path = input_output_path.1;
    wind_song::convert_input_to_mp3(&*input_path, &*output_path);
}

fn build_path(m4s_file_name: String, bid: String) -> (String, String) {
    let download_dir = PathBuf::from("downloads"); // 相对路径
    let output_dir = PathBuf::from("output"); // 相对路径
    fs::create_dir_all(&download_dir).expect("创建目录失败");
    fs::create_dir_all(&output_dir).expect("创建目录失败");
    let input_path = download_dir.join(&m4s_file_name);
    let output_path = output_dir.join(format!("{}.mp3", bid));
    let input_path = input_path.to_string_lossy().into_owned();
    let output_path = output_path.to_string_lossy().into_owned();
    (input_path, output_path)
}

async fn download(bid: &str) -> Result<String, reqwest::Error> {
    let url = format!("https://api.bilibili.com/x/web-interface/view?bvid={}", bid);
    let response = reqwest::get(&url).await?;
    if response.status().is_success() {
        let api_response: ApiResponse = response.json().await?;
        println!("api_response: {:#?}", api_response);
        let video_data = api_response.data;
        println!("video_data: {:#?}", video_data);
        let audio_url = format!(
            "https://api.bilibili.com/x/player/playurl?fnval=16&bvid={}&cid={}",
            video_data.bvid, video_data.cid
        );
        println!("audio_url: {:#?}", audio_url);
        let response = reqwest::get(&audio_url).await?;
        let json: Value = response.json().await?;
        let final_audio_url = json["data"]["dash"]["audio"][0]["baseUrl"]
            .as_str()
            .map(ToString::to_string)
            .unwrap();
        println!("final_audio_url: {:#?}", final_audio_url);
        //我需要把组装出来的final_audio_url对应的w4s文件下载到当前目录下，用bvid+.m4s命名
        let response = reqwest::get(&final_audio_url).await?;
        let content = response.bytes().await?;
        let download_dir = PathBuf::from("downloads");
        let file_name = format!("{}.m4s", video_data.bvid);
        let file_path = download_dir.join(&file_name);
        let mut file = File::create(file_path.clone()).unwrap();
        file.write_all(&content).expect("TODO: panic message");
        println!("音频文件已保存为: {}", file_path.display());
        Ok(file_name)
    } else {
        println!("请求失败: {:#?}", response.error_for_status());
        panic!("下载视频失败")
    }
}

fn parse_cil() -> String {
    // 将返回类型改为 String
    println!("请输入视频编号：");
    let mut bid = String::new();
    std::io::stdin()
        .read_line(&mut bid)
        .expect("读取输入时发生错误");
    bid = bid.trim().to_string(); // 去掉末尾的换行符并转换为 String
    bid // 返回 String 类型
}
