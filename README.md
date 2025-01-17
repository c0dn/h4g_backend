# Backend Server Setup Guide

This is a Rust-based API server built with:
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Diesel](https://diesel.rs/) - ORM with PostgreSQL
- [Redis](https://redis.io/) - Temporary storage, password reset
- [PASETO](https://paseto.io/) - For secure token-based authentication
- [Casbin](https://casbin.org/) - For role-based access control

## Prerequisites

1. **Rust Toolchain (Windows)**
   ```bash
   # Download and run rustup-init.exe from:
   # https://rustup.rs/
   
   # After installation, open a new terminal and install Diesel CLI
   cargo install diesel_cli --no-default-features --features postgres
   ```

   Note: The PostgreSQL client library (libpq) is required to compile the program:
   - Windows: Install it by downloading PostgreSQL from https://www.postgresql.org/download/windows/ (client tools included in installation)
   - Linux: Install the libpq development package:
     ```bash
     # Debian/Ubuntu
     sudo apt install libpq-dev
     
     # Fedora
     sudo dnf install libpq-devel
     
     # Arch Linux
     sudo pacman -S postgresql-libs
     ```

2. **Docker Desktop for Windows**
   - Download and install Docker Desktop from: https://www.docker.com/products/docker-desktop
   - Start Docker Desktop
   - Ensure WSL 2 is installed and enabled (Follow docker Desktop instructions)

## Database and Redis Setup (Using Docker)

1. **Start PostgreSQL Container**
   ```bash
   # Replace YOUR_USERNAME and YOUR_SECURE_PASSWORD with your chosen credentials
   # Example: mydbuser and a strong password like xj2k9#mP$q5v
   docker run --name postgres_server \
     -e POSTGRES_USER=YOUR_USERNAME \
     -e POSTGRES_PASSWORD=YOUR_SECURE_PASSWORD \
     -e POSTGRES_DB=hack4good \
     -p 5432:5432 \
     -d postgres:17.2
   ```

   Note: If you use a username other than "testuser", you'll need to update the username in `migrations/2025-01-11-060923_init/up.sql` to match your chosen username.

2. **Start Redis Container**
   ```bash
   docker run --name redis-stack-server -p 6379:6379 -d redis/redis-stack-server:latest
   ```

3. **Initialize PostgreSQL Database**
   ```bash
   # Replace YOUR_USERNAME with the username you used above
   docker exec -it postgres_server psql -U YOUR_USERNAME -d hack4good -c "GRANT ALL ON SCHEMA public TO YOUR_USERNAME;"
   ```

Note: If you prefer not to use Docker, manual installation instructions for Windows are provided at the end of this README.

## Environment Setup

1. Create a `.env` file in the project root:
   ```env
   # Database (Docker setup)
   # Replace YOUR_USERNAME and YOUR_SECURE_PASSWORD with the credentials you used in the docker run command
   DATABASE_URL=postgres://YOUR_USERNAME:YOUR_SECURE_PASSWORD@localhost/hack4good
   
   # Redis (Docker setup)
   REDIS_URL=redis://127.0.0.1:6379
   
   # Server (If changing from default localhost):
   HOST=127.0.0.1:3000
   ```


## Project Setup

1. **Clone and Build**
   ```bash
   # Clone the repository
   git clone <repository-url>
   cd <project-directory>
   
   # Install dependencies and build
   cargo build
   ```

2. **Run Database Migrations**
   ```bash
   diesel migration run
   ```

3. **Start the Server**
   ```bash
   # Development mode
   cargo run
   
   # Production mode
   cargo run --release
   ```


## API Documentation

API endpoints are documented in the `api/welfare_home` directory using Bruno collections. Install [Bruno](https://www.usebruno.com/) to interact with the API endpoints.

## Role-Based Access Control

The system uses Casbin for role-based access control:
- Configuration file: `casbin.conf`
- Policy definitions: `role_policy.csv`

## Common Issues

1. **Docker Issues**
   - Ensure Docker Desktop is running
   - Check container status with `docker ps -a`
   - View logs with `docker logs <container_name>`
   - Ensure ports 5432 and 6379 are not in use by other services

2. **Database Connection Issues**
   - Verify Docker containers are running
   - Check credentials in `.env` match Docker container setup
   - Ensure PostgreSQL container is healthy

3. **Migration Issues**
   - Run `diesel migration revert` to undo last migration
   - Check `diesel.toml` configuration
   - Ensure database URL is correct

## Manual Installation (Windows Alternative)

If you prefer not to use Docker:

1. **PostgreSQL**
   - Download installer from: https://www.postgresql.org/download/windows/
   - Run installer and follow setup wizard
   - Add PostgreSQL bin directory to PATH
   - Start PostgreSQL service through Services app

2. **Redis**
   - Download Redis for Windows from: https://github.com/microsoftarchive/redis/releases
   - Run installer
   - Start Redis service through Services app
