-- Your SQL goes here
CREATE TABLE cache_metadata (
    id INT AUTO_INCREMENT PRIMARY KEY,
    file_path VARCHAR(255) NOT NULL,
    total_countries INT NOT NULL,
    top_countries_json TEXT NOT NULL,
    last_refreshed_at DATETIME NOT NULL
);