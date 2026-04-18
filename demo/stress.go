// stress_test.go — Luddy Hackathon SP26 API Stress Tester
//
// Usage:
//   go run stress_test.go -base http://localhost:8080
//   go run stress_test.go -base http://localhost:8080 -concurrency 50 -duration 30s
//
// Flags:
//   -base         Base URL of the API (default: http://localhost:8080)
//   -concurrency  Number of parallel workers (default: 20)
//   -duration     How long to run the test (default: 20s)
//   -rampup       Ramp-up period before full concurrency (default: 2s)

package main

import (
	"bytes"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"math"
	"math/rand"
	"net/http"
	"os"
	"sort"
	"strings"
	"sync"
	"sync/atomic"
	"time"
)

// ── Config ────────────────────────────────────────────────────────────────────

var (
	baseURL     = flag.String("base", "http://localhost:8080", "Base URL of the API")
	concurrency = flag.Int("concurrency", 20, "Number of concurrent workers")
	duration    = flag.Duration("duration", 20*time.Second, "Test duration")
	rampup      = flag.Duration("rampup", 2*time.Second, "Ramp-up period")
	users       = flag.Int("users", 2000, "Size of the simulated user pool")
)

// ── Result types ─────────────────────────────────────────────────────────────

type Result struct {
	Endpoint   string
	Method     string
	StatusCode int
	Latency    time.Duration
	Error      string
}

// ── Endpoint definitions ──────────────────────────────────────────────────────

type Endpoint struct {
	Name   string
	Method string
	Path   func() string
	Body   func() interface{}
	Weight int // relative frequency
}

func generateParticipants(n int) []string {
	first := []string{
		"alice", "bob", "charlie", "diana", "eve", "frank", "grace", "heidi",
		"ivan", "judy", "mallory", "niaj", "olivia", "peggy", "rupert", "sybil",
		"trent", "victor", "wendy", "zara", "aaron", "bella", "carlos", "demi",
	}
	last := []string{
		"smith", "jones", "patel", "chen", "kim", "garcia", "müller", "okafor",
		"tanaka", "silva", "brown", "white", "lee", "wang", "martin", "thompson",
	}
	seen := map[string]bool{}
	var out []string
	r := rand.New(rand.NewSource(42))
	for len(out) < n {
		name := fmt.Sprintf("%s_%s_%04d",
			first[r.Intn(len(first))],
			last[r.Intn(len(last))],
			r.Intn(n*10),
		)
		if !seen[name] {
			seen[name] = true
			out = append(out, name)
		}
	}
	return out
}

var participants []string

func randomParticipant() string {
	return participants[rand.Intn(len(participants))]
}

func randomScore() float64 {
	return math.Round(rand.Float64()*10000) / 100
}

func endpoints() []Endpoint {
	return []Endpoint{
		{
			Name:   "POST /add",
			Method: "POST",
			Path:   func() string { return "/add" },
			Body: func() interface{} {
				return map[string]interface{}{
					"key":   randomParticipant(),
					"value": randomScore(),
				}
			},
			Weight: 40,
		},
		{
			Name:   "GET /leaderboard/json",
			Method: "GET",
			Path:   func() string { return "/leaderboard/json" },
			Weight: 25,
		},
		{
			Name:   "GET /leaderboard",
			Method: "GET",
			Path:   func() string { return "/leaderboard" },
			Weight: 10,
		},
		{
			Name:   "GET /leaderboard/json/{num}",
			Method: "GET",
			Path: func() string {
				n := []int{5, 10, 20, 50}[rand.Intn(4)]
				return fmt.Sprintf("/leaderboard/json/%d", n)
			},
			Weight: 8,
		},
		{
			Name:   "GET /info",
			Method: "GET",
			Path:   func() string { return "/info" },
			Weight: 7,
		},
		{
			Name:   "GET /health",
			Method: "GET",
			Path:   func() string { return "/health" },
			Weight: 5,
		},
		{
			Name:   "GET /boardconfig",
			Method: "GET",
			Path:   func() string { return "/boardconfig" },
			Weight: 3,
		},
		{
			Name:   "GET /performance",
			Method: "GET",
			Path:   func() string { return "/performance" },
			Weight: 2,
		},
	}
}

