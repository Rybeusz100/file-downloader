use actix_web::{dev::ServiceRequest, error::Error, web, HttpMessage};
use actix_web_httpauth::extractors::{
    bearer::{self, BearerAuth},
    AuthenticationError,
};
use jwt::VerifyWithKey;

use crate::{AppState, TokenClaims};

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();
    let data = req.app_data::<web::Data<AppState>>().unwrap();
    let claims: Result<TokenClaims, _> = token.verify_with_key(&data.jwt_secret);

    match claims {
        Ok(val) => {
            req.extensions_mut().insert(val);
            Ok(req)
        }
        Err(_) => {
            let config = req
                .app_data::<bearer::Config>()
                .cloned()
                .unwrap_or_default()
                .scope("");

            Err((AuthenticationError::from(config).into(), req))
        }
    }
}
