
use httparse::Request;
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims{
    pub sub: String,
    pub exp:usize,
    pub role:String,
}

pub fn is_authorized(buffer:&[u8],key:&DecodingKey) -> std::option::Option<Claims>{
    // let secret_token = env::var("IAP_SECRET_TOKEN").unwrap_or_else(|_| "BLOCKED".to_string());
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

        //We are finding the authorization header is present or not
        let auth_header=req.headers.iter().find(|h| {
            h.name.eq_ignore_ascii_case("authorization")
        });

        //We are decoding auth_header and checking if it is Bearer token or not
        if let Some(header)=auth_header{{
            let header_str=String::from_utf8_lossy(header.value);
            if header_str.starts_with("Bearer "){
                let token=&header_str[7..];


                let validation=Validation::default();
                return jsonwebtoken::decode::<Claims>(token,key,&validation)
                    .ok()
                    .map(|data| data.claims);
            }
        }}
    }
    None
}