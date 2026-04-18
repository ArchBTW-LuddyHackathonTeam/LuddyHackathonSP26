# LibreBoard

![Rust](https://img.shields.io/badge/Rust-2024_Edition-f74c00?style=flat&logo=rust&logoColor=white)
![Axum](https://img.shields.io/badge/Axum-0.8.9-7c3aed?style=flat&logo=rust&logoColor=white)
![Tokio](https://img.shields.io/badge/Tokio-1.52-4f86c6?style=flat&logo=rust&logoColor=white)
![SQLx](https://img.shields.io/badge/SQLx-0.8.6-4479a1?style=flat&logo=rust&logoColor=white)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16-4169e1?style=flat&logo=postgresql&logoColor=white)
![Docker](https://img.shields.io/badge/Docker-Compose-2496ed?style=flat&logo=docker&logoColor=white)
![Swagger](https://img.shields.io/badge/Docs-Swagger_UI-85ea2d?style=flat&logo=swagger&logoColor=black)
![License](https://img.shields.io/badge/License-MIT-green?style=flat)

A blazingly fast, self-hostable leaderboard API for real-time ranking, benchmarking, and performance tracking. Submit scores, track standings, and analyze results through a REST API and live web dashboard.

## Dashboard & Performance

![Web Interface](assets/web_interface.png)
*Real-time leaderboard dashboard*

| Stress Test Summary | Detailed Results |
|--------------------|-----------------|
| ![Stress Test Summary](assets/stress_test_report_summary.png) | ![Detailed Stress Test Results](assets/stress_test_report_detailed.png) |

*Stress testing results*

## About
LibreBoard is an open-source leaderboard platform designed for straightforward self-hosting. With this project, our goal is to offer developers full control over a fast, lightweight, flexible, and user-friendly leaderboard with no unnecessary friction.

The high-efficiency design of LibreBoard is intended to make it useful for a wide range of ranking and performance-tracking scenarios, including traditional competitive events. These include coding competitions, gaming tournaments, classroom challenges, internal benchmarking and algorithm or model evaluation workflows, where fast live updates and performance computations can be necessary.

### Tech Stack
- **Backend:** Rust + Axum
- **Database:** PostgreSQL 16
- **Containerization:** Docker + Docker Compose
- **API Docs:** Swagger UI (via utoipa) — available at `/swagger-ui` when running

## Features
- **Live leaderboard**: top 10 rankings loaded to Markdown table or JSON
- **Score submission**: add or update entries via REST (one active score per participant)
- **Score removal**: remove active entries for a user
- **Statistics**: mean, median, standard deviation, percentile ranks, and more
- **Performance metrics**: rolling average latency per endpoint
- **Score history**: full submission log with datetime and user filtering
- **Live config**: update the leaderboard title and sort order at runtime without restarting
- **One-command deploy**: fully containerized with Docker Compose
- **Interactive API docs**: Swagger UI included out of the box

## Getting Started

### Prerequisites
- [Docker Engine](https://docs.docker.com/engine/install/) (v20.10+ for Compose V2)
- [Docker Compose](https://docs.docker.com/compose/install/)

### 1. Clone the repository
```bash
git clone https://github.com/ArchBTW-LuddyHackathonTeam/LuddyHackathonSP26
cd LuddyHackathonSP26
```

### 2. Start the server

```bash
make run
```

The API is now live at `http://localhost:3000`.  
Swagger UI is available at `http://localhost:3000/swagger-ui`.


### 3. Save your admin secret

On the very first startup, LibreBoard detects that no admin token exists in the database and automatically generates one. It is printed once to the container logs. Make sure to save this immediately:

```
Admin Secret (new): xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

### 4. Using the admin secret

Pass the secret as a Bearer token in the `Authorization` header for any admin-protected endpoint:

```bash
curl -X PATCH http://localhost:3000/admin/config \
  -H "Authorization: Bearer xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" \
  -H "Content-Type: application/json" \
  -d '{"title": "My Leaderboard", "sort_order": "descending"}'
```

You can also input the admin secret through the web interface by navigating to Admin Settings > Administrator Login in the top-right corner of the dashboard.

As of right now, the admin endpoint only supports changing the leaderboard title and ranking sort order. However, this functionality can be expanded for additional security and operational purposes, such as protecting sensitive configuration changes and managing deployment-specific settings through authenticated access.


## API Endpoints

Full interactive documentation with request/response schemas is available at `http://localhost:3000/swagger-ui`.

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| `GET` | `/health` | Health check | — |
| `POST` | `/add` | Submit or replace a participant's score | — |
| `DELETE` | `/remove/{uploader}` | Remove a participant's score | — |
| `GET` | `/leaderboard` | Top-10 scores as a Markdown table | — |
| `GET` | `/leaderboard/json` | Top-10 scores as JSON | — |
| `GET` | `/leaderboard/{num}` | Top-N scores as a Markdown table | — |
| `GET` | `/leaderboard/json/{num}` | Top-N scores as JSON | — |
| `GET` | `/info` | Aggregate score statistics | — |
| `GET` | `/history` | Full score submission history (filterable) | — |
| `GET` | `/history/{user}` | Submission history for a specific user | — |
| `GET` | `/boardconfig` | Read the current leaderboard config | — |
| `GET` | `/performance` | Per-endpoint latency metrics | — |
| `PATCH` | `/admin/config` | Update leaderboard title and sort order | Bearer token |


## Demo

- Open `demo/index.html` in your browser (with the server running) for a live interactive dashboard.
- Run the Go stress-test script in the `/demo` directory to benchmark server performance (see `demo/README.md` for instructions).


## Debugging

### Full reset (wipes database)

```bash
# Modern (Docker Compose V2)
docker compose down -v

# Legacy
docker-compose down -v
```

### Regenerate the admin secret

If you lose the admin secret, generate a new one with:

```bash
make reset-password
```

This clears all existing tokens from the database, prints a fresh secret to stdout, and exits. The server does not need to be restarted. Do keep in mind that resetting the password immediately invalidates the old token. Any client using the old secret will receive `401 Unauthorized`.
