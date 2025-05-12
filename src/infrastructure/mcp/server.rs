use anyhow::Result;
use rmcp::{
    Error as McpError, ServerHandler,
    model::{CallToolResult, Content, ErrorCode},
    tool,
    transport::SseServer,
};
use schemars::JsonSchema;
use serde::Deserialize;
use sqlx::MySqlPool;

use crate::{
    client::spotify,
    constant::music_search,
    model::{
        excluded_artist::{ExcludedArtist, InsertInput},
        music_genre::MusicGenre,
        music_search_progress::{self, MusicSearchProgress},
    },
};

pub struct Server;

impl Server {
    const BIND_ADDRESS: &str = "127.0.0.1:8000";

    pub async fn start(db_pool: MySqlPool) -> Result<()> {
        SseServer::serve(Self::BIND_ADDRESS.parse()?)
            .await?
            .with_service(move || ArtistSearch::new(db_pool.clone()));

        Ok(())
    }
}

#[derive(Deserialize, JsonSchema)]
struct SearchQuery {
    #[schemars(description = "ジャンル")]
    genre: String,
    #[schemars(description = "現在の検索位置", with = "i64")]
    position: u32,
}

#[derive(Deserialize, JsonSchema)]
struct IsFollowingQuery {
    #[schemars(description = "アーティストIDの配列")]
    ids: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
struct PlayQuery {
    #[schemars(description = "URI")]
    context_uri: String,
}

#[derive(Deserialize, JsonSchema)]
struct FollowQuery {
    #[schemars(description = "アーティストID")]
    ids: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
struct GetExcludedArtistsByIdsQuery {
    #[schemars(description = "アーティストIDの配列")]
    ids: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
struct InsertExcludedArtistQuery {
    #[schemars(description = "アーティストID")]
    id: String,
    #[schemars(description = "アーティスト名")]
    name: String,
}

#[derive(Deserialize, JsonSchema)]
struct MusicSearchProgressQuery {
    #[schemars(description = "音楽ジャンルID", with = "i64")]
    music_genre_id: u32,
}

#[derive(Deserialize, JsonSchema)]
struct InsertMusicSearchProgressQuery {
    #[schemars(description = "音楽ジャンルID", with = "i64")]
    music_genre_id: u32,
}

#[derive(Deserialize, JsonSchema)]
struct UpdateMusicSearchProgressQuery {
    #[schemars(description = "音楽ジャンルID", with = "i64")]
    music_genre_id: u32,
}

#[derive(Clone)]
struct ArtistSearch {
    db_pool: MySqlPool,
}

#[tool(tool_box)]
impl ArtistSearch {
    pub fn new(db_pool: MySqlPool) -> Self {
        Self { db_pool }
    }

