use env_logger::Env;
use reqwest::blocking::Client;
use reqwest::header::{REFERER, USER_AGENT};
use serde_json::{from_str, Value};
use std::{env, fs, io::Write, process::exit};

#[derive(Debug)]
struct Bilibili {
    cli: Client,
    cid_url: String,
    cid: String,
    play_url: String,
    url: String,
    real_url: String,
    bvid: String,
    title: String,
}

impl Bilibili {
    fn new(url: String, play_url: String, mut cid_base_url: String) -> Self {
        let mut bvid = "".to_string();
        if let Some(bv_index) = url.find("BV") {
            if url.len() >= bv_index + 12 {
                bvid.push_str(&url[bv_index..bv_index + 12]);
                cid_base_url.push_str(&url[bv_index..bv_index + 12]);
            }
        }
        Self {
            cli: Client::new(),
            cid: "".to_string(),
            cid_url: cid_base_url,
            play_url: play_url,
            url: url,
            real_url: "".to_string(),
            bvid: bvid,
            title: "".to_string(),
        }
    }

    fn get_cid(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let r = self.cli.get(&self.cid_url).send()?;
        let json: Value = from_str(&r.text()?).unwrap();
        let title = json["data"]["title"].as_str().unwrap();
        let cid = json["data"]["cid"].as_u64().unwrap().to_string();
        self.title = title.to_string();
        self.cid = cid;
        Ok(())
    }

    fn get_real_url(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.play_url.push_str("?bvid=");
        self.play_url.push_str(&self.bvid);
        self.play_url.push_str("&cid=");
        self.play_url.push_str(&self.cid);
        self.play_url.push_str("&qn=120");
        let r = self.cli.get(&self.play_url).send()?;
        let json: Value = from_str(&r.text()?).unwrap();
        self.real_url = json["data"]["durl"][0]["url"].as_str().unwrap().to_string();
        Ok(())
    }

    fn download(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.get_cid();
        let _ = self.get_real_url();
        info!("正在下载 {}，请稍后。。。", self.title);
        let r = self.cli.get(&self.real_url)
            .header(REFERER, "https://www.bilibili.com")
            .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:78.0) Gecko/20100101 Firefox/78.0")
            .send()?
            .bytes()?;
        let mut fname = self.title.clone();
        fname.push_str(".flv");
        let mut file = fs::File::create(&fname)?;
        file.write_all(&r)?;

        info!("{} 下载完成", fname);

        Ok(())
    }
}

#[macro_use]
extern crate log;

fn main() {
    logger_init();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        error!("argument is error");
        exit(-1);
    }

    let mut bili = Bilibili::new(
        args[1].clone(),
        String::from("https://api.bilibili.com/x/player/playurl"),
        String::from("https://api.bilibili.com/x/web-interface/view?bvid="),
    );
    let _ = bili.download();
}

fn logger_init() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
}
