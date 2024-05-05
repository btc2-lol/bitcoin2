CREATE TABLE accounts(
  id BIGSERIAL PRIMARY KEY, 
  address BYTEA UNIQUE CHECK (octet_length(address) = 20),
  balance BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE blocks(
  number BIGSERIAL PRIMARY KEY,
  hash BYTEA UNIQUE CHECK (octet_length(hash) = 32),
  hash_state BYTEA,
  timestamp TIMESTAMP
);

CREATE TABLE transactions(
  id BIGSERIAL PRIMARY KEY, 
  account_id BIGINT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
  nonce BIGINT NOT NULL,
  gas_price BIGINT NOT NULL,
  _to BYTEA UNIQUE CHECK (octet_length(_to) = 20),
  value BIGINT NOT NULL,
  input BYTEA, 
  signature BYTEA, 
  block_number BIGINT REFERENCES blocks(number) ON DELETE RESTRICT
);

CREATE TABLE entries(
  id BIGSERIAL PRIMARY KEY, 
  transaction_id BIGINT NOT NULL REFERENCES transactions(id) ON DELETE RESTRICT,
  amount BIGINT NOT NULL CHECK (amount > 0), 
  creditor_id BIGINT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT, 
  debtor_id BIGINT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT
);

CREATE TABLE spent_legacy_outputs(
  id BIGSERIAL PRIMARY KEY, 
  transaction_id BIGINT NOT NULL REFERENCES transactions(id) ON DELETE RESTRICT,
  hash BYTEA UNIQUE CHECK (octet_length(hash) = 32),
  index SMALLINT,
  UNIQUE(hash, index)
);


CREATE 
OR REPLACE FUNCTION validate_entry() RETURNS TRIGGER AS $$ BEGIN
    IF (SELECT balance FROM accounts WHERE id = NEW.creditor_id) < NEW.amount THEN RAISE EXCEPTION 'Insufficient funds in the debtor_id account.';
END IF;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER validate_before_insert BEFORE INSERT ON entries FOR EACH ROW EXECUTE FUNCTION validate_entry();

CREATE OR REPLACE FUNCTION update_account_balances() RETURNS TRIGGER AS $$ BEGIN 
UPDATE accounts SET balance = balance + NEW.amount 
WHERE 
  accounts.id = NEW.debtor_id;
UPDATE accounts SET balance = balance - NEW.amount 
WHERE 
  accounts.id = NEW.creditor_id;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER update_balances_after_insert 
AFTER 
  INSERT ON entries FOR EACH ROW EXECUTE FUNCTION update_account_balances();

CREATE OR REPLACE PROCEDURE transfer(transaction_id BIGINT, creditor_account BYTEA, debtor_account BYTEA, transfer_amount BIGINT)
LANGUAGE plpgsql
AS $$
DECLARE
    creditor_id BIGINT;
    debtor_id BIGINT;
BEGIN
    SELECT id INTO creditor_id FROM accounts WHERE address = creditor_account;

    IF NOT FOUND THEN
        INSERT INTO accounts (address, balance) VALUES (creditor_account, 0) RETURNING id INTO creditor_id;
    END IF;

    SELECT id INTO debtor_id FROM accounts WHERE address = debtor_account;

    IF NOT FOUND THEN
        INSERT INTO accounts (address, balance) VALUES (debtor_account, 0) RETURNING id INTO debtor_id;
    END IF;


    INSERT INTO entries (transaction_id, creditor_id, debtor_id, amount)
    VALUES (transaction_id, creditor_id, debtor_id, transfer_amount);
END;
$$;

CREATE OR REPLACE FUNCTION select_or_insert_account(new_address BYTEA)
RETURNS BIGINT
LANGUAGE plpgsql
AS $$
DECLARE
    account_id BIGINT;
BEGIN
    SELECT id INTO account_id FROM accounts WHERE address = new_address;

    IF account_id IS NULL THEN
        INSERT INTO accounts (address) VALUES (new_address) RETURNING id INTO account_id;
    END IF;

    RETURN account_id;
END;
$$;
