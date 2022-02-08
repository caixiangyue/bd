use std::{process::exit, env, fs, io::Write};
use reqwest::header::{REFERER, USER_AGENT};
use env_logger::Env;

#[macro_use]
extern crate log;

fn main() {
    logger_init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        error!("argument is error");
        exit(-1);
    }

    if let Some(bvid) = get_bvid(&args[1]) {
        download(bvid);
    } else {
        error!("url is invalid");
    }
}

fn logger_init() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
}

fn get_bvid(url: &str) -> Option<&str> {
    if let Some(bv_index) = url.find("BV") {
        if url.len() < bv_index+12 {
            return None
        } else {
            return Some(&url[bv_index..bv_index+12])
        }
    }
    None
}

#[tokio::main]
async fn download(bvid: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut c_url = String::from("https://api.bilibili.com/x/web-interface/view?bvid=");
    let mut g_url = String::from("https://api.bilibili.com/x/player/playurl");

    c_url.push_str(bvid);
    let r1 = reqwest::get(c_url.as_str())
        .await?
        .json::<serde_json::Value>()
        .await?;
    let title = &r1["data"]["title"];
    let v= &r1["data"]["cid"];
    let cid = v.as_u64().unwrap().to_string();

    g_url.push_str("?bvid=");
    g_url.push_str(bvid);
    g_url.push_str("&cid=");
    g_url.push_str(cid.as_str());
    g_url.push_str("&qn=80");
    
    
    let r2 = reqwest::get(g_url.as_str())
        .await?
        .json::<serde_json::Value>()
        .await?;

    let v = &r2["data"]["durl"][0]["url"];
    let real_url = v.as_str().unwrap();
    info!("正在下载 {}，请稍后。。。", title.as_str().unwrap());
    let client = reqwest::Client::builder()
        .build()?;
    let r3 = client.get(real_url)
            .header(REFERER, "https://www.bilibili.com")
            .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:78.0) Gecko/20100101 Firefox/78.0")
            .send()
            .await?
            .bytes()
            .await?;

    let mut fname = String::from(title.as_str().unwrap());
    fname.push_str(".flv");
    let mut file = fs::File::create(&fname)?;
    file.write_all(&r3)?;

    info!("{} 下载完成", fname);
    Ok(())
}
