CREATE TABLE
  excluded_artists (
    id VARCHAR(255) NOT NULL PRIMARY KEY COMMENT 'SpotifyのアーティストID',
    name VARCHAR(255) NOT NULL COMMENT 'アーティスト名',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '作成日時'
  ) COMMENT '除外されているアーティスト';
