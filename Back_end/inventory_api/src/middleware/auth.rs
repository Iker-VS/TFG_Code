use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{
    body::{BoxBody, MessageBody},
    Error, HttpMessage,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::{
    env,
    task::Context,
    time::{SystemTime, UNIX_EPOCH},
};

/// Middleware de autenticación.
pub struct AuthMiddleware;

#[derive(Debug, serde::Deserialize, serde::Serialize,Clone)]
pub struct Claims {
    pub sub: String,
    exp: usize,
    pub role: String,
}

// Se elimina el bound `Clone` ya que no clonaremos el servicio.
impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService { service })
    }
}

/// Servicio que envuelve el inner service y ejecuta la autenticación.
pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extrae el token del encabezado.
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|hv| hv.to_str().ok())
            .map(|t| t.trim_start_matches("Bearer ").to_string());

        let secret = env::var("API_KEY").unwrap_or_else(|_| "clave_secreta".to_string());

        let auth_result: Result<ServiceRequest, Error> = if let Some(token) = token {
            match decode::<Claims>(
                &token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &Validation::default(),
            ) {
                Ok(data) => {
                    let mut claims = data.claims;
                    claims.sub = claims.sub
                        .strip_prefix("ObjectId(")
                        .and_then(|s| s.strip_suffix(")"))
                        .unwrap_or(&claims.sub)
                        .to_string();

                    if claims.sub.is_empty() {
                        Err(actix_web::error::ErrorBadRequest("ID inválido"))
                    } else {
                        // Inserta el ID en las extensiones de la request.
                        req.extensions_mut().insert(claims);
                        Ok(req)
                    }
                }
                Err(_) => Err(actix_web::error::ErrorUnauthorized(
                    "Token inválido o expirado",
                )),
            }
        } else {
            Err(actix_web::error::ErrorUnauthorized("Token ausente"))
        };

        match auth_result {
            Ok(req) => {
                let fut = self.service.call(req);
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res.map_into_boxed_body())
                })
            }
            Err(err) => Box::pin(async move { Err(err) }),
        }
    }
}

/// Función para generar un token a partir de un usuario.
pub fn generate_token(user_id: String, role:String ) -> String {
    let clave = env::var("API_KEY").unwrap_or_else(|_| "clave_secreta".into());
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + 3600;

    let claims = Claims { sub: user_id, exp, role };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(clave.as_ref()),
    )
    .unwrap()
}
// pub fn decode_token(token: &str)->Result<Claims,Error>{
//     let clave = env::var("API_KEY").unwrap_or_else(|_| "clave_secreta".into());
//     decode::<Claims>(token, &DecodingKey::from_secret(clave.as_ref()), &Validation::default())
//         .map(|data| data.claims)
//         .map_err(|_| actix_web::error::ErrorUnauthorized("Token inválido o expirado"))

// }