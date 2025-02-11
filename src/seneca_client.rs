use crate::utils::{generate_hex_string, generate_time_vec};

use chrono::{prelude::*, Duration};
use url::form_urlencoded::byte_serialize;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::error::Error;

pub struct SenecaClient {
    client: Client,
    api_key: String,
    refresh_token: String,
    access_key: String,
    pub user_id: String,
}

impl SenecaClient {
    pub async fn new(api_key: String, refresh_token: String) -> Result<Self, Box<dyn Error>> {
        let client = Client::new();

        let mut self_to_return = Self {
            client,
            api_key,
            refresh_token,
            access_key: String::new(),
            user_id: String::new(),
        };

        self_to_return.access_key = self_to_return.get_access_key().await?;
        self_to_return.user_id = Self::get_user_id(&mut self_to_return).await?;
        Ok(self_to_return)
    }

    async fn get_access_key(&self) -> Result<String, Box<dyn Error>> {
        let url = format!("https://securetoken.googleapis.com/v1/token?key={}", self.api_key);

        let body = json!({
            "grant_type": "refresh_token",
            "refresh_token": self.refresh_token,
        });

        let response = self.client.post(url).json(&body).send().await?;

        if response.status().is_success() {
            let body = response.json::<Value>().await?;
            Ok(body["access_token"].as_str().unwrap().to_string())
        } else {
            if response.status() == 400 {
                eprintln!("🚧 Invalid refresh token or API key");
            }
            Err(Box::new(response.error_for_status().unwrap_err()))
        }
    }

    async fn get_user_id(&mut self) -> Result<String, Box<dyn Error>> {
        let url = "https://user-info.app.senecalearning.com/api/user-info/me";
        
        let mut has_refreshed = false;
        loop {
            let headers: HeaderMap = Self::assemble_headers(vec![
                ("Host", "user-info.app.senecalearning.com"),
                (
                    "User-Agent",
                    "Mozilla/5.0 (X11; Linux x86_64; rv:134.0) Gecko/20100101 Firefox/134.0",
                ),
                ("Accept", "*/*"),
                ("Accept-Language", "en-GB,en;q=0.5"),
                ("Accept-Encoding", "gzip, deflate, br, zstd"),
                ("Referer", "https://app.senecalearning.com/"),
                ("access-key", &self.access_key),
                ("Content-Type", "application/json"),
                (
                    "correlationId",
                    "1737330516472::76115c42-02c9-4d56-0000-000000000000",
                ),
                ("user-region", "GB"),
                ("Origin", "https://app.senecalearning.com"),
                ("DNT", "1"),
                ("Sec-GPC", "1"),
                ("Sec-Fetch-Dest", "empty"),
                ("Sec-Fetch-Mode", "cors"),
                ("Sec-Fetch-Site", "same-site"),
                ("Connection", "keep-alive"),
                ("host", "user-info.app.senecalearning.com"),
            ]);

            let response = self.client.get(url).headers(headers).send().await?;

            if response.status().is_success() {
                let body = response.json::<Value>().await?;
                break Ok(body["userId"].as_str().unwrap().to_string());
            } else {
                if response.status() == StatusCode::UNAUTHORIZED {
                    if has_refreshed {
                        eprintln!("🚧 Invalid access key");
                        break Err(Box::new(response.error_for_status().unwrap_err()));
                    }
                    self.access_key = self.get_access_key().await?;
                    has_refreshed = true;
                    continue;
                }
                break Err(Box::new(response.error_for_status().unwrap_err()));
            }
        }
    }

