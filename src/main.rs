use std::{env, process::Command, time::Duration};
use chrono::{Datelike, Local};
use thirtyfour::{error::WebDriverErrorInfo, prelude::*};
use dotenv::dotenv;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), WebDriverError> {
    dotenv().ok();

    // 1. setup
    let web_driver_type = env::var("WEB_DRIVER_TYPE")
        .expect("WEB_DRIVER_TYPE must be set.");
    assert!(web_driver_type == "chrome", "Only Chrome is supported for now.");    

    let chromedriver_path = env::var("WEB_DRIVER_PATH")
        .expect( "WEB_DRIVER_PATH must be set.");
    let chromedriver_port = env::var("WEB_DRIVER_PORT")
        .expect("WEB_DRIVER_PORT must be set.")
        .parse::<u32>()
        .expect("WEB_DRIVER_PORT must be a number.");
    let chromedriver_process = start_chromedriver(
        chromedriver_path.as_str(),
        chromedriver_port
    );

    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new(format!("http://localhost:{}", chromedriver_port), caps).await?;

    let username = env::var("SSTM_USERNAME")
        .expect("SSTM_USERNAME must be set.");
    let password = env::var("SSTM_PASSWORD")
        .expect("SSTM_PASSWORD must be set.");
    let wait_time = env::var("DEFAULT_WAIT_TIME_SECS")
        .expect("DEFAULT_WAIT_TIME_SECS must be set.")
        .parse::<u64>()
        .expect("DEFAULT_WAIT_TIME_SECS must be a number.");

    macro_rules! goto_url {
        ($url:expr) => {
            {
                driver.goto($url).await?;
                sleep(Duration::from_secs(wait_time)).await;
                let current_url = driver.current_url().await?;
                current_url.as_str() == $url
            }
        };
    }

    // 2. interact
    let result = goto_url!("https://sstm.moe/login");
    if result {
        // not login yet

        let username_input = driver.find(By::Id("auth")).await?;
        let password_input = driver.find(By::Id("password")).await?;
        let login_button = driver.find(By::Id("elSignIn_submit")).await?;

        username_input.send_keys(username).await?;
        password_input.send_keys(password).await?;
        login_button.click().await?;

        sleep(Duration::from_secs(wait_time)).await;
    }

    let result = goto_url!("https://sstm.moe/login");
    if result {
        eprintln!("=> Failed to login.");
        return Err(WebDriverError::UnknownError(WebDriverErrorInfo::new("Failed to login.".to_string())));
    }

    // output JieCao
    let profile_url_result = env::var("SSTM_PROFILE_URL");
    let current_jiecao = if profile_url_result.is_ok() {
        let profile_url = profile_url_result.unwrap();
        let _ = goto_url!(profile_url.as_str());
        let jiecao_element = driver.find(By::Css("#elProfileInfoColumn > div > div:nth-child(4) > div > ul > li:nth-child(1) > span.ipsDataItem_main")).await?;
        let jiecao_text = jiecao_element.text().await?;
        // 去除text末尾的" J"
        let jiecao_text = jiecao_text.trim_end_matches(" J");
        let jiecao = jiecao_text.parse::<f32>().unwrap_or(-1.0);
        jiecao
    } else {
        -1.0
    };
    if current_jiecao >= 0.0 {
        println!("=> Current 节操 is {}.", current_jiecao);
    } else {
        println!("=> 没有设置SSTM_PROFILE_URL，无法获取当前节操。");
    }
    
    // go to the forum page
    let result = goto_url!("https://sstm.moe/forum/72-%E5%90%8C%E7%9B%9F%E7%AD%BE%E5%88%B0%E5%8C%BA/");
    if !result {
        eprintln!("=> Failed to go to the forum page.");
        return Err(WebDriverError::UnknownError(WebDriverErrorInfo::new("Failed to go to the forum page.".to_string())));
    }

    // get time to locate the latest post
    let year = Local::now().year();
    let month = Local::now().month();
    let day = Local::now().day();
    let target_string = format!("【{}/{}/{}】", year, month, day);
    let target_css_selector = format!("a[title*='{}']", target_string);
    println!("=> Target string (today date): {}", target_string);

    // target element:
    let target_element = driver.find(By::Css(target_css_selector)).await?;
    target_element.click().await?;
    sleep(Duration::from_secs(wait_time)).await;

    let current_url = driver.current_url().await?;
    if !current_url.as_str().contains("topic") {
        eprintln!("=> Failed to go to the 签到页面.");
        return Err(WebDriverError::UnknownError(WebDriverErrorInfo::new("Failed to go to the target post.".to_string())));
    }
    println!("=> Successfully went to the 签到页面.");

    let goto_reply_page_button = driver.find(By::Css("#ipsLayout_mainArea > div.ipsClearfix > ul > li > span > a")).await?;
    goto_reply_page_button.click().await?;
    sleep(Duration::from_secs(wait_time)).await;
    println!("=> Successfully went to the 回复页面.");

    let reply_input = driver.find(By::Css("#cke_1_contents > div")).await?;
    let reply_content = format!("签到喵~{}", target_string);
    reply_input.send_keys(reply_content).await?;
    println!("=> Successfully 输入回复信息.");

    // 将页面滚动到底部
    driver.execute("window.scrollTo(0, document.body.scrollHeight);", vec![]).await?;
    sleep(Duration::from_secs(wait_time)).await;

    let reply_button = driver.find(By::Css("#comments > div.cTopicPostArea.ipsBox.ipsResponsive_pull.ipsPadding.ipsSpacer_top > form > div > div.ipsComposeArea_editor > ul > li:nth-child(2) > button")).await?;
    reply_button.click().await?;
    println!("=> Successfully 点击回复按钮.");

    sleep(Duration::from_secs(wait_time)).await;
    println!("=> Successfully 签到!");

    // 2.1 output current JieCao

    // output JieCao
    let profile_url_result = env::var("SSTM_PROFILE_URL");
    let updated_jiecao = if profile_url_result.is_ok() {
        let profile_url = profile_url_result.unwrap();
        let _ = goto_url!(profile_url.as_str());
        let jiecao_element = driver.find(By::Css("#elProfileInfoColumn > div > div:nth-child(4) > div > ul > li:nth-child(1) > span.ipsDataItem_main")).await?;
        let jiecao_text = jiecao_element.text().await?;
        // 去除text末尾的" J"
        let jiecao_text = jiecao_text.trim_end_matches(" J");
        let jiecao = jiecao_text.parse::<f32>().unwrap_or(-1.0);
        jiecao
    } else {
        -1.0
    };
    if updated_jiecao >= 0.0 {
        println!("=> 节操从 {} 变为 {}.", current_jiecao, updated_jiecao);
    } else {
        println!("=> 没有设置SSTM_PROFILE_URL，无法获取当前节操。");
    }

    // 3. end
    sleep(Duration::from_secs(5)).await;
    driver.quit().await?;
    let result = end_chromedriver(chromedriver_process);
    
    // 按任意键退出
    println!("按任意键退出...");
    let _ = std::io::stdin().read_line(&mut String::new());

    result
}

fn start_chromedriver(chromedriver_path: &str, chromedriver_port: u32) -> std::process::Child {
    Command::new(chromedriver_path)
        .arg(format!("--port={}", chromedriver_port))
        .spawn()
        .expect("Failed to start chromedriver.")
}

fn end_chromedriver(mut chromedriver_process: std::process::Child) -> Result<(), WebDriverError> {
    if let Err(e) = chromedriver_process.kill() {
        eprintln!("=> Failed to kill chromedriver: {}", e);
        Err(WebDriverError::UnknownError(WebDriverErrorInfo::new(e.to_string())))
    } else {
        println!("=> Successfully killed chromedriver.");
        println!("=> Goodbye!");
        Ok(())
    }
}