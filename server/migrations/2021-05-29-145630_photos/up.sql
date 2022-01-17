CREATE TABLE photos (
  datetime timestamp DEFAULT (datetime('now')),
  filename text PRIMARY KEY NOT NULL,
  camera_id text NOT NULL,
  food_weight smallint NOT NULL,
  name text,
  FOREIGN KEY(name) REFERENCES names(name)
);
