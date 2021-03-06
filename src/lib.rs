mod utils;

use crate::utils::{to_default, to_option};
use thirtyfour::prelude::*;

pub type AmazonBrowserResult<T> = WebDriverResult<T>;

#[derive(Debug, Clone)]
pub struct Log {
    pub hash: String,
    pub name: String,
    pub price: i32,
    pub purchased_at: String,
}

pub struct AmazonBrowser {
    driver: Option<Box<WebDriver>>,
    email: String,
    password: String,
}

impl AmazonBrowser {
    pub async fn new(
        email: &str,
        password: &str,
        user_data_dir: &str,
    ) -> WebDriverResult<AmazonBrowser> {
        let caps_args = &format!(
            r#"--user-data-dir="/Users/a.kano/Library/Application Support/Google/Chrome/{}""#,
            user_data_dir.to_string()
        );
        let mut caps = DesiredCapabilities::chrome();
        let _ = caps.add_chrome_arg(caps_args);
        let driver = WebDriver::new("http://localhost:4444", &caps).await?;
        Ok(AmazonBrowser {
            driver: Some(Box::new(driver)),
            email: email.to_string(),
            password: password.to_string(),
        })
    }
    pub async fn quit(&mut self) -> WebDriverResult<()> {
        let driver = self.check_out();
        driver.quit().await?;
        Ok(())
    }
}
impl AmazonBrowser {
    fn check_out(&mut self) -> WebDriver {
        to_default(&mut self.driver)
    }
    fn check_in(&mut self, driver: WebDriver) {
        to_option(&mut self.driver, driver);
    }
}

impl AmazonBrowser {
    async fn title(&mut self) -> WebDriverResult<String> {
        let driver = self.check_out();
        let title = driver.title().await?;
        self.check_in(driver);
        Ok(title)
    }
    async fn goto_home(&mut self) -> WebDriverResult<()> {
        let driver = self.check_out();
        let home_url = "https://www.amazon.co.jp/ref=nav_logo";
        driver.get(home_url).await?;
        self.check_in(driver);
        Ok(())
    }
    async fn goto_login(&mut self) -> WebDriverResult<()> {
        let driver = self.check_out();
        let login_url = "https://www.amazon.co.jp/ap/signin?ie=UTF8&openid.pape.max_auth_age=0&openid.return_to=https%3A%2F%2Fwww.amazon.co.jp%2Fgp%2Fcss%2Fhomepage.html%3Fref_%3Dnav_youraccount_switchacct&openid.identity=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0%2Fidentifier_select&openid.assoc_handle=jpflex&_encoding=UTF8&openid.mode=checkid_setup&ignoreAuthState=1&openid.claimed_id=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0%2Fidentifier_select&openid.ns=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0";
        driver.get(login_url).await?;
        self.check_in(driver);
        Ok(())
    }
    async fn goto_logout(&mut self) -> WebDriverResult<()> {
        let driver = self.check_out();
        let logout_url = "https://www.amazon.co.jp/gp/flex/sign-out.html?path=%2Fgp%2Fyourstore%2Fhome&signIn=1&useRedirectOnSuccess=1&action=sign-out&ref_=nav_AccountFlyout_signout";
        driver.get(logout_url).await?;
        self.check_in(driver);
        Ok(())
    }
    async fn login(&mut self) -> WebDriverResult<()> {
        self.goto_logout().await?;
        self.goto_login().await?;

        let driver = self.check_out();

        let element_email = driver.find_element(By::Id("ap_email")).await?;
        element_email.send_keys(&self.email).await?;
        let element_email_button = driver.find_element(By::Id("continue")).await?;
        element_email_button.click().await?;

        let element_password = driver.find_element(By::Id("ap_password")).await?;
        element_password.send_keys(&self.password).await?;
        let element_password_button = driver.find_element(By::Id("signInSubmit")).await?;
        element_password_button.click().await?;

        self.check_in(driver);

        self.goto_home().await?;
        Ok(())
    }
    async fn goto_history(&mut self, year: &i32) -> WebDriverResult<()> {
        let driver = self.check_out();
        let history_url = format!("https://www.amazon.co.jp/gp/your-account/order-history?opt=ab&digitalOrders=1&unifiedOrders=1&returnTo=&__mk_ja_JP=%E3%82%AB%E3%82%BF%E3%82%AB%E3%83%8A&orderFilter=year-{}", year);
        driver.get(history_url).await?;
        self.check_in(driver);
        Ok(())
    }
    async fn nav_message(&mut self) -> WebDriverResult<String> {
        let driver = self.check_out();
        let message = driver
            .find_element(By::Id("glow-ingress-line1"))
            .await?
            .text()
            .await?;
        self.check_in(driver);
        Ok(message)
    }
    async fn year_in_prompt(&mut self) -> WebDriverResult<String> {
        let driver = self.check_out();
        let message = driver
            .find_element(By::ClassName("a-dropdown-prompt"))
            .await?
            .text()
            .await?;
        self.check_in(driver);
        Ok(message)
    }
}

