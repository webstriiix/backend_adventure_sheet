-- Seed test user for frontend development / testing
INSERT INTO users (username, email, password_hash)
VALUES ('test', 'test@gmail.com', '$argon2id$v=19$m=19456,t=2,p=1$RQfUmN7QIQTaTDgeHdGl3g$+U4h7xWscx57ik8EO85ndpmkKJxO0joy8AFeiWEeVxU')
ON CONFLICT (email) DO NOTHING;
