use crate::identity::Claims;

pub struct Route {
    pub target: String,
}

pub fn get_route(first_chunk: &[u8], claims: &Claims, default_target: &str) -> Result<String, String> {
    let request_str = String::from_utf8_lossy(first_chunk);
    let path = request_str.lines().next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    // RBAC Check
    if path.starts_with("/vault") && claims.role != "admin" {
        return Err("Forbidden: Admins Only".to_string());
    }

    // Dynamic Routing Match
    let target = match path {
        p if p.starts_with("/api") => "127.0.0.1:8080".to_string(),
        p if p.starts_with("/vault") => "127.0.0.1:9090".to_string(),
        _ => default_target.to_string(),
    };

    Ok(target)
}