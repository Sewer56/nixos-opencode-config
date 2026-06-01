package main

import "testing"

func TestTakeBoolFlagAllowsFlagBeforeOrAfterPositionals(t *testing.T) {
	value, positional, err := takeBoolFlag([]string{"normal", "--dry-run"}, "dry-run")
	if err != nil {
		t.Fatal(err)
	}
	if !value || len(positional) != 1 || positional[0] != "normal" {
		t.Fatalf("after positional: value=%v positional=%v", value, positional)
	}

	value, positional, err = takeBoolFlag([]string{"--dry-run", "normal"}, "dry-run")
	if err != nil {
		t.Fatal(err)
	}
	if !value || len(positional) != 1 || positional[0] != "normal" {
		t.Fatalf("before positional: value=%v positional=%v", value, positional)
	}
}

func TestTakeBoolFlagRejectsUnknownFlag(t *testing.T) {
	if _, _, err := takeBoolFlag([]string{"normal", "--nope"}, "dry-run"); err == nil {
		t.Fatalf("unknown flag accepted")
	}
}