    #[tool(description = "アーティストを検索します")]
    async fn search(
        &self,
        #[tool(aggr)] SearchQuery { genre, position }: SearchQuery,
    ) -> Result<CallToolResult, McpError> {
        let access_token = match spotify::api::token::post().await {
            Ok(response) => response.access_token,
            Err(e) => {
                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("アクセストークンの取得に失敗しました,{}", e),
                    None,
                ));
            }
        };
        let query = spotify::v1::search::artist::GetQuery {
            offset: Some(position),
            limit: Some(music_search::FETCH_LIMIT),
            genre: Some(genre.clone()),
        };
        let artists = match spotify::v1::search::artist::get(&access_token, &query).await {
            Ok(response) => response.artists.items,
            Err(e) => {
                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("アーティストの検索に失敗しました,{}", e),
                    None,
                ));
            }
        };
        let output = if artists.is_empty() {
            format!(
                "ジャンル '{}' に一致するアーティストが見つかりませんでした。",
                genre
            )
        } else {
            let mut output = format!("ジャンル '{}' の検索結果:\n\n", genre);
            for artist in artists {
                output.push_str(&format!(
                    "アーティストID: {}\nアーティスト名: {}\nジャンル: {}\nURI: {}\n",
                    artist.id,
                    artist.name,
                    artist.genres.join(","),
                    artist.uri
                ));
            }

            output
        };

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "アーティストをフォローしているか判定します")]
    async fn is_following(
        &self,
        #[tool(aggr)] IsFollowingQuery { ids }: IsFollowingQuery,
    ) -> Result<CallToolResult, McpError> {
        let access_token = match spotify::api::token::post().await {
            Ok(response) => response.access_token,
            Err(e) => {
                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("アクセストークンの取得に失敗しました,{}", e),
                    None,
                ));
            }
        };
        let response = spotify::v1::me::following::contains::get(
            &access_token,
            spotify::v1::me::following::contains::Type::Artist,
            &ids,
        )
        .await
        .unwrap();
        let mut output = format!("アーティストのフォロー状況:\n");
        for item in response {
            output.push_str(&format!(
                "{}\n",
                match item {
                    true => "フォロー済み",
                    false => "未フォロー",
                }
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "曲を再生します")]
    async fn play(
        &self,
        #[tool(aggr)] PlayQuery { context_uri }: PlayQuery,
    ) -> Result<CallToolResult, McpError> {
        let access_token = match spotify::api::token::post().await {
            Ok(response) => response.access_token,
            Err(e) => {
                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("アクセストークンの取得に失敗しました,{}", e),
                    None,
                ));
            }
        };
        let response = spotify::v1::me::player::play::put(&access_token, &context_uri).await;
        let output = match response {
            Ok(_) => "曲を再生しました",
            Err(_) => "曲の再生に失敗しました",
        };

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "アーティストをフォローします")]
    async fn follow(
        &self,
        #[tool(aggr)] FollowQuery { ids }: FollowQuery,
    ) -> Result<CallToolResult, McpError> {
        let access_token = match spotify::api::token::post().await {
            Ok(response) => response.access_token,
            Err(e) => {
                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("アクセストークンの取得に失敗しました,{}", e),
                    None,
                ));
            }
        };
        let response = spotify::v1::me::following::put(
            &access_token,
            spotify::v1::me::following::PutType::Artist,
            &ids,
        )
        .await;
        let output = match response {
            Ok(_) => "アーティストをフォローしました",
            Err(_) => "アーティストのフォローに失敗しました",
        };

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "除外されているアーティストのリストを取得します")]
    async fn get_excluded_artists_by_ids(
        &self,
        #[tool(aggr)] GetExcludedArtistsByIdsQuery { ids }: GetExcludedArtistsByIdsQuery,
    ) -> Result<CallToolResult, McpError> {
        let excluded_artists = ExcludedArtist::find_by_ids(&self.db_pool, &ids).await;
        let mut output = format!("除外されているアーティスト:\n");
        match excluded_artists {
            Ok(excluded_artists) => {
                for excluded_artist in &excluded_artists {
                    output.push_str(&format!(
                        "アーティストID: {}\nアーティスト名: {}\n",
                        excluded_artist.id, excluded_artist.name
                    ));
                }
            }
            Err(e) => {
                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("除外されているアーティストの取得に失敗しました ,{}", e),
                    None,
                ));
            }
        };

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "アーティストを除外リストに登録します")]
    async fn insert_excluded_artist(
        &self,
        #[tool(aggr)] InsertExcludedArtistQuery { id, name }: InsertExcludedArtistQuery,
    ) -> Result<CallToolResult, McpError> {
        let input = InsertInput::new(id, name);
        let excluded_artist = ExcludedArtist::insert(&self.db_pool, &input).await;
        let output;
        match excluded_artist {
            Ok(_) => {
                output = "アーティストを除外リストに登録しました";
            }
            Err(e) => {
                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("アーティストを除外リストに登録するのに失敗しました ,{}", e),
                    None,
                ));
            }
        };

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "音楽ジャンル一覧を取得します")]
    async fn get_music_genres(&self) -> Result<CallToolResult, McpError> {
        let genres = MusicGenre::find_all(&self.db_pool).await;
        let mut output = String::from("音楽ジャンル:\n");
        match genres {
            Ok(genres) => {
                for genre in genres {
                    output.push_str(&format!(
                        "ID: {}\nジャンル名: {}\n検索キー: {}\n\n",
                        genre.id, genre.name, genre.search_key
                    ));
                }
            }
            Err(e) => {
                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("音楽ジャンル一覧の取得に失敗しました,{}", e),
                    None,
                ));
            }
        }

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "音楽検索の進捗を取得します")]
    async fn get_music_search_progress(
        &self,
        #[tool(aggr)] MusicSearchProgressQuery { music_genre_id }: MusicSearchProgressQuery,
    ) -> Result<CallToolResult, McpError> {
        let progress =
            MusicSearchProgress::find_by_music_genre_id(&self.db_pool, music_genre_id).await;
        match progress {
            Ok(progress) => {
                let output = match progress {
                    Some(progress) => {
                        format!(
                            "音楽ジャンルID: {}\n現在の検索位置: {}",
                            progress.music_genre_id, progress.position,
                        )
                    }
                    None => "音楽検索の進捗が見つかりません".to_string(),
                };

                Ok(CallToolResult::success(vec![Content::text(output)]))
            }
            Err(e) => Err(McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("音楽検索の進捗の取得に失敗しました,{}", e),
                None,
            )),
        }
    }

    #[tool(description = "音楽検索の進捗を登録します")]
    async fn insert_music_search_progress(
        &self,
        #[tool(aggr)]
        InsertMusicSearchProgressQuery { music_genre_id }: InsertMusicSearchProgressQuery,
    ) -> Result<CallToolResult, McpError> {
        let input = music_search_progress::UpsertInput::new(music_search::INITIAL_POSITION);
        let progress = MusicSearchProgress::upsert(&self.db_pool, music_genre_id, &input).await;
        match progress {
            Ok(progress) => {
                let output = format!(
                    "音楽検索の進捗を登録しました\n音楽ジャンルID: {}\n現在の検索位置: {}",
                    progress.music_genre_id, progress.position,
                );

                Ok(CallToolResult::success(vec![Content::text(output)]))
            }
            Err(e) => Err(McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("音楽検索の進捗の登録に失敗しました,{}", e),
                None,
            )),
        }
    }

    #[tool(description = "音楽検索の進捗を更新します")]
    async fn update_music_search_progress(
        &self,
        #[tool(aggr)]
        UpdateMusicSearchProgressQuery { music_genre_id }: UpdateMusicSearchProgressQuery,
    ) -> Result<CallToolResult, McpError> {
        let music_search_progress =
            MusicSearchProgress::find_by_music_genre_id(&self.db_pool, music_genre_id).await;
        match music_search_progress {
            Ok(Some(progress)) => {
                let input = music_search_progress::UpsertInput::new(
                    progress.position + music_search::FETCH_LIMIT,
                );
                let progress =
                    MusicSearchProgress::upsert(&self.db_pool, music_genre_id, &input).await;
                match progress {
                    Ok(progress) => {
                        let output = format!(
                            "音楽検索の進捗を更新しました\n音楽ジャンルID: {}\n現在の検索位置: {}",
                            progress.music_genre_id, progress.position,
                        );

                        return Ok(CallToolResult::success(vec![Content::text(output)]));
                    }
                    Err(e) => {
                        return Err(McpError::new(
                            ErrorCode::INTERNAL_ERROR,
                            format!("音楽検索の進捗の更新に失敗しました,{}", e),
                            None,
                        ));
                    }
                }
            }
            Ok(None) => {
                return Ok(CallToolResult::success(vec![Content::text(
                    "音楽検索の進捗が見つかりません",
                )]));
            }
            Err(e) => {
                return Err(McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("音楽検索の進捗の取得に失敗しました,{}", e),
                    None,
                ));
            }
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for ArtistSearch {}
