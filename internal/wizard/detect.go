package wizard

import "github.com/potato4d/translate-cli/internal/agent"

func foundTools(results []agent.DetectionResult) []agent.DetectionResult {
	var found []agent.DetectionResult
	for _, result := range results {
		if result.Found {
			found = append(found, result)
		}
	}
	return found
}
