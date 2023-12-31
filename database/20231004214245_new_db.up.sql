-- users relation
CREATE TABLE users (
	id bigserial NOT NULL UNIQUE,
	name varchar(256) NOT NULL,
	email varchar(256) NOT NULL UNIQUE,
	password varchar(256) NOT NULL,
	created_at timestamptz NOT NULL DEFAULT now(),
	updated_at timestamptz NOT NULL DEFAULT now(),
	CONSTRAINT users_pkey PRIMARY KEY (id)
);
CREATE INDEX user_email_index ON users USING hash (email);
CREATE INDEX user_id_index ON users USING hash (id);

-- session relation used for authentication and session specific data
-- TODO replace char(64) with uuid and fetch the key after creating via
-- the RETURNING clause
CREATE TABLE sessions (
	key char(64) NOT NULL UNIQUE,
	state jsonb NOT NULL,
	CONSTRAINT sessions_pkey PRIMARY KEY (key)
);
-- the key attribute is queried everytime a user logs in or makes a request
-- that is protected by the identity middleware
-- therefore at least having an hash index on it might be useful
CREATE INDEX session_key_index ON sessions USING hash (key);

-- todos relation
CREATE TABLE todos (
	id bigserial NOT NULL UNIQUE,
	title varchar(255) NOT NULL,
	description text NOT NULL DEFAULT '',
	is_done bool NOT NULL DEFAULT false,
	created_at timestamptz NOT NULL DEFAULT now(),
	updated_at timestamptz NOT NULL DEFAULT now(),
	owner bigint NOT NULL,
	CONSTRAINT todos_pkey PRIMARY KEY (id)
);
ALTER TABLE todos ADD CONSTRAINT todos_owner_fkey FOREIGN KEY (owner) REFERENCES users(id);