use async_recursion::async_recursion;
impl AmazonBrowser {
    #[async_recursion]
    async fn scrape_history(
        &mut self,
        result: WebDriverResult<Vec<Log>>,
        range: &Range,
    ) -> WebDriverResult<Vec<Log>> {
        use crate::utils::to_naive_date;
        use chrono::NaiveDate;
        use regex::Regex;

        let driver = self.check_out();

        let mut result = result.unwrap();

        let mut purchased_at = to_naive_date(range.end());
        let groups = driver.find_elements(By::ClassName("a-box-group")).await?;
        for n in 0..groups.len() {
            let groups = driver.find_elements(By::ClassName("a-box-group")).await?;
            let group = groups.get(n).unwrap();
            let purchased_at_str = group
                .find_element(By::ClassName("a-span3"))
                .await?
                .find_element(By::ClassName("a-color-secondary.value"))
                .await?
                .text()
                .await?;
            purchased_at = NaiveDate::parse_from_str(&purchased_at_str, "%Y???%m???%d???").unwrap();

            // ???????????????????????????????????????
            if purchased_at > to_naive_date(range.end()) {
                continue;
            }
            // ??????????????????
            if purchased_at < to_naive_date(range.start()) {
                break;
            }
            // ???????????????????????????????????????????????????
            {
                group
                    .find_element(By::ClassName("a-unordered-list"))
                    .await?
                    .find_elements(By::ClassName("a-link-normal"))
                    .await?
                    .first()
                    .unwrap()
                    .click()
                    .await?; // -> ???????????????????????????????????????
                let log_elements = driver
                    .find_elements(By::ClassName("a-fixed-left-grid-inner"))
                    .await?;

                // TODO: ??2?????????????????????????????????????????????????????????
                for log_element in &log_elements {
                    let countability = log_element
                        .find_element(By::ClassName("item-view-qty"))
                        .await;
                    let count: i32 = match countability {
                        Ok(e) => e.text().await?.parse().unwrap(),
                        _ => 1,
                    };
                    let href_str = log_element
                        .find_element(By::ClassName("a-link-normal"))
                        .await?
                        .get_attribute("href")
                        .await?
                        .unwrap();
                    let re = Regex::new(r"/gp/product/(\w{10})/ref=").unwrap();
                    let caps = re.captures(&href_str).unwrap();
                    let hash = caps.get(1).unwrap().as_str().to_string();

                    let name = log_element
                        .find_element(By::ClassName("a-col-right"))
                        .await?
                        .find_element(By::ClassName("a-link-normal"))
                        .await?
                        .text()
                        .await?;
                    let price_raw_str = log_element
                        .find_element(By::ClassName("a-color-price"))
                        .await?
                        .text()
                        .await?;
                    let price_str: String = price_raw_str.trim().replace(&['???', ' ', ','][..], "");
                    let price = price_str.parse::<i32>().unwrap();

                    let new = Log {
                        hash: hash,
                        name: name,
                        price: price,
                        purchased_at: purchased_at.to_string(),
                    };
                    for _ in 0..count {
                        result.push(new.clone());
                        println!("??????????????????: {:?}", new);
                    }
                }
                driver.back().await?;
            }
        }

        // ??????????????????
        if purchased_at < to_naive_date(range.start()) {
            self.check_in(driver);
            return Ok(result);
        }
        if let Ok(_) = driver
            .find_element(By::ClassName("a-disabled.a-last"))
            .await
        {
            self.check_in(driver);
        } else if let Ok(e) = driver.find_element(By::ClassName("a-last")).await {
            e.click().await?;
            self.check_in(driver);
            result = self.scrape_history(Ok(result), range).await?
        } else {
            self.check_in(driver);
        }
        Ok(result)
    }
}

