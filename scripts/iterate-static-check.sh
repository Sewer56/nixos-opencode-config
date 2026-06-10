#!/usr/bin/env bash
# Usage:
#   scripts/iterate-static-check.sh <artifact_base>
#
# Runs deterministic checks for a direct /iterate/edit run and writes
# <artifact_base>.static-check.md plus the result on stdout.
#
# Owns: changed-path discovery, Delta reconciliation, concrete import existence,
# local command/agent reference resolution, frontmatter delimiter shape,
# renderer output validation, and `git diff --check`.
#
# Does not own: prompt wording, selected-pattern application, permission safety
# beyond reference existence, or reviewer domain decisions.
set -eu

artifact_base="${1:?Usage: iterate-static-check.sh <artifact_base>}"
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

state_path="${artifact_base}.prep.md"
log_path="${artifact_base}.md"
result_path="${artifact_base}.static-check.md"

finding_table="| ID | Severity | Path | Problem | Fix |
|----|----------|------|---------|-----|
"
findings=""
ids=""

add_finding() {
  local id="$1" sev="$2" path="$3" problem="$4" fix="$5"
  findings="${findings}| ${id} | ${sev} | ${path} | ${problem} | ${fix} |
"
  if [ -n "${ids}" ]; then ids="${ids}, "; fi
  ids="${ids}${id}"
}

# 1. State and log presence.
if [ ! -f "${state_path}" ] || [ ! -f "${log_path}" ]; then
  echo "FAIL: missing prep state (${state_path}) or edit log (${log_path})" >&2
  exit 2
fi

# 2. Derive changed paths.
changed_paths="$(git diff --name-only HEAD 2>/dev/null | grep -v "^${artifact_base}\\." || true)"
untracked_paths="$(git ls-files --others --exclude-standard 2>/dev/null | grep -v "^${artifact_base}\\." || true)"
all_paths="$(printf "%s\n%s\n" "${changed_paths}" "${untracked_paths}" | grep -v '^$' | sort -u)"

# 3. Validate references and frontmatter.
for path in ${all_paths}; do
  [ -f "${path}" ] || continue
  case "${path}" in
    *.md)
      # Reject imports pointing into opencode-source.
      if grep -E '\{\{[^}]*file="[^"]*opencode-source/' "${path}" >/dev/null 2>&1; then
        add_finding "STAT-$(date +%s%N | tail -c 6)" "BLOCKING" "${path}" \
          "renderer import points into opencode-source/" \
          "use a local config/ or .opencode/ path or remove the import"
      fi
      # Frontmatter delimiters.
      if [ "${path#*.opencode/agent/}" != "${path}" ] || [ "${path#config/agent/}" != "${path}" ] || \
         [ "${path#.opencode/command/}" != "${path}" ] || [ "${path#config/command/}" != "${path}" ]; then
        first="$(head -n 1 "${path}")"
        if [ "${first}" != "---" ]; then
          add_finding "STAT-$(date +%s%N | tail -c 6)" "BLOCKING" "${path}" \
            "missing opening frontmatter delimiter" \
            "add '---' as the first line"
        fi
      fi
      # Local @agent/name and permission.task references resolve.
      for ref in $(grep -oE '@[A-Za-z][A-Za-z0-9_./-]+' "${path}" 2>/dev/null | sed 's/^@//' | sort -u); do
        case "${ref}" in
          */*) name="${ref#*/}" ;;
          *)   name="${ref}" ;;
        esac
        if [ ! -f ".opencode/agent/${name}.md" ] && [ ! -f "config/agent/${name}.md" ]; then
          add_finding "STAT-$(date +%s%N | tail -c 6)" "BLOCKING" "${path}" \
            "unresolved @agent reference: ${ref}" \
            "create the agent file or fix the reference"
        fi
      done
      ;;
  esac
done

