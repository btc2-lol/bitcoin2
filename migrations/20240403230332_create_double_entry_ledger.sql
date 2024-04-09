CREATE TABLE accounts(
  id serial PRIMARY KEY, 
  address BYTEA UNIQUE CHECK (octet_length(address) = 20),
  balance BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE entries(
  id serial PRIMARY KEY, 
  amount BIGINT NOT NULL CHECK (amount > 0), 
  creditor_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT, 
  debtor_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT
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

CREATE OR REPLACE PROCEDURE transfer(creditor_account BYTEA, debtor_account BYTEA, transfer_amount BIGINT)
LANGUAGE plpgsql
AS $$
DECLARE
    creditor_id INTEGER;
    debtor_id INTEGER;
BEGIN
    SELECT id INTO creditor_id FROM accounts WHERE address = creditor_account;

    IF NOT FOUND THEN
        INSERT INTO accounts (address, balance) VALUES (creditor_account, 0) RETURNING id INTO creditor_id;
    END IF;

    SELECT id INTO debtor_id FROM accounts WHERE address = debtor_account;

    IF NOT FOUND THEN
        INSERT INTO accounts (address, balance) VALUES (debtor_account, 0) RETURNING id INTO debtor_id;
    END IF;


    INSERT INTO entries (creditor_id, debtor_id, amount)
    VALUES (creditor_id, debtor_id, transfer_amount);
END;
$$;