    async fn get_signed_url(
        &mut self,
        course_id: &str,
        section_id: &str,
    ) -> Result<String, Box<dyn Error>> {
        let url = format!(
            "https://course.app.senecalearning.com/api/courses/{}/signed-url?sectionId={}&contentTypes=standard,hardestQuestions",
            course_id, section_id
        );

        let mut has_refreshed = false;
        loop {
            let headers: HeaderMap = Self::assemble_headers(vec![
                ("Host", "course.app.senecalearning.com"),
                (
                    "User-Agent",
                    "Mozilla/5.0 (X11; Linux x86_64; rv:134.0) Gecko/20100101 Firefox/134.0",
                ),
                ("Accept", "*/*"),
                ("Accept-Language", "en-GB,en;q=0.5"),
                ("Accept-Encoding", "gzip, deflate, br, zstd"),
                ("Referer", "https://app.senecalearning.com/"),
                // ("access-key", &self.access_key),
                ("Content-Type", "application/json"),
                (
                    "correlationId",
                    "1737330516472::76115c42-02c9-4d56-0000-000000000000",
                ),
                ("user-region", "GB"),
                ("Origin", "https://app.senecalearning.com"),
                ("DNT", "1"),
                ("Sec-GPC", "1"),
                ("Sec-Fetch-Dest", "empty"),
                ("Sec-Fetch-Mode", "cors"),
                ("Sec-Fetch-Site", "same-site"),
                ("Connection", "keep-alive"),
                ("host", "course.app.senecalearning.com"),
            ]);

            let response = self.client.get(&url).headers(headers).send().await?;

            if response.status().is_success() {
                let body = response.json::<Value>().await?;
                break Ok(body["url"].as_str().unwrap().to_string())
            } else {
                if response.status() == StatusCode::UNAUTHORIZED {
                    if has_refreshed {
                        eprintln!("🚧 Invalid access key");
                        break Err(Box::new(response.error_for_status().unwrap_err()));
                    }
                    self.access_key = self.get_access_key().await?;
                    has_refreshed = true;
                    continue;
                }
                break Err(Box::new(response.error_for_status().unwrap_err()))
            }
        }
    }

    pub async fn get_contents(
        &mut self,
        course_id: &str,
        section_id: &str,
    ) -> Result<(String, String, Value), Box<dyn Error>> {
        let url = self.get_signed_url(course_id, section_id).await?;

        let mut has_refreshed = false;
        loop {
            let response = self.client.get(&url).send().await?;

            if response.status().is_success() {
                let body = response.json::<Value>().await?;
                break Ok((body["number"].as_str().unwrap_or("").to_string(), body["title"].as_str().unwrap_or("").to_string(), body["contents"].clone()))
            } else {
                if response.status() == StatusCode::UNAUTHORIZED {
                    if has_refreshed {
                        eprintln!("🚧 Invalid access key");
                        break Err(Box::new(response.error_for_status().unwrap_err()));
                    }
                    self.access_key = self.get_access_key().await?;
                    has_refreshed = true;
                    continue;
                }
                break Err(Box::new(response.error_for_status().unwrap_err()))
            }
        }
    }

