use actix_web::dev::ServiceRequest;
use actix_web::FromRequest;
use crate::utils::message_util::MessageUtil;

pub struct LoggerMiddleware;

impl LoggerMiddleware {

    pub fn log_request(req: &ServiceRequest, path: &str, request_id: &uuid::Uuid) {
        let method = req.method().to_string();
        let headers = req.headers();
        let query_string = req.query_string();
        
        let mut message = format!(
            "\n┌─ Incoming Request ─────────────────────────────────────\n\
         │ Request ID: {}\n\
         │ Method: {}\n\
         │ Path: {}\n",
            request_id,method, path
        );
        
        if !query_string.is_empty() {
            message.push_str(&format!("│ Query: {}\n", query_string));
        }
        
        message.push_str("│ Headers:\n");
        for (name, value) in headers.iter() {
            if let Ok(value_str) = value.to_str() {
                message.push_str(&format!("│   {}: {}\n", name, value_str));
            }
        }
        
        if let Some(content_length) = headers.get("content-length") {
            if let Ok(length_str) = content_length.to_str() {
                message.push_str(&format!("│ Content-Length: {}\n", length_str));
            }
        }

        if let Some(content_type) = headers.get("content-type") {
            if let Ok(type_str) = content_type.to_str() {
                message.push_str(&format!("│ Content-Type: {}\n", type_str));
            }
        }

        message.push_str("└───────────────────────────────────────────────────────");

        MessageUtil::api(&message);
    }

    pub fn log_response(status_code: u16, request_id: &uuid::Uuid, response_time: Option<u64>) {
        MessageUtil::api(&format!(
            "\n┌─ Response ──────────────────────────────────────────────\n\
         │ Request ID: {}\n\
         │ Status Code: {}\n\
         │ Response Time: {}\n\
         └───────────────────────────────────────────────────────",
            request_id, status_code, response_time.map_or("N/A".to_string(), |time| format!("{} ms", time))
        ));
    }
}