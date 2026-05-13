package agent

import (
	"context"
	"os/exec"
	"time"
)

func DetectAll(ctx context.Context, runtime DetectionRuntime) []DetectionResult {
	adapters := All()
	results := make([]DetectionResult, 0, len(adapters))
	for _, adapter := range adapters {
		results = append(results, adapter.Detect(ctx, runtime))
	}
	return results
}

func Recommended(results []DetectionResult) (DetectionResult, bool) {
	var best DetectionResult
	for _, result := range results {
		if !result.Found {
			continue
		}
		if !best.Found || result.Score > best.Score {
			best = result
		}
	}
	return best, best.Found
}

func lookPath(id string) (string, bool) {
	path, err := exec.LookPath(id)
	return path, err == nil
}

func detectContext(parent context.Context) (context.Context, context.CancelFunc) {
	return context.WithTimeout(parent, 3*time.Second)
}

func score(id string, found bool, authenticated bool, nonInteractive bool, runtime DetectionRuntime) int {
	total := 0
	if found {
		total += 50
	}
	if authenticated {
		total += 30
	}
	if nonInteractive {
		total += 20
	}
	if runtime.ExistingDefault == id {
		total += 100
	}
	if runtime.EnvTool == id {
		total += 100
	}
	return total
}
