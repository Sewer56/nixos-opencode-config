package main

import (
	"fmt"
	"os"
)

// main only wires repo discovery to command dispatch. Real behavior lives in
// small files by responsibility: config, rewrite logic, model discovery, CLI,
// and TUI.
func main() {
	env, err := findEnv()
	if err != nil {
		fatal(err)
	}
	if err := runCLI(env, os.Args[1:]); err != nil {
		fatal(err)
	}
}

func fatal(err error) {
	fmt.Fprintf(os.Stderr, "error: %v\n", err)
	os.Exit(1)
}
