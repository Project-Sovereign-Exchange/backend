

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const PASTEL_BLUE: &str = "\x1b[38;5;117m";
const PASTEL_ORANGE: &str = "\x1b[38;5;214m";

pub struct MessageUtil;

impl MessageUtil {
    pub fn error(message: &str) {
        print_message("Error", message, RED);
    }
    pub fn success(message: &str) {
        print_message("Success", message, GREEN);
    }
    pub fn info(message: &str) {
        print_message("Info", message, PASTEL_BLUE);
    }
    pub fn api(message: &str) {
        print_message("API", message, PASTEL_ORANGE);
    }
}

fn print_message(label: &str, message: &str, color: &str) {
    println!("{}{}[{}]{}: {}", color, BOLD, label, RESET, message);
}