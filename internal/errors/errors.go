package apperrors

import (
	"fmt"
	"io"
)

type ExitCode int

const (
	CodeOK             ExitCode = 0
	CodeGeneral        ExitCode = 1
	CodeUsage          ExitCode = 2
	CodeConfig         ExitCode = 3
	CodeToolNotFound   ExitCode = 4
	CodeAgentExecution ExitCode = 5
	CodeTimeout        ExitCode = 6
)

type Error struct {
	Code    ExitCode
	Message string
	Hint    string
}

func New(code ExitCode, format string, args ...any) *Error {
	return &Error{Code: code, Message: fmt.Sprintf(format, args...)}
}

func WithHint(err *Error, hint string) *Error {
	err.Hint = hint
	return err
}

func (e *Error) Error() string {
	if e == nil {
		return ""
	}
	return e.Message
}

func Render(err error, stderr io.Writer) int {
	if err == nil {
		return int(CodeOK)
	}

	if appErr, ok := err.(*Error); ok {
		fmt.Fprintf(stderr, "error: %s\n", appErr.Message)
		if appErr.Hint != "" {
			fmt.Fprintf(stderr, "hint: %s\n", appErr.Hint)
		}
		return int(appErr.Code)
	}

	fmt.Fprintf(stderr, "error: %s\n", err)
	return int(CodeGeneral)
}
