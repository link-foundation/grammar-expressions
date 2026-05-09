#!/usr/bin/env bash

set -euo pipefail

limit=1500
warning=1350
failures=()
warnings=()
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
package_dir="$(cd "$script_dir/.." && pwd)"

while IFS= read -r -d '' file; do
  line_count=$(wc -l < "$file" | tr -d '[:space:]')
  if [ "$line_count" -gt "$limit" ]; then
    echo "::error file=$file::JavaScript file has $line_count lines; limit is $limit."
    failures+=("$file")
  elif [ "$line_count" -gt "$warning" ]; then
    echo "::warning file=$file::JavaScript file has $line_count lines; warning threshold is $warning."
    warnings+=("$file")
  fi
done < <(find "$package_dir" -type f \( -name '*.js' -o -name '*.mjs' -o -name '*.d.ts' \) -not -path '*/node_modules/*' -print0)

if [ "${#warnings[@]}" -gt 0 ]; then
  printf 'JavaScript files approaching the line limit:\n'
  printf '  %s\n' "${warnings[@]}"
fi

if [ "${#failures[@]}" -gt 0 ]; then
  printf 'JavaScript files exceeding the line limit:\n'
  printf '  %s\n' "${failures[@]}"
  exit 1
fi

echo "All JavaScript files are within the $limit line limit."
