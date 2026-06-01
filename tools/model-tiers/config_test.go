package main

import "testing"

func TestValidateWorkRequiresWorkProvider(t *testing.T) {
	good := TierSet{
		tierLow:  "sewer-axonhub-work/gpt-5.4-mini",
		tierMed:  "sewer-axonhub-work/gpt-5.4",
		tierHigh: "sewer-axonhub-work/gpt-5.5",
	}
	if err := validateWork(good); err != nil {
		t.Fatalf("good work config rejected: %v", err)
	}

	bad := cloneTierSet(good)
	bad[tierMed] = "sewer-axonhub/MiniMax-M3"
	if err := validateWork(bad); err == nil {
		t.Fatalf("bad work config accepted")
	}
}

func TestMarshalConfigKeepsTierOrder(t *testing.T) {
	data, err := marshalConfig(Config{
		"work": TierSet{
			tierHigh: "sewer-axonhub-work/high",
			tierLow:  "sewer-axonhub-work/low",
			tierMed:  "sewer-axonhub-work/med",
		},
	})
	if err != nil {
		t.Fatal(err)
	}
	want := `{
  "work": {
    "LOW": "sewer-axonhub-work/low",
    "MED": "sewer-axonhub-work/med",
    "HIGH": "sewer-axonhub-work/high"
  }
}
`
	if string(data) != want {
		t.Fatalf("marshal mismatch\nwant:\n%s\ngot:\n%s", want, string(data))
	}
}
