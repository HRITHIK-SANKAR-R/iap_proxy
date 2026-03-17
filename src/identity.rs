use httparse::Request;
use std::env;

pub fn is_authorized(buffer:&[u8]) -> bool{
    let secret_token = env::var("IAP_SECRET_TOKEN").unwrap_or_else(|_| "BLOCKED".to_string());
    let mut headers=[httparse::EMPTY_HEADER;64];
    let mut req=Request::new(&mut headers);

    if let Ok(_) = req.parse(buffer) {
        // for header in req.headers{
        //     if header.name.to_lowercase()=="authorization"{
        //         if header.value==secret_token.as_bytes(){
        //             return true;
        //         }
        //     }
        // } // Memory expensive as it allocates heap memory for usage of "for" as well as "to_lowercase()" function
        return req.headers.iter().any(|header| {
            header.name.eq_ignore_ascii_case("authorization") &&
            header.value == secret_token.as_bytes()
        });
    }
    false
}