    pub async fn run_solver(
        &mut self,
        course_id: &str,
        section_id: &str,
        content: &Value,
    ) -> Result<(), Box<dyn Error>> {
        let content_id = content["id"].as_str().unwrap();

        let session_id = format!(
            "{}-{}-{}-{}-{}",
            generate_hex_string(4),
            generate_hex_string(2),
            generate_hex_string(2),
            generate_hex_string(2),
            generate_hex_string(6)
        );
        
        let non_question_module_types = vec!["concept", "video", "image", "delve"];
        
        let content_modules = content["contentModules"].as_array().unwrap();
        let content_modules_len = content_modules.len();

        let now = Utc::now();

        let (mut min, mut max) = (Duration::seconds(5), Duration::seconds(25));

        if std::env::args()
        .collect::<Vec<String>>()
        .contains(&"--duration-farm".to_string())
        {
            min = Duration::minutes(30);
            max = Duration::minutes(45);
        }

        let (started, module_times) = generate_time_vec(now, min, max, content_modules_len);
        
        let mut modules = Vec::<Value>::new();

        let mut data = json!({
            "platform": "seneca",
            "clientVersion": "3.0.4",
            "userId": self.user_id,
            "userLevelFeatureFlagValue": "control",
            "session": {
                "sessionId": session_id,
                "courseId": &course_id,
                "timeStarted": started.to_rfc3339_opts(SecondsFormat::Secs, false),
                "timeFinished": now.to_rfc3339_opts(SecondsFormat::Secs, false),
                "startingProficiency": 0,
                "endingProficiency": 0.5,
                "startingCourseProficiency": 0.003601579633505109,
                "endingCourseProficiency": 0.04580470162748644,
                "endingCourseScore": 0.07210750573582432,
                "sessionScore": 1,
                "completed": true,
                "modulesCorrect": content_modules_len,
                "modulesIncorrect": 0,
                "averageScore": 1,
                "modulesGaveUp": 0,
                "modulesStudied": content_modules_len,
                "modulesTested": content_modules_len,
                "sessionType": "adaptive",
                "sectionIds": [&section_id],
                "contentIds": [content_id],
                "options": {
                    "hasHardestQuestionContent": if let Some(content_type) = content["contentType"].as_str() {
                        content_type == "hardestQuestions"
                    } else {
                        false
                    },
                },
            },
            "modules": [],
        });

        let module_template = json!({
            "sessionId": session_id,
            "moduleOrder": 0,
            "moduleId": "",
            "timeStarted": "",
            "timeFinished": "",
            "gaveUp": false,
            "submitted": true,
            "completed": true,
            "testingActive": true,
            "content": {},
            "score": 1,
            "moduleScore": {
                "score": 1,
            },
            "userAnswer": {},
            "courseId": course_id,
            "sectionId": section_id,
            "contentId": content_id,
        });

        let mut non_questions: u64 = 0;

        // Loop over every content module
        for (content_module_no, content_module) in content_modules.iter().enumerate() {
            let content_module = content_module.as_object().unwrap();

            let started_module = module_times[content_module_no];
            let finished_module = module_times[content_module_no + 1];

            let mut module = module_template.clone();
            module["moduleOrder"] = json!(content_module_no);
            module["moduleId"] = json!(content_module["id"].as_str().unwrap());
            module["moduleType"] = json!(content_module["moduleType"].as_str().unwrap());
            module["timeStarted"] = json!(started_module.to_rfc3339_opts(SecondsFormat::Secs, false));
            module["timeFinished"] = json!(finished_module.to_rfc3339_opts(SecondsFormat::Secs, false));
            
            // Special handling for non-question modules
            if non_question_module_types.contains(&content_module["moduleType"].as_str().unwrap()) {
                module["submitted"] = json!(false);
                module["testingActive"] = json!(false);
                module["score"] = json!(0);
                module.as_object_mut().unwrap().remove("moduleScore");
                module.as_object_mut().unwrap().remove("userAnswer");

                // Increment non_questions counter
                non_questions += 1;
            } else if module["moduleType"].as_str().unwrap() == "toggles" {
                module["userAnswer"] = json!([]);
            }

            modules.push(module);
        }

        data["modules"] = json!(modules);

        data["session"]["modulesCorrect"] =
            json!(data["session"]["modulesCorrect"].as_u64().unwrap() - non_questions);
        data["session"]["modulesTested"] =
            json!(data["session"]["modulesTested"].as_u64().unwrap() - non_questions);

        let url = "https://stats.app.senecalearning.com/api/stats/sessions";

        let mut has_refreshed = false;
        loop {
            let headers = Self::assemble_headers(vec![
                ("Host", "stats.app.senecalearning.com"),
                (
                    "User-Agent",
                    "Mozilla/5.0 (X11; Linux x86_64; rv:134.0) Gecko/20100101 Firefox/134.0",
                ),
                ("Accept", "*/*"),
                ("Accept-Language", "en-GB,en;q=0.5"),
                ("Accept-Encoding", "gzip, deflate, br, zstd"),
                ("Content-Type", "application/json"),
                ("Referer", "https://app.senecalearning.com/"),
                ("access-key", &self.access_key),
                (
                    "correlationId",
                    "1737330516472::76115c42-02c9-4d56-0000-000000000000",
                ),
                ("user-region", "GB"),
                ("Origin", "https://app.senecalearning.com"),
                ("DNT", "1"),
                ("Sec-GPC", "1"),
                ("Sec-Fetch-Dest", "empty"),
                ("Sec-Fetch-Mode", "cors"),
                ("Sec-Fetch-Site", "same-site"),
                ("Connection", "keep-alive"),
                ("host", "stats.app.senecalearning.com"),
            ]);

            let response = self
                .client
                .post(url)
                .headers(headers)
                .json(&data)
                .send()
                .await?;

            // println!("{}", data.to_string());

            if response.status().is_success() {
                break Ok(())
            } else {
                if response.status() == StatusCode::UNAUTHORIZED {
                    if has_refreshed {
                        eprintln!("🚧 Invalid access key");
                        break Err(Box::new(response.error_for_status().unwrap_err()));
                    }
                    self.access_key = self.get_access_key().await?;
                    has_refreshed = true;
                    continue;
                }
                break Err(Box::new(response.error_for_status().unwrap_err()))
            }
        }
    }

