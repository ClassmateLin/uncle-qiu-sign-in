use std::env;

/// 丘大叔签到脚本
use reqwest::{Client, header::{HeaderMap, HeaderValue}};
use anyhow::Result;
use rand::Rng;
use serde_json::json;
use rand::distributions::Slice;


struct QClient {
    client: Client,
    base_url: String,
    rand_string: String,
}

impl QClient {

    pub fn new(token: &str) -> Self {
        
        let numbers = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'];
        let numbers_slice = Slice::new(&numbers).unwrap();
        let rand_string: String = rand::thread_rng()
            .sample_iter(&numbers_slice)
            .take(8)
            .collect();

        let mut headers = HeaderMap::new();
        
        let user_agent = "Mozilla/5.0 (iPhone; CPU iPhone OS 15_4_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 MicroMessenger/8.0.29(0x18001d2c) NetType/WIFI Language/zh_CN";
        
        headers.append("User-Agent", HeaderValue::from_static(user_agent));
        headers.append("Qm-User-Token", HeaderValue::from_str(token).unwrap());
        headers.append("qm-from", HeaderValue::from_static("wechat"));

        let client = Client::builder()
            .default_headers(headers)
            .build().unwrap();
        
        let base_url = String::from("https://webapi.qmai.cn/");
        
        Self { client, base_url, rand_string  }
    }

    pub async fn sign_in(&self) -> Result<()> {
        
        let url = format!("{}/web/catering/integral/sign/signIn", self.base_url);
        let form = json!({
            "activityId": "100820000000000642",
            "mobilePhone": format!("138{}", self.rand_string),
            "userName": self.rand_string,
            "appid": format!("wx{}", self.rand_string),
        });
        let data = self.client
            .post(url)
            .form(&form)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        let code = data["code"].as_i64().unwrap_or(-1);

        match code {
            0 | 400041 => {
                print!("今日已签到, ");
            },
            _ => {println!("签到失败!")}
        }
        Ok(())
    }

    pub async fn sign_info(&self) -> Result<()> {
        let url = format!("{}/web/catering/integral/sign/detail", self.base_url);
        
        let form = json!({
            "appid": format!("wx{}", self.rand_string),
        });

        let data = self.client
            .post(url)
            .form(&form)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        if let Some(days) = data["data"]["totalDays"].as_u64() {
            println!("已连续签到{:?}天...", days);    
        }

        Ok(())
    }
}


#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    if let Ok(token) = env::var("Q_TOKEN") {
        let qclient = QClient::new(&token);
        qclient.sign_in().await?;
        qclient.sign_info().await?;
    }else{
        println!("未设置环境变量Q_TOKEN, 退出...");
    };
    Ok(())
}
