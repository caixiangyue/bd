use std::{process::exit, env};
use std::fs;
use std::io::Write;
use reqwest::header::{REFERER, USER_AGENT};

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
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
    c_url.push_str(bvid);
    let cid: serde_json::Value;
    let res = get_cid(&c_url);
    match res {
        Ok(v)=> {
            cid = v;
        }
        Err(e) => {
            eprintln!("get bvid error: {}", e);
            exit(-1);
        }
    }
    let cid = cid.as_u64().unwrap().to_string();
    println!("{}", cid);

    g_url.push_str("?bvid=");
    g_url.push_str(bvid);
    g_url.push_str("&cid=");
    g_url.push_str(cid.as_str());
    g_url.push_str("&qn=80");

    println!("{}", g_url);
    get_real(&g_url, bvid);
}

fn get_bvid(url: &str) -> Option<&str> {
    match url.find("BV") {
        Some(bv_index) => Some(&url[bv_index..]),
        None => None
    }
}

#[tokio::main]
async fn get_cid(c_url: &String) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let resp = reqwest::get(c_url)
        .await?
        .json::<serde_json::Value>()
        .await?;
    let cid= resp["data"]["cid"].clone();
    // Box::new(cid);
    // println!("{}", cid);
    Ok(cid)
}

#[tokio::main]
async fn get_real(g_url: &String, bvid: &str) -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::get(g_url)
        .await?
        .json::<serde_json::Value>()
        .await?;

    let v = &resp["data"]["durl"][0]["backup_url"][0];
    let real_url = v.as_str().unwrap();
    println!("{}", real_url);
    let client = reqwest::Client::builder()
        .build()?;
    let resp = client.get(real_url)
            .header(REFERER, "https://www.bilibili.com")
            .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:78.0) Gecko/20100101 Firefox/78.0")
            .send()
            .await?
            .bytes()
            .await?;
    
    let mut fname = String::from(bvid);
    fname.push_str(".flv");
    let mut file = fs::File::create(fname).unwrap();
    file.write_all(&resp).unwrap();

    Ok(())
}