    pub async fn get_assignments(&mut self) -> Result<Vec<Value>, Box<dyn Error>> {
        let one_month_ago = Utc::now() - Duration::days(30);
        let url = 
            format!("https://assignments.app.senecalearning.com/api/students/me/assignments?limit=500&date={}&archived=false",
            byte_serialize(one_month_ago.to_rfc3339_opts(SecondsFormat::Secs, false).as_bytes()).collect::<String>());

        let mut has_refreshed = false;
        loop {
            let headers = Self::assemble_headers(vec![
                ("Host", "assignments.app.senecalearning.com"),
                (
                    "User-Agent",
                    "Mozilla/5.0 (X11; Linux x86_64; rv:134.0) Gecko/20100101 Firefox/134.0",
                ),
                ("Accept", "*/*"),
                ("Accept-Language", "en-GB,en;q=0.5"),
                ("Accept-Encoding", "gzip, deflate, br, zstd"),
                ("Referer", "https://app.senecalearning.com/"),
                ("access-key", &self.access_key),
                (
                    "correlationId",
                    "1737330516472::76115c42-02c9-4d56-0000-000000000000",
                ),
                ("Content-Type", "application/json"),
                ("X-Amz-Date", "20250122T190413Z"),
                ("user-region", "GB"),
                ("Origin", "https://app.senecalearning.com"),
                ("DNT", "1"),
                ("Sec-GPC", "1"),
                ("Sec-Fetch-Dest", "empty"),
                ("Sec-Fetch-Mode", "cors"),
                ("Sec-Fetch-Site", "same-site"),
                ("Connection", "keep-alive"),
                ("host", "assignments.app.senecalearning.com"),
            ]);

            let response = self.client.get(&url).headers(headers).send().await?;

            if response.status().is_success() {
                let body = response.json::<Value>().await?;
                break Ok(body.as_object().unwrap()["items"]
                    .as_array()
                    .unwrap()
                    .to_vec())
            } else {
                if let Err(err) = response.error_for_status() {
                    if err.status() == Some(StatusCode::UNAUTHORIZED) {
                        if has_refreshed {
                            eprintln!("🚧 Invalid access key");
                            break Err(Box::new(err));
                        }
                        self.access_key = self.get_access_key().await?;
                        has_refreshed = true;
                        continue;
                    }
                    break Err(Box::new(err))
                } else {
                    break Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to fetch assignments",
                    )))
                }
            }
        }
    }

    fn assemble_headers(headers: Vec<(&str, &str)>) -> HeaderMap {
        headers
            .iter()
            .map(|(key, value)| (key.parse().unwrap(), HeaderValue::from_str(value).unwrap()))
            .collect()
    }
}
