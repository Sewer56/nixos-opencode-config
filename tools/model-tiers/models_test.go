package main

import (
	"reflect"
	"testing"
)

func TestParseModelsOutputDedupesAndIgnoresNoise(t *testing.T) {
	models, err := parseModelsOutput(`
not-a-model
sewer-axonhub/GLM-5.1
sewer-axonhub/GLM-5.1
sewer-axonhub-work/gpt-5.5
`)
	if err != nil {
		t.Fatal(err)
	}
	want := []string{"sewer-axonhub/GLM-5.1", "sewer-axonhub-work/gpt-5.5"}
	if !reflect.DeepEqual(models, want) {
		t.Fatalf("models = %#v, want %#v", models, want)
	}
}

func TestFilterModelsUsesTokensAndWorkProvider(t *testing.T) {
	models := []string{
		"sewer-axonhub/GLM-5.1",
		"sewer-axonhub/gpt-5.5",
		"sewer-axonhub-work/gpt-5.5",
		"sewer-axonhub-work/gpt-5.4-mini",
	}

	got := filterModels("normal", models, "gpt 5.5")
	want := []string{"sewer-axonhub/gpt-5.5", "sewer-axonhub-work/gpt-5.5"}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("normal filter = %#v, want %#v", got, want)
	}

	got = filterModels("work", models, "gpt")
	want = []string{"sewer-axonhub-work/gpt-5.5", "sewer-axonhub-work/gpt-5.4-mini"}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("work filter = %#v, want %#v", got, want)
	}
}
