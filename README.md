# Leaderboard API

# Build command

```bash
docker-compose up --build
```

# Stop command

```
docker-compose down
```

# Test

```bash
curl http://localhost:3000
```

## Database Connection

From inside the API container:
```
postgresql://leaderboard:hackathon@postgres:5432/leaderboard
```

From local:
```
postgresql://leaderboard:hackathon@localhost:5432/leaderboard
```
