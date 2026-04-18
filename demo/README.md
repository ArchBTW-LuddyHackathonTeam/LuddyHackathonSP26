# Demo Scripts

## Go Stress Test

Usage:

```bash
# Realistic: 10x concurrency
go run stress.go -base http://localhost:3000 -concurrency 200 -duration 60s -users 2000

# Conservative: 50x concurrency, near-zero collisions
go run stress.go -base http://localhost:3000 -concurrency 200 -duration 60s -users 10000
```
