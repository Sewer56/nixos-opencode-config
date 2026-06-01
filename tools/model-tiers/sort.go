package main

import "sort"

func sortedCounts(counts map[string]int) []countItem {
	items := make([]countItem, 0, len(counts))
	for model, count := range counts {
		items = append(items, countItem{model, count})
	}
	sort.Slice(items, func(i, j int) bool {
		if items[i].Count == items[j].Count {
			return items[i].Model < items[j].Model
		}
		return items[i].Count > items[j].Count
	})
	return items
}

func sortedProfiles(cfg Config) []string {
	profiles := profileNames(cfg)
	sort.Strings(profiles)
	return profiles
}

func sortedFileKeys(files map[string]int) []string {
	keys := make([]string, 0, len(files))
	for key := range files {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	return keys
}

func contains(values []string, target string) bool {
	for _, value := range values {
		if value == target {
			return true
		}
	}
	return false
}