use range::Range;
impl AmazonBrowser {
    pub async fn extract(&mut self, range: &Range) -> WebDriverResult<Vec<Log>> {
        let mut logs = Ok(vec![]);
        use crate::utils::to_year;
        let end = to_year(range.end());
        let start = to_year(range.start());
        let years = (start..=end).rev().collect::<Vec<i32>>();
        self.login().await?;
        self.goto_home().await?; // Amazon????????????????????????????????????????????????????????????
        println!("????????????????????????????????????");
        for year in &years {
            self.goto_history(year).await?;
            logs = self.scrape_history(logs, range).await;
        }
        println!("????????????????????????????????????");
        logs
    }
    async fn goto_first_history(&mut self) -> WebDriverResult<()> {
        let driver = self.check_out();
        let first_url = "https://www.amazon.co.jp/gp/css/order-history?ref_=nav_orders_first";
        driver.get(first_url).await?;
        self.check_in(driver);
        Ok(())
    }
    fn to_year_num_from_str(maybe_year_str: &str) -> i32 {
        use regex::Regex;
        let re = Regex::new(r"(\d{4})???").unwrap();

        use chrono::prelude::*;
        use chrono::{Duration, Local};
        let yesterday = (Local::today() + Duration::days(-1)).naive_local();
        let yesterday_year: i32 = yesterday.year();

        let result = re
            .captures(&maybe_year_str)
            .and_then(|caps| caps.get(1))
            .and_then(|year| year.as_str().parse::<i32>().ok());
        result.unwrap_or(yesterday_year)
    }
    pub async fn most_formerly_date(&mut self) -> WebDriverResult<String> {
        self.login().await?;
        self.goto_first_history().await?;

        let driver = self.check_out();

        driver
            .find_element(By::Id("a-autoid-1-announce"))
            .await?
            .click()
            .await?;
        let dropdown_elements = driver
            .find_elements(By::ClassName("a-dropdown-item"))
            .await?;

        // assert_eq!(dropdown_elements.len(), 7);

        use futures::prelude::*;
        let dropdown_ok_strs = stream::iter(&dropdown_elements)
            .then(|e| e.text())
            .collect::<Vec<WebDriverResult<String>>>()
            .await;

        self.check_in(driver);

        let dropdown_strs = dropdown_ok_strs
            .iter()
            .map(|ok_str| ok_str.as_ref().unwrap().clone())
            .collect::<Vec<String>>();

        // assert_eq!(
        //     dropdown_strs,
        //     vec![
        //         "??????30??????",
        //         "??????3??????",
        //         "2022???",
        //         "2021???",
        //         "2020???",
        //         "2019???",
        //         "2018???"
        //     ]
        // );

        let all_years: Vec<i32> = dropdown_strs
            .iter()
            .map(|maybe_year_str| Self::to_year_num_from_str(maybe_year_str))
            .collect();

        // assert_eq!(all_years, vec![2022, 2022, 2022, 2021, 2020, 2019, 2018]);

        let most_formerly_year = all_years.iter().min().unwrap();
        let most_formerly_date = format!("{}-01-01", most_formerly_year);
        Ok(most_formerly_date)
    }
}

#[cfg(test)]
mod tests {
    use super::{AmazonBrowser, Log};
    use range::Range;
    use thirtyfour::prelude::*;
    use tokio;

