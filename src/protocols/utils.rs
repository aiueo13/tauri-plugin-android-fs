use tauri::http;


pub fn resolve_allow_header(allow: impl IntoIterator<Item = http::Method>) -> String {
    let mut result = String::new();
    for method in allow {
        if !result.is_empty() {
            result.push_str(", ");
        }
        result.push_str(method.as_str());
    }
    result
}