// build a weighted selection table
func buildWeightTable(eps []Endpoint) []int {
	var table []int
	for i, ep := range eps {
		for w := 0; w < ep.Weight; w++ {
			table = append(table, i)
		}
	}
	return table
}

// ── HTTP helper ───────────────────────────────────────────────────────────────

func doRequest(client *http.Client, base string, ep Endpoint) Result {
	url := base + ep.Path()

	var bodyReader io.Reader
	if ep.Body != nil {
		b, _ := json.Marshal(ep.Body())
		bodyReader = bytes.NewReader(b)
	}

	req, err := http.NewRequest(ep.Method, url, bodyReader)
	if err != nil {
		return Result{Endpoint: ep.Name, Method: ep.Method, Error: err.Error()}
	}
	if ep.Body != nil {
		req.Header.Set("Content-Type", "application/json")
	}

	start := time.Now()
	resp, err := client.Do(req)
	latency := time.Since(start)

	if err != nil {
		return Result{Endpoint: ep.Name, Method: ep.Method, Latency: latency, Error: err.Error()}
	}
	io.Copy(io.Discard, resp.Body)
	resp.Body.Close()

	return Result{
		Endpoint:   ep.Name,
		Method:     ep.Method,
		StatusCode: resp.StatusCode,
		Latency:    latency,
	}
}

// ── Stats ─────────────────────────────────────────────────────────────────────

type Stats struct {
	Latencies  []float64 // milliseconds
	StatusCodes map[int]int
	Errors     int
	Count      int
}

func newStats() *Stats {
	return &Stats{StatusCodes: map[int]int{}}
}

func (s *Stats) add(r Result) {
	s.Count++
	if r.Error != "" {
		s.Errors++
		return
	}
	s.StatusCodes[r.StatusCode]++
	s.Latencies = append(s.Latencies, float64(r.Latency.Microseconds())/1000.0)
}

func (s *Stats) percentile(p float64) float64 {
	if len(s.Latencies) == 0 {
		return 0
	}
	sorted := make([]float64, len(s.Latencies))
	copy(sorted, s.Latencies)
	sort.Float64s(sorted)
	idx := int(math.Ceil(p/100.0*float64(len(sorted)))) - 1
	if idx < 0 {
		idx = 0
	}
	return sorted[idx]
}

func (s *Stats) mean() float64 {
	if len(s.Latencies) == 0 {
		return 0
	}
	sum := 0.0
	for _, v := range s.Latencies {
		sum += v
	}
	return sum / float64(len(s.Latencies))
}

func (s *Stats) stddev() float64 {
	if len(s.Latencies) < 2 {
		return 0
	}
	m := s.mean()
	variance := 0.0
	for _, v := range s.Latencies {
		diff := v - m
		variance += diff * diff
	}
	return math.Sqrt(variance / float64(len(s.Latencies)-1))
}

func (s *Stats) successRate() float64 {
	ok := 0
	for code, n := range s.StatusCodes {
		if code >= 200 && code < 400 {
			ok += n
		}
	}
	if s.Count == 0 {
		return 0
	}
	return float64(ok) / float64(s.Count) * 100
}

// ── Main ──────────────────────────────────────────────────────────────────────

