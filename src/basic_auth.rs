use rocket::request::{FromRequest, Outcome};
use rocket::http::Status;
use rocket::Request;    

pub struct BasicAuthStruct {
    pub username: String,
    pub password: String,
}

// Basic username:password
impl BasicAuthStruct {
    fn from_header(header: &str) -> Option<BasicAuthStruct>{
        let header_vec = header.split_whitespace().collect::<Vec<&str>>();
        if header_vec.len() != 2 {
            return None;
        }
        if header_vec[0] != "Basic" {
            return None;
        }

        // process header_vec[1]
        Self::from_base64(header_vec[1])
    }

    fn from_base64(base64_string: &str) -> Option<BasicAuthStruct> {
        let decoded = base64::decode(base64_string).ok()?;
        // convert utf8 to string
        let decoded_string = String::from_utf8(decoded).ok()?;
        // split decoded_string by :
        let decoded_vec = decoded_string.split(":").collect::<Vec<&str>>();
        if decoded_vec.len() != 2 {
            return None;
        }
        // assgin decode value to BasicAuthStruct
        Some(BasicAuthStruct {
            username: decoded_vec[0].to_string(),
            password: decoded_vec[1].to_string(),
        })
    }
}


#[rocket::async_trait]
impl <'r>FromRequest<'r> for BasicAuthStruct {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = request.headers();
        let auth_header = headers.get_one("Authorization");
        if let Some(auth_header) = auth_header {
            if let Some(auth) = BasicAuthStruct::from_header(auth_header) {
                return Outcome::Success(auth);
            }
        }
        return Outcome::Failure((Status::Unauthorized, ()));
    }
}