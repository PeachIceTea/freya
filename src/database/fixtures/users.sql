-- The migrations, so we only create a normal user for some additional tests.
INSERT INTO users (name, password, admin)
VALUES ('user', '$argon2i$v=19$m=4096,t=3,p=1$c2FsdEl0V2l0aFNhbHQ$xTGvQNICqetaNA0Wu1GwFmYhQjAreRcjBz6ornhaFXA', FALSE);
