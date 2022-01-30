use std::{process::exit, env};
use std::fs;
use std::io::Write;
use reqwest::header::{REFERER, USER_AGENT};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        exit(-1);
    }
    let bv_url = &args[1];
    let mut c_url = String::from("https://api.bilibili.com/x/web-interface/view?bvid=");
    let mut g_url = String::from("https://api.bilibili.com/x/player/playurl");
    let bvid: &str;
    match get_bvid(bv_url) {
        Some(id) => bvid = id,
        None => {
                eprintln!("get bvid error");
                exit(-1);
            }
    }

    download(&mut c_url, &mut g_url, bvid);
}

fn get_bvid(url: &str) -> Option<&str> {
    match url.find("BV") {
        Some(bv_index) => Some(&url[bv_index..bv_index+12]),
        None => None
    }
}

#[tokio::main]
async fn download(c_url: &mut String, g_url: &mut String, bvid: &str) -> Result<(), Box<dyn std::error::Error>> {
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
    println!("正在下载 {}，请稍后。。。", title.as_str().unwrap());
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

    println!("{} 下载完成", fname);
    Ok(())
}