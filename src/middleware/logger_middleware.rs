use actix_web::dev::ServiceRequest;
use crate::utils::message_util::MessageUtil;

pub struct LoggerMiddleware;

impl LoggerMiddleware {

    pub fn log_request(req: ServiceRequest, path: &str) {
        let method = req.method().to_string();
        let headers = req.headers().to_owned();
        let query_string = req.query_string().to_string();

        MessageUtil::info(&format!(
            "Received request: {} {} | Headers: {:?} | Query: {}",
            method, path, headers, query_string
        ));
    }

    pub fn log_response(status_code: u16) {
        println!("Sending response with status code: {}", status_code);
    }
}