func main() {
	flag.Parse()

	eps := endpoints()
	table := buildWeightTable(eps)

	participants = generateParticipants(*users)

	client := &http.Client{
		Timeout: 10 * time.Second,
		Transport: &http.Transport{
			MaxIdleConnsPerHost: *concurrency + 10,
			IdleConnTimeout:     30 * time.Second,
		},
	}

	results := make(chan Result, *concurrency*100)
	var wg sync.WaitGroup
	var totalReqs int64

	deadline := time.Now().Add(*duration)
	rampDeadline := time.Now().Add(*rampup)

	fmt.Printf("\n╔══════════════════════════════════════════════════════════╗\n")
	fmt.Printf("║       Luddy Hackathon SP26 — API Stress Tester           ║\n")
	fmt.Printf("╚══════════════════════════════════════════════════════════╝\n\n")
	fmt.Printf("  Target:      %s\n", *baseURL)
	fmt.Printf("  Concurrency: %d workers\n", *concurrency)
	fmt.Printf("  Duration:    %s\n", *duration)
	fmt.Printf("  Ramp-up:     %s\n\n", *rampup)

	// progress ticker
	ticker := time.NewTicker(1 * time.Second)
	go func() {
		elapsed := 0
		for range ticker.C {
			elapsed++
			n := atomic.LoadInt64(&totalReqs)
			pct := int(float64(elapsed) / duration.Seconds() * 100)
			bar := strings.Repeat("█", pct/5) + strings.Repeat("░", 20-pct/5)
			fmt.Printf("\r  [%s] %3d%% | %6d reqs sent", bar, pct, n)
		}
	}()

	// spawn workers
	for i := 0; i < *concurrency; i++ {
		wg.Add(1)

		// stagger launch during ramp-up
		delay := time.Duration(0)
		if *rampup > 0 {
			delay = time.Duration(int64(*rampup) / int64(*concurrency) * int64(i))
		}

		go func(workerDelay time.Duration) {
			defer wg.Done()
			time.Sleep(workerDelay)

			localRand := rand.New(rand.NewSource(rand.Int63()))
			_ = rampDeadline

			for time.Now().Before(deadline) {
				idx := table[localRand.Intn(len(table))]
				ep := eps[idx]
				r := doRequest(client, *baseURL, ep)
				results <- r
				atomic.AddInt64(&totalReqs, 1)
			}
		}(delay)
	}

	// close results when all workers done
	go func() {
		wg.Wait()
		close(results)
	}()

	// collect
	allStats := map[string]*Stats{}
	global := newStats()

	for r := range results {
		global.add(r)
		if _, ok := allStats[r.Endpoint]; !ok {
			allStats[r.Endpoint] = newStats()
		}
		allStats[r.Endpoint].add(r)
	}

	ticker.Stop()
	fmt.Println()

	totalTime := *duration

	// ── Report ────────────────────────────────────────────────────────────────

	fmt.Printf("\n\n╔══════════════════════════════════════════════════════════════════════╗\n")
	fmt.Printf("║                         STRESS TEST REPORT                          ║\n")
	fmt.Printf("╚══════════════════════════════════════════════════════════════════════╝\n\n")

	fmt.Printf("  %-28s %s\n", "Total Requests:", fmt.Sprintf("%d", global.Count))
	fmt.Printf("  %-28s %.0f req/s\n", "Throughput:", float64(global.Count)/totalTime.Seconds())
	fmt.Printf("  %-28s %.1f%%\n", "Overall Success Rate:", global.successRate())
	fmt.Printf("  %-28s %d\n", "Total Errors:", global.Errors)

	fmt.Printf("\n  ── Global Latency (ms) ───────────────────────────────────────────────\n\n")
	fmt.Printf("  %-12s %-12s %-12s %-12s %-12s %-12s\n",
		"Mean", "StdDev", "p50", "p75", "p95", "p99")
	fmt.Printf("  %-12s %-12s %-12s %-12s %-12s %-12s\n",
		strings.Repeat("─", 10), strings.Repeat("─", 10), strings.Repeat("─", 10),
		strings.Repeat("─", 10), strings.Repeat("─", 10), strings.Repeat("─", 10))
	fmt.Printf("  %-12.2f %-12.2f %-12.2f %-12.2f %-12.2f %-12.2f\n\n",
		global.mean(), global.stddev(),
		global.percentile(50), global.percentile(75),
		global.percentile(95), global.percentile(99))

	fmt.Printf("  ── Status Code Distribution ─────────────────────────────────────────\n\n")
	type kv struct {
		k int
		v int
	}
	var codes []kv
	for k, v := range global.StatusCodes {
		codes = append(codes, kv{k, v})
	}
	sort.Slice(codes, func(i, j int) bool { return codes[i].k < codes[j].k })
	for _, c := range codes {
		pct := float64(c.v) / float64(global.Count) * 100
		bar := strings.Repeat("▓", int(pct/2))
		fmt.Printf("  HTTP %-4d %6d  (%5.1f%%)  %s\n", c.k, c.v, pct, bar)
	}

	fmt.Printf("\n  ── Per-Endpoint Breakdown ────────────────────────────────────────────\n\n")
	fmt.Printf("  %-36s %7s %8s %8s %8s %8s %8s %6s\n",
		"Endpoint", "Reqs", "Mean ms", "p50", "p95", "p99", "Succ%", "Errs")
	fmt.Printf("  %s\n", strings.Repeat("─", 97))

	// sort endpoints by name
	var epNames []string
	for k := range allStats {
		epNames = append(epNames, k)
	}
	sort.Strings(epNames)

	for _, name := range epNames {
		s := allStats[name]
		fmt.Printf("  %-36s %7d %8.2f %8.2f %8.2f %8.2f %6.1f%% %5d\n",
			name, s.Count, s.mean(),
			s.percentile(50), s.percentile(95), s.percentile(99),
			s.successRate(), s.Errors)
	}

	// latency histogram (global)
	fmt.Printf("\n  ── Latency Histogram (ms) ───────────────────────────────────────────\n\n")
	buckets := []struct {
		label string
		lo, hi float64
	}{
		{"  0–5   ms", 0, 5},
		{"  5–10  ms", 5, 10},
		{" 10–25  ms", 10, 25},
		{" 25–50  ms", 25, 50},
		{" 50–100 ms", 50, 100},
		{"100–250 ms", 100, 250},
		{"250–500 ms", 250, 500},
		{"500+    ms", 500, math.MaxFloat64},
	}
	maxBucket := 1
	counts := make([]int, len(buckets))
	for _, lat := range global.Latencies {
		for i, b := range buckets {
			if lat >= b.lo && lat < b.hi {
				counts[i]++
				if counts[i] > maxBucket {
					maxBucket = counts[i]
				}
				break
			}
		}
	}
	for i, b := range buckets {
		bar := strings.Repeat("█", counts[i]*40/maxBucket)
		pct := float64(counts[i]) / float64(len(global.Latencies)) * 100
		fmt.Printf("  %s |%-40s| %6d (%4.1f%%)\n", b.label, bar, counts[i], pct)
	}

	fmt.Printf("\n  ── Recommendations ──────────────────────────────────────────────────\n\n")

	p99 := global.percentile(99)
	p95 := global.percentile(95)
	sr := global.successRate()

	if sr < 99 {
		fmt.Printf("  ⚠  Success rate is %.1f%% — investigate 4xx/5xx responses.\n", sr)
	} else {
		fmt.Printf("  ✓  Success rate looks healthy at %.1f%%.\n", sr)
	}
	if p95 > 200 {
		fmt.Printf("  ⚠  p95 latency (%.0fms) is high — consider caching or indexing.\n", p95)
	} else {
		fmt.Printf("  ✓  p95 latency is %.0fms — within acceptable range.\n", p95)
	}
	if p99 > 500 {
		fmt.Printf("  ⚠  p99 latency (%.0fms) suggests tail latency issues.\n", p99)
	} else {
		fmt.Printf("  ✓  p99 latency is %.0fms.\n", p99)
	}
	if global.Errors > 0 {
		fmt.Printf("  ⚠  %d connection/timeout errors encountered.\n", global.Errors)
	}

	throughput := float64(global.Count) / totalTime.Seconds()
	fmt.Printf("\n  Peak throughput: %.0f req/s across %d workers.\n\n", throughput, *concurrency)

	fmt.Printf("╔══════════════════════════════════════════════════════════════════════╗\n")
	fmt.Printf("║  Done. Re-run with -concurrency and -duration to explore limits.    ║\n")
	fmt.Printf("╚══════════════════════════════════════════════════════════════════════╝\n\n")

	os.Exit(0)
}
