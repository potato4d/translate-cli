package agent

import (
	"context"
	"time"

	"github.com/potato4d/translate-cli/internal/translate"
)

type Adapter interface {
	ID() string
	DisplayName() string
	Detect(ctx context.Context, runtime DetectionRuntime) DetectionResult
	BuildCommand(req translate.TranslationRequest, runtime RuntimeContext) ExecSpec
	ExtractResult(raw ExecResult) (translate.TranslationResult, error)
}

type DetectionRuntime struct {
	ExistingDefault string
	EnvTool         string
}

type DetectionResult struct {
	ID             string
	DisplayName    string
	Path           string
	Found          bool
	Authenticated  bool
	NonInteractive bool
	Score          int
	Status         string
}

type RuntimeContext struct {
	Timeout         time.Duration
	WorkDir         string
	SchemaPath      string
	LastMessagePath string
}

type ExecSpec struct {
	Command  string
	Args     []string
	Stdin    string
	WorkDir  string
	AllowRaw bool
}

type ExecResult struct {
	Stdout          string
	Stderr          string
	LastMessageText string
}

func All() []Adapter {
	return []Adapter{
		CodexAdapter{},
		ClaudeAdapter{},
	}
}

func ByID(id string) (Adapter, bool) {
	for _, adapter := range All() {
		if adapter.ID() == id {
			return adapter, true
		}
	}
	return nil, false
}

func SupportedIDs() []string {
	return []string{"codex", "claude"}
}
