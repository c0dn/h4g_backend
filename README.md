# Dev environment setup

## PostgreSQL Setup Commands

1. Create a new user:
```sql
CREATE USER newuser WITH PASSWORD 'password';
```

2. Create the database:
```sql
CREATE DATABASE hack4good;
```

3. Grant privileges:
```sql
GRANT ALL PRIVILEGES ON DATABASE hack4good TO newuser;
\c hack4good
GRANT ALL ON SCHEMA public TO newuser;
```

## Quick Setup

To set up the database in one go, run these commands in PostgreSQL terminal:
```sql
CREATE USER newuser WITH PASSWORD 'password';
CREATE DATABASE hack4good;
GRANT ALL PRIVILEGES ON DATABASE hack4good TO newuser;
\c hack4good
GRANT ALL ON SCHEMA public TO newuser;
```

## Environment Setup

1. Create a `.env` file in the root directory
2. Add the following database connection string:
DATABASE_URL=postgres://testuser:password@localhost/hack4good

Note: 
- Replace 'password' with a secure password of your choice
- Replace 'testuser' with your PostgreSQL username if different