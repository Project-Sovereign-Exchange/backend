use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use crate::app_state::AppState;
use crate::entities::{games, sets};

pub struct GameService {
    pub state: AppState,
}

impl GameService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn get_games(&self) -> Result<Vec<String>, String> {
        games::Entity::find()
            .all(&self.state.db)
            .await
            .map_err(|e| format!("Failed to fetch games: {}", e))
            .map(|games| games.into_iter().map(|game| game.name).collect())
    }

    pub async fn get_game_by_name(
        &self,
        game_name: &str,
    ) -> Result<Option<games::Model>, String> {
        if game_name.is_empty() {
            return Err("Game name cannot be empty".to_string());
        }

        games::Entity::find()
            .filter(games::Column::Name.eq(game_name))
            .one(&self.state.db)
            .await
            .map_err(|e| format!("Failed to fetch game '{}': {}", game_name, e))
    }

    pub async fn get_sets_by_game(
        &self,
        game_name: &str,
    ) -> Result<Vec<String>, String> {
        if game_name.is_empty() {
            return Err("Game name cannot be empty".to_string());
        }

        let games = self.get_games().await?;
        if !games.iter().any(|g| g == game_name) {
            return Err(format!("Game '{}' not found", game_name));
        }

        sets::Entity::find()
            .filter(sets::Column::GameName.eq(game_name))
            .all(&self.state.db)
            .await
            .map_err(|e| format!("Failed to fetch sets for game '{}': {}", game_name, e))
            .map(|sets| sets.into_iter().map(|set| set.name).collect())
    }
}