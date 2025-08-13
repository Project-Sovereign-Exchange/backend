use actix_web::{get, web, Responder, Result, HttpResponse};
use crate::app_state::AppState;
use crate::services::marketplace::game_service::GameService;

#[get("/{name}")]
pub async fn get_game(
    name: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder> {
    let game_service = GameService::new(state.as_ref().clone());

    match game_service.get_game_by_name(&name).await {
        Ok(game) => {
            if let Some(game) = game {
                Ok(HttpResponse::Ok().json(game))
            } else {
                Ok(HttpResponse::NotFound().body("Game not found"))
            }
        }
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[get("")]
pub async fn get_games(
    state: web::Data<AppState>,
) -> Result<impl Responder> {
    let game_service = GameService::new(state.as_ref().clone());

    match game_service.get_games().await {
        Ok(games) => Ok(HttpResponse::Ok().json(games)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[get("/{name}/sets")]
pub async fn get_sets_by_game(
    name: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder> {
    let game_service = GameService::new(state.as_ref().clone());

    match game_service.get_sets_by_game(&name).await {
        Ok(sets) => Ok(HttpResponse::Ok().json(sets)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

