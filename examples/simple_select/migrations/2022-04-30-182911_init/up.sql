CREATE TABLE pupil (
  id    SERIAL PRIMARY KEY,
  name  TEXT NOT NULL
);

CREATE TABLE teacher (
  id    SERIAL PRIMARY KEY,
  name  TEXT NOT NULL
);

INSERT INTO pupil (id, name)
VALUES
  (1, 'Robert Redrust');

INSERT INTO teacher (id, name)
VALUES
  (1, 'Rebecca Rustwood');