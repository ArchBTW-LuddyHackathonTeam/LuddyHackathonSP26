# Demo Scripts

## Go Stress Test

Usage:

```bash
# Defaults: 20 workers, 20 seconds
go run stress.go -base http://localhost:3000

# Heavy usage
go run stress.go -base http://localhost:3000 -concurrency 100 -duration 60s

# Compile first for max speed
go build -o stress stress.go
./stress -base http://localhost:3000 -concurrency 200 -duration 120s
```
