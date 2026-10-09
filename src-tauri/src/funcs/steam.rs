pub async fn get_steamgriddb_cover(
    client: &reqwest::Client,
    app_id: Option<i64>,
    title: Option<&str>,
) -> Option<String> {
    let api_key = std::env::var("STEAM_GRID_DB").ok()?;
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return None;
    }

    let extract_grid_url = |json: &serde_json::Value| -> Option<String> {
        let data = json.get("data")?.as_array()?;
        let first_grid = data.first()?;
        first_grid
            .get("url")
            .or_else(|| first_grid.get("thumb"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string())
    };

    // 1. Recherche par app_id Steam sur SteamGridDB
    if let Some(id) = app_id {
        if id > 0 {
            let url_600x900 = format!(
                "https://www.steamgriddb.com/api/v2/grids/steam/{}?dimensions=600x900",
                id
            );
            if let Ok(resp) = client.get(&url_600x900).bearer_auth(api_key).send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(cover_url) = extract_grid_url(&json) {
                        return Some(cover_url);
                    }
                }
            }

            // Fallback sans restriction de dimension si pas de 600x900 strict
            let url_any = format!("https://www.steamgriddb.com/api/v2/grids/steam/{}", id);
            if let Ok(resp) = client.get(&url_any).bearer_auth(api_key).send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(cover_url) = extract_grid_url(&json) {
                        return Some(cover_url);
                    }
                }
            }
        }
    }

    // 2. Recherche par titre du jeu sur SteamGridDB
    if let Some(game_title) = title {
        let clean_title = game_title.replace(['™', '®', '©'], "").trim().to_string();

        if !clean_title.is_empty() {
            let encoded_title = urlencoding::encode(&clean_title);
            let search_url = format!(
                "https://www.steamgriddb.com/api/v2/search/autocomplete/{}",
                encoded_title
            );

            if let Ok(resp) = client.get(&search_url).bearer_auth(api_key).send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                        if let Some(first_game) = data.first() {
                            if let Some(sgdb_id) = first_game.get("id").and_then(|i| i.as_i64()) {
                                let grid_url_600x900 = format!(
                                    "https://www.steamgriddb.com/api/v2/grids/game/{}?dimensions=600x900",
                                    sgdb_id
                                );
                                if let Ok(grid_resp) = client
                                    .get(&grid_url_600x900)
                                    .bearer_auth(api_key)
                                    .send()
                                    .await
                                {
                                    if let Ok(grid_json) =
                                        grid_resp.json::<serde_json::Value>().await
                                    {
                                        if let Some(cover_url) = extract_grid_url(&grid_json) {
                                            return Some(cover_url);
                                        }
                                    }
                                }

                                let grid_url_any = format!(
                                    "https://www.steamgriddb.com/api/v2/grids/game/{}",
                                    sgdb_id
                                );
                                if let Ok(grid_resp) =
                                    client.get(&grid_url_any).bearer_auth(api_key).send().await
                                {
                                    if let Ok(grid_json) =
                                        grid_resp.json::<serde_json::Value>().await
                                    {
                                        if let Some(cover_url) = extract_grid_url(&grid_json) {
                                            return Some(cover_url);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
