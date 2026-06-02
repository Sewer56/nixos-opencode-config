package main

import "testing"

func TestValidateWorkRequiresWorkProvider(t *testing.T) {
	tierOrder := []string{"LOW", "MED", "HIGH"}
	good := TierSet{
		"LOW":  "sewer-axonhub-work/gpt-5.4-mini",
		"MED":  "sewer-axonhub-work/gpt-5.4",
		"HIGH": "sewer-axonhub-work/gpt-5.5",
	}
	if err := validateWork(good, tierOrder); err != nil {
		t.Fatalf("good work config rejected: %v", err)
	}

	bad := cloneTierSet(good, tierOrder)
	bad["MED"] = "sewer-axonhub/MiniMax-M3"
	if err := validateWork(bad, tierOrder); err == nil {
		t.Fatalf("bad work config accepted")
	}
}

func TestMarshalConfigKeepsTierOrder(t *testing.T) {
	tierOrder := []string{"LOW", "MED", "HIGH"}
	data, err := marshalConfig(Config{
		"work": TierSet{
			"HIGH": "sewer-axonhub-work/high",
			"LOW":  "sewer-axonhub-work/low",
			"MED":  "sewer-axonhub-work/med",
		},
	}, tierOrder)
	if err != nil {
		t.Fatal(err)
	}
	want := `{
  "$tierOrder": {"0": "LOW","1": "MED","2": "HIGH"},
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
