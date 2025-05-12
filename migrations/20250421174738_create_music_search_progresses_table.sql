CREATE TABLE
  music_search_progresses (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY COMMENT 'ID',
    music_genre_id INT UNSIGNED NOT NULL COMMENT 'music_genresテーブルのID',
    position INT UNSIGNED NOT NULL COMMENT '検索の現在の位置',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '作成日時',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新日時',
    CONSTRAINT fk_music_search_progresses_music_genres FOREIGN KEY (music_genre_id) REFERENCES music_genres (id),
    UNIQUE KEY uk_music_search_progresses_music_genre_id (music_genre_id)
  ) COMMENT '音楽検索の進捗';
