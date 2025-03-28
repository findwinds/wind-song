use anyhow::{Result};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

mod audio;
mod bilibili;
mod commands;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    println!("欢迎使用 Bilibili 音频下载工具。输入 'help' 查看可用命令。");
    let mut debug_mode = false;

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
            "help" => commands::show_help(),
            "list" => commands::list_downloaded_audios().await?,
            "exit" => {
                println!("退出程序。");
                break;
            }
            "debug" => {
                debug_mode = !debug_mode;
                println!("调试模式已{}", if debug_mode { "开启" } else { "关闭" });
            }
            cmd if cmd.starts_with("play ") => commands::play_audio_by_index(cmd).await?,
            cmd if cmd.starts_with("download ") => {
                commands::download_audio_from_bvid(cmd, debug_mode).await?
            }
            _ => println!("未知命令。输入 'help' 查看可用命令。"),
        }
    }
    Ok(())
}
