// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use serde_json::json;
use std::*;

pub struct WebDriver {
    process: process::Child,
    client: reqwest::blocking::Client,
    url: String,
    session_id: String,
}

#[derive(serde::Deserialize)]
struct Success<T> {
    value: T,
}

impl Drop for WebDriver {
    fn drop(&mut self) {
        self.process.kill().ok();
        self.process.wait().ok();
    }
}

impl WebDriver {
    pub fn new(
        command: &mut process::Command,
        caps: serde_json::Value,
        port: usize,
    ) -> Result<Self, Box<dyn error::Error>> {
        let process = command
            .arg(format!("--port={}", port))
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .spawn()?;
        let client = reqwest::blocking::Client::new();
        let mut this = WebDriver {
            process: process,
            client,
            url: format!("http://localhost:{}", port),
            session_id: String::new(),
        };

        let resp = loop {
            let req = this
                .client
                .post(&format!("{}/session", this.url))
                .json(&json!({ "capabilities": caps }));
            if let Ok(e) = req.send() {
                break e;
            }
            thread::sleep(time::Duration::from_millis(125));
            if let Some(_) = this.process.try_wait()? {
                Err("The webdriver process has exited.")?;
            }
        };
        #[derive(serde::Deserialize)]
        struct Response {
            #[serde(rename(deserialize = "sessionId"))]
            session_id: String,
        }
        this.session_id = resp.json::<Success<Response>>()?.value.session_id;

        Ok(this)
    }

    pub fn set_url(&self, url: &str) -> Result<(), Box<dyn error::Error>> {
        self.post(format_args!("url"), json!({ "url": url }))
    }

    pub fn set_window_size(&self, w: usize, h: usize) -> Result<(usize, usize), Box<dyn error::Error>> {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Rect {
            width: usize,
            height: usize,
        }
        let resp: Rect = self.post(format_args!("window/rect"), Rect { width: w, height: h })?;
        Ok((resp.width, resp.height))
    }

    pub fn execute<T: serde::de::DeserializeOwned>(&self, code: &str) -> Result<T, Box<dyn error::Error>> {
        self.post(format_args!("execute/sync"), json!({ "script": code, "args": [] }))
    }

    pub fn screenshot(&self) -> Result<Vec<u8>, Box<dyn error::Error>> {
        let resp: String = self.get(format_args!("screenshot"))?;
        let data = base64::decode(&resp)?;
        Ok(data)
    }

    pub fn element(&self, selector: &str) -> Result<String, Box<dyn error::Error>> {
        #[derive(serde::Deserialize)]
        struct Response {
            #[serde(rename(deserialize = "element-6066-11e4-a52e-4f735466cecf"))]
            element: String,
        }
        let resp: Response = self.post(
            format_args!("element"),
            json!({ "using": "css selector", "value": selector }),
        )?;
        Ok(resp.element)
    }

    pub fn element_screenshot(&self, element: &str) -> Result<Vec<u8>, Box<dyn error::Error>> {
        let resp: String = self.get(format_args!("element/{}/screenshot", element))?;
        let data = base64::decode(&resp)?;
        Ok(data)
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, path: fmt::Arguments) -> Result<T, Box<dyn error::Error>> {
        let url = format!("{}/session/{}/{}", self.url, self.session_id, path);
        let resp: Success<T> = self.client.get(&url).send()?.json()?;
        Ok(resp.value)
    }

    pub fn post<T: serde::de::DeserializeOwned, U: serde::Serialize>(
        &self,
        path: fmt::Arguments,
        req: U,
    ) -> Result<T, Box<dyn error::Error>> {
        let url = format!("{}/session/{}/{}", self.url, self.session_id, path);
        let resp: Success<T> = self.client.post(&url).json(&req).send()?.json()?;
        Ok(resp.value)
    }
}
