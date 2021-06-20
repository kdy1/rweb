use rweb::{form::Valid, get, validator::Validate, Json, Schema};
use serde::Deserialize;

#[derive(Debug, Schema, Validate, Deserialize)]
struct RegistrationForm {
    #[validate(email)]
    email: String,
}

#[get("/")]
fn index(_: Json<Valid<RegistrationForm>>) -> String {
    String::new()
}
