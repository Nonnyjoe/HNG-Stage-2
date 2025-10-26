-- Your SQL goes here
CREATE TABLE countries (
  id INT AUTO_INCREMENT PRIMARY KEY,
  name VARCHAR(191) NULL,
  capital VARCHAR(191) NULL,
  region VARCHAR(191) NULL,
  population BIGINT NULL,
  currency_code VARCHAR(32) NULL,
  exchange_rate DOUBLE NULL,
  estimated_gdp DOUBLE NULL,
  flag_url VARCHAR(255) NULL,
  last_refreshed_at DATETIME NULL,

  -- For upserts (unique business key example)
  UNIQUE KEY uniq_name (name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;