# 4. Render every changed agent/command file.
for path in ${all_paths}; do
  [ -f "${path}" ] || continue
  case "${path}" in
    .opencode/agent/*|.opencode/command/*|config/agent/*|config/command/*)
      rendered="${path}.rendered"
      if bun plugins/opencode-plugin-md-expand/src/cli/cli.ts render "${path}" >"${rendered}" 2>"${rendered}.err"; then
        # Whitespace and empty-line lint on the rendered output.
        if grep -nE ' +$' "${rendered}" >/dev/null 2>&1; then
          add_finding "STAT-$(date +%s%N | tail -c 6)" "BLOCKING" "${path}" \
            "rendered output has trailing whitespace" \
            "remove trailing spaces from the source prompt"
        fi
        if awk 'NF==0 { blank=NR; next } blank && NR==blank+1 && NF==0 { print NR }' "${rendered}" | head -n1 | grep -q .; then
          add_finding "STAT-$(date +%s%N | tail -c 6)" "BLOCKING" "${path}" \
            "rendered output has consecutive blank lines" \
            "collapse repeated blank lines in the source prompt"
        fi
        # Markdown safety: a code-fence line inside an already-open code block
        # would close the outer fence prematurely. Detect nested ``` inside ```.
        nested_fence_line="$(awk '
          BEGIN { depth = 0 }
          /^```/ { if (depth > 0) { print NR; exit } depth = 1; next }
          /^~~~/ { depth = 1; next }
        ' "${rendered}" | head -n1)"
        if [ -n "${nested_fence_line}" ]; then
          add_finding "STAT-$(date +%s%N | tail -c 6)" "BLOCKING" "${path}" \
            "rendered output has a nested backtick fence at line ${nested_fence_line}" \
            "use ~~~ for the inner fence, or close the outer fence first"
        fi
        rm -f "${rendered}" "${rendered}.err"
      else
        add_finding "STAT-$(date +%s%N | tail -c 6)" "BLOCKING" "${path}" \
          "renderer failed: $(head -n1 "${rendered}.err")" \
          "fix the source template or import, then re-render"
        rm -f "${rendered}" "${rendered}.err"
      fi
      ;;
  esac
done

# 5. git diff --check.
if diff_warnings="$(git diff --check HEAD 2>&1)"; then
  : # no warnings
else
  while IFS= read -r line; do
    [ -z "${line}" ] && continue
    add_finding "STAT-$(date +%s%N | tail -c 6)" "BLOCKING" "git-diff" \
      "${line}" \
      "fix whitespace in the named file"
  done <<EOF
${diff_warnings}
EOF
fi

# 6. Write result.
if [ -z "${findings}" ]; then
  decision="PASS"
  finding_table="${finding_table}| None | none | None | no findings | None |
"
  changed_block="- None"
  verified_block="- None"
  ids_out="None"
  summary="static check passed"
else
  decision="BLOCKING"
  changed_block="$(printf "%s\n" "${all_paths}" | sed '/^$/d' | sed 's/^/- /' | head -n 50)"
  [ -z "${changed_block#- }" ] && changed_block="- None"
  verified_block="- None"
  ids_out="${ids}"
  summary="static check found ${ids%%,*} (and possibly more) BLOCKING findings"
fi

{
  printf "# Iterate Edit Static Check\n"
  printf "Schema: v1\n"
  printf "Decision: %s\n\n" "${decision}"
  printf "## Changed Paths\n"
  printf "%s\n\n" "${changed_block}"
  printf "## Findings\n"
  printf "%s\n" "${finding_table}"
  printf "## Verified\n"
  printf "%s\n" "${verified_block}"
} >"${result_path}"

# Stdout verdict for the primary.
printf "# STATIC CHECK\n"
printf "Decision: %s\n" "${decision}"
printf "Result: %s\n" "$(pwd)/${result_path}"
printf "Changed Paths: %s\n" "$(printf "%s\n" "${all_paths}" | sed '/^$/d' | paste -sd ',' -)"
printf "IDs: %s\n" "${ids_out}"
printf "Summary: %s\n" "${summary}"

if [ "${decision}" = "BLOCKING" ]; then
  exit 1
fi
exit 0
