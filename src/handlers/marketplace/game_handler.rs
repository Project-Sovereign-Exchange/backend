use actix_web::{get, web, Responder, Result, HttpResponse};
use crate::app_state::AppState;

/*
#[get("/{name}")]
pub async fn get_game(
    web::Path(name): web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder> {
    let game = data
        .db
        .get_game_by_name(&name)
        .await
        .map_err(|e| {
            error!("Failed to get game: {}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    if let Some(game) = game {
        Ok(HttpResponse::Ok().json(game))
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

 */