use uaparser::*;
use worker::*;

mod utils;

static UA_DEVICE_HEADER: &str = "x-ua-device";
static UA_OS_HEADER: &str = "x-ua-os";

struct UAHeader {
    device: String,
    os: String,
}

impl UAHeader {
    fn new(device: impl Into<String>, os: impl Into<String>) -> Self {
        Self {
            device: device.into(),
            os: os.into(),
        }
    }
}

fn parse_user_agent(req: &Request) -> Result<UAHeader> {
    let regexes = include_bytes!("./regexes.yaml");
    match UserAgentParser::from_bytes(regexes) {
        Ok(ua_parser) => {
            let user_agent = req.headers().get("user-agent").unwrap().unwrap();
            let ua = ua_parser.parse(user_agent.as_str());
            console_log!("user-agent: {}", user_agent);
            console_log!("ua: {}", ua.os.family);
            Ok(UAHeader::new(ua.device.family.to_string(), ua.os.family.to_string()))
        }
        Err(e) => {
            console_log!("{} =====", e);
            Err(worker::Error::from(e.to_string()))
        }
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    let mut new_req = req.clone_mut().unwrap();
    let ua = parse_user_agent(&req);
    match ua {
        Ok(ua_header) => {
            match new_req.headers_mut() {
                Ok(headers) => {
                    headers.set(UA_DEVICE_HEADER, ua_header.device.as_str()).unwrap();
                    headers.set(UA_OS_HEADER, ua_header.os.as_str()).unwrap();
                    console_log!("{}", headers.get(UA_DEVICE_HEADER).unwrap().unwrap())
                }
                Err(e) => {
                    console_log!("{} header not exist", e.to_string())
                }
            }
        }
        _ => {
            console_log!("ua not exist")
        }
    }
    let response = Fetch::Request(new_req).send().await;
    match response {
        Ok(res) => {
            Ok(Response::from(res))
        }
        Err(e) => {
            Response::error(e.to_string(), 400)
        }
    }
}
