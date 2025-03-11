# [wind-song]
一个用于处理视频下载和转换的 Rust 应用程序。

## 目录

- [功能](#功能)
- [安装](#安装)
- [使用方法](#使用方法)

## 功能
- 根据输入的 Bid 下载 Bilibili 视频的音频流。
- 将下载的音频流转换为 MP3 格式。

## 安装

### 依赖环境
- Rust
- ffmpeg

## 使用方法

```
请输入视频编号：
BV1Fu411s78a
```
效果：

```
请输入视频编号：
BV1Fu411s78a
api_response: ApiResponse {
    code: 0,
    message: "0",
    data: VideoData {
        bvid: "BV1Fu411s78a",
        title: "【HiRes】周杰伦-《晴天》｜好想再问一遍，你会等待，还是离开、",
        cid: 1141402681,
    },
}
video_data: VideoData {
    bvid: "BV1Fu411s78a",
    title: "【HiRes】周杰伦-《晴天》｜好想再问一遍，你会等待，还是离开、",
    cid: 1141402681,
}
audio_url: "https://api.bilibili.com/x/player/playurl?fnval=16&bvid=BV1Fu411s78a&cid=1141402681"
final_audio_url: "https://xy183x94x183x138xy.mcdn.bilivideo.cn:8082/v1/resource/1141402681-1-30280.m4s?agrr=1&build=0&buvid=&bvc=vod&bw=25255&deadline=1741683866&e=ig8euxZM2rNcNbdlhoNvNC8BqJIzNbfqXBvEqxTEto8BTrNvN0GvT90W5JZMkX_YN0MvXg8gNEV4NC8xNEV4N03eN0B5tZlqNxTEto8BTrNvNeZVuJ10Kj_g2UB02J0mN0B5tZlqNCNEto8BTrNvNC7MTX502C8f2jmMQJ6mqF2fka1mqx6gqj0eN0B599M%3D&f=u_0_0&gen=playurlv2&logo=A0020000&mcdnid=50018816&mid=0&nbs=1&nettype=0&og=cos&oi=3657536509&orderid=0%2C3&os=mcdn&platform=pc&sign=4533ef&traceid=trXKKWXbdMKLlv_0_e_N&uipk=5&uparams=e%2Cuipk%2Cnbs%2Cdeadline%2Cgen%2Cos%2Coi%2Ctrid%2Cmid%2Cplatform%2Cog&upsig=5f07eda8b6a9921351091248c4767c54"
音频文件已保存为: downloads/BV1Fu411s78a.m4s
+-----------+
|    in     |default--[48000Hz fltp:stereo]--auto_aresample_0:default
| (abuffer) |
+-----------+

                                                      +---------------+
Parsed_anull_0:default--[48000Hz s32p:stereo]--default|      out      |
                                                      | (abuffersink) |
                                                      +---------------+

                                                        +----------------+
auto_aresample_0:default--[48000Hz s32p:stereo]--default| Parsed_anull_0 |default--[48000Hz s32p:stereo]--out:default
                                                        |    (anull)     |
                                                        +----------------+

                                          +------------------+
in:default--[48000Hz fltp:stereo]--default| auto_aresample_0 |default--[48000Hz s32p:stereo]--Parsed_anull_0:default
                                          |   (aresample)    |
                                          +------------------+
```

保存后的mp3文件在output文件夹下