    #[test]
    fn to_year_num_from_str?????????????????????() {
        assert_eq!(AmazonBrowser::to_year_num_from_str("??????30??????"), 2022);
        assert_eq!(AmazonBrowser::to_year_num_from_str("??????3??????"), 2022);
        assert_eq!(AmazonBrowser::to_year_num_from_str("2022???"), 2022);
        assert_eq!(AmazonBrowser::to_year_num_from_str("2018???"), 2018);
    }
    #[tokio::test]
    async fn ????????????????????????????????????????????????() -> WebDriverResult<()> {
        use dotenv::dotenv;
        use std::env;
        dotenv().ok();
        let email = env::var("AMAZON_EMAIL").expect("AMAZON_EMAIL must be set");
        let pass = env::var("AMAZON_PASSWORD").expect("AMAZON_PASSWORD must be set");
        let mut browser = AmazonBrowser::new(&email, &pass, "formerly_year_correct").await?;
        let most_formerly_date = "2018-01-01";
        assert_eq!(browser.most_formerly_date().await?, most_formerly_date);
        browser.quit().await?;
        Ok(())
    }
    #[tokio::test]
    async fn ????????????????????????????????????() -> WebDriverResult<()> {
        use dotenv::dotenv;
        use std::env;
        dotenv().ok();
        let email = env::var("AMAZON_EMAIL").expect("AMAZON_EMAIL must be set");
        let pass = env::var("AMAZON_PASSWORD").expect("AMAZON_PASSWORD must be set");
        let mut browser = AmazonBrowser::new(&email, &pass, "signin").await?;
        browser.goto_logout().await?;
        browser.goto_login().await?;
        let login_title = "Amazon???????????????";
        assert_eq!(browser.title().await?, login_title);
        browser.goto_logout().await?;
        browser.quit().await?;
        Ok(())
    }
    #[tokio::test]
    async fn ??????????????????home??????????????????() -> WebDriverResult<()> {
        use dotenv::dotenv;
        use std::env;
        dotenv().ok();
        let email = env::var("AMAZON_EMAIL").expect("AMAZON_EMAIL must be set");
        let pass = env::var("AMAZON_PASSWORD").expect("AMAZON_PASSWORD must be set");
        let mut browser = AmazonBrowser::new(&email, &pass, "home").await?;
        browser.login().await?;
        let not_logged_in_nav_message = "???????????????";
        let logged_in_nav_message = "???????????? ???????????????";
        let page_message = browser.nav_message().await?;
        assert_ne!(page_message, not_logged_in_nav_message);
        assert_eq!(page_message, logged_in_nav_message);
        browser.goto_logout().await?;
        browser.quit().await?;
        Ok(())
    }
    #[tokio::test]
    async fn ???????????????????????????home????????????????????????????????????() -> WebDriverResult<()> {
        use dotenv::dotenv;
        use std::env;
        dotenv().ok();
        let email = env::var("AMAZON_EMAIL").expect("AMAZON_EMAIL must be set");
        let pass = env::var("AMAZON_PASSWORD").expect("AMAZON_PASSWORD must be set");
        let mut browser = AmazonBrowser::new(&email, &pass, "no_home").await?;
        browser.goto_logout().await?;
        browser.goto_home().await?;
        let not_logged_in_nav_message = "???????????????";
        let logged_in_nav_message = "???????????? ???????????????";
        let page_message = browser.nav_message().await?;
        assert_eq!(page_message, not_logged_in_nav_message);
        assert_ne!(page_message, logged_in_nav_message);
        browser.quit().await?;
        Ok(())
    }
    #[tokio::test]
    async fn history????????????????????????????????????????????????() -> WebDriverResult<()> {
        use dotenv::dotenv;
        use std::env;
        dotenv().ok();
        let email = env::var("AMAZON_EMAIL").expect("AMAZON_EMAIL must be set");
        let pass = env::var("AMAZON_PASSWORD").expect("AMAZON_PASSWORD must be set");
        let mut browser = AmazonBrowser::new(&email, &pass, "history2020").await?;
        browser.login().await?;
        browser.goto_history(&2020).await?;
        let year_in_prompot = "2020???";
        let prompt_year = browser.year_in_prompt().await?;
        assert_eq!(prompt_year, year_in_prompot);
        browser.quit().await?;
        Ok(())
    }
    #[tokio::test]
    async fn ?????????????????????????????????????????????????????????????????????() -> WebDriverResult<()> {
        use dotenv::dotenv;
        use std::env;
        dotenv().ok();
        let email = env::var("AMAZON_EMAIL").expect("AMAZON_EMAIL must be set");
        let pass = env::var("AMAZON_PASSWORD").expect("AMAZON_PASSWORD must be set");
        let mut browser = AmazonBrowser::new(&email, &pass, "page_over").await?;
        // browser.login().await?; // extract()??????????????????
        let span = Range::new("2021-08-17", "2021-09-18"); // ???????????????????????????
        let logs = browser.extract(&span).await?;
        assert_eq!(logs.len(), 2);
        browser.quit().await?;
        Ok(())
    }
    #[tokio::test]
    async fn ?????????????????????????????????????????????????????????() -> WebDriverResult<()> {
        use dotenv::dotenv;
        use std::env;
        dotenv().ok();
        let email = env::var("AMAZON_EMAIL").expect("AMAZON_EMAIL must be set");
        let pass = env::var("AMAZON_PASSWORD").expect("AMAZON_PASSWORD must be set");
        let mut browser = AmazonBrowser::new(&email, &pass, "gift").await?;
        // browser.login().await?; // extract()??????????????????
        let span = Range::new("2020-07-17", "2020-07-17");
        let logs = browser.extract(&span).await?;
        // 2020.7.17 ???3,299 ??????????????????
        assert_eq!(
            logs.iter().filter(|&log| log.hash == "B088KDK163").count(),
            1
        );
        browser.quit().await?;
        Ok(())
    }
    #[tokio::test]
    async fn ????????????????????????????????????????????????????????????????????????() -> WebDriverResult<()> {
        use dotenv::dotenv;
        use std::env;
        dotenv().ok();
        let email = env::var("AMAZON_EMAIL").expect("AMAZON_EMAIL must be set");
        let pass = env::var("AMAZON_PASSWORD").expect("AMAZON_PASSWORD must be set");
        let mut browser = AmazonBrowser::new(&email, &pass, "double").await?;
        // browser.login().await?; // extract()??????????????????
        let span = Range::new("2021-10-19", "2021-10-19");
        let logs = browser.extract(&span).await?;
        assert_eq!(logs.len(), 4);
        browser.quit().await?;
        Ok(())
    }
    #[tokio::test]
    async fn name???????????????????????????????????????() -> WebDriverResult<()> {
        use dotenv::dotenv;
        use std::env;
        dotenv().ok();
        let email = env::var("AMAZON_EMAIL").expect("AMAZON_EMAIL must be set");
        let pass = env::var("AMAZON_PASSWORD").expect("AMAZON_PASSWORD must be set");
        let mut browser = AmazonBrowser::new(&email, &pass, "double").await?;
        // browser.login().await?; // extract()??????????????????
        let span = Range::new("2021-11-08", "2021-11-08");
        let logs = browser.extract(&span).await?;
        assert_eq!(
            logs.iter()
                .filter(|&log| log.name
                    == "AGF ??????????????? ??????????????? ??????????????? 100??? ??? ??????????????????????????? ???")
                .count(),
            1
        );
        assert_eq!(logs.len(), 1);
        browser.quit().await?;
        Ok(())
    }
    #[tokio::test]
    async fn ????????????????????????????????????????????????????????????() -> WebDriverResult<()> {
        let logs = vec![Log {
            hash: "B088KDK163".to_string(),
            name: "name".to_string(),
            price: 42,
            purchased_at: "2021-07-17".to_string(),
        }];
        assert_eq!(
            logs.iter().filter(|&log| log.hash == "B088KDK163").count(),
            1
        );
        Ok(())
    }
    #[ignore]
    #[tokio::test]
    async fn headless????????????????????????extract????????????????????????????????????????????????????????????(
    ) -> WebDriverResult<()> {
        use dotenv::dotenv;
        use std::env;
        dotenv().ok();
        let email = env::var("AMAZON_EMAIL").expect("AMAZON_EMAIL must be set");
        let pass = env::var("AMAZON_PASSWORD").expect("AMAZON_PASSWORD must be set");
        let mut browser = AmazonBrowser::new(&email, &pass, "check_in_bug").await?;
        let span = Range::new("2018-01-01", "2022-01-09");
        let logs = browser.extract(&span).await?;
        assert_eq!(logs.len(), 219);
        browser.quit().await?;
        Ok(())
    }
}
