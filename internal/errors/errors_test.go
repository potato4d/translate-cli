package apperrors

import (
	"bytes"
	"strings"
	"testing"
)

func TestRenderAppError(t *testing.T) {
	var stderr bytes.Buffer
	code := Render(WithHint(New(CodeToolNotFound, "missing tool"), "install it"), &stderr)
	if code != int(CodeToolNotFound) {
		t.Fatalf("code = %d", code)
	}
	got := stderr.String()
	if !strings.Contains(got, "error: missing tool") || !strings.Contains(got, "hint: install it") {
		t.Fatalf("stderr = %q", got)
	}
}
