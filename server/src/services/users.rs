use actix_web::{get, post, web, HttpResponse, Responder};
use actix_web_httpauth::extractors::basic::BasicAuth;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use jwt::SignWithKey;
use log::error;
use rand::rngs::OsRng;

use crate::{
    db::{check_user_name_free, get_user_with_name, insert_new_user},
    AppState, CreateUserQuery, TokenClaims,
};

#[post("/create_user")]
async fn create_user(
    state: web::Data<AppState>,
    input: web::Json<CreateUserQuery>,
) -> impl Responder {
    let input: CreateUserQuery = input.into_inner();

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(input.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    match check_user_name_free(state.db_conn.clone(), &input.name) {
        Ok(false) => return format!("User with name {} already exists", input.name),
        Err(why) => {
            error!("{}", why);
            return "Error creating the user".to_owned();
        }
        Ok(true) => (),
    }

    match insert_new_user(state.db_conn.clone(), &input.name, &password_hash) {
        Ok(id) => id.to_string(),
        Err(why) => {
            error!("{}", why);
            "Error creating the user".to_owned()
        }
    }
}

#[get("/auth")]
async fn auth(state: web::Data<AppState>, credentials: BasicAuth) -> impl Responder {
    let username = credentials.user_id();
    let password = credentials.password();

    match password {
        None => HttpResponse::Unauthorized().json("Incorrect username or password"),
        Some(pass) => match get_user_with_name(state.db_conn.clone(), username) {
            Err(why) => {
                error!("{}", why);
                HttpResponse::InternalServerError().finish()
            }
            Ok(user_opt) => match user_opt {
                None => HttpResponse::Unauthorized().json("Incorrect username or password"),
                Some(user) => {
                    let parsed_hash = PasswordHash::new(&user.password).unwrap();
                    let argon2 = Argon2::default();
                    if argon2
                        .verify_password(pass.as_bytes(), &parsed_hash)
                        .is_ok()
                    {
                        let claims = TokenClaims { id: user.id };
                        let token = claims.sign_with_key(&state.jwt_secret).unwrap();
                        HttpResponse::Ok().json(token)
                    } else {
                        HttpResponse::Unauthorized().json("Incorrect username or password")
                    }
                }
            },
        },
    }
}
