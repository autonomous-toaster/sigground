set quiet

# Run all checks (mirrors CI)
[parallel]
ci: check lint test check-file-sizes

# Fast compile check
check:
    #!/usr/bin/env bash
    if output=$(cargo check --workspace --all-targets 2>&1); then
        echo "✓ check passed"
    else
        printf '%s\n' "$output"
        exit 1
    fi

# Build (dev profile)
build:
    #!/usr/bin/env bash
    if output=$(cargo build --workspace 2>&1); then
        echo "✓ build passed"
    else
        printf '%s\n' "$output"
        exit 1
    fi

# Run tests
test:
    #!/usr/bin/env bash
    output=$(cargo test --workspace 2>&1)
    code=$?
    if [ $code -eq 0 ]; then
        printf '%s\n' "$output" | grep -E "^cargo test:" || echo "✓ tests passed"
    else
        printf '%s\n' "$output"
        exit $code
    fi

# Clippy — deny all warnings
lint:
    #!/usr/bin/env bash
    if output=$(cargo clippy --workspace --all-targets -- -Dwarnings 2>&1); then
        echo "✓ lint passed"
    else
        printf '%s\n' "$output"
        exit 1
    fi

# Check format
fmt:
    #!/usr/bin/env bash
    if output=$(cargo fmt --check 2>&1); then
        echo "✓ fmt passed"
    else
        printf '%s\n' "$output"
        echo "→ fix with: cargo fmt"
        exit 1
    fi

# Check file sizes (max 500 lines, 10% tolerance)
check-file-sizes max="500" tolerance="10":
    #!/usr/bin/env bash
    TARGET={{max}}
    TOL={{tolerance}}
    MAX=$(( TARGET + TARGET * TOL / 100 ))
    fail=0
    while IFS= read -r f; do
        lines=$(wc -l < "$f")
        if [ "$lines" -gt "$MAX" ]; then
            echo "FAIL: $f has $lines lines (target $TARGET, hard limit $MAX)"
            fail=1
        fi
    done < <(find src -name '*.rs' | grep -v '/tests/')
    [ $fail -eq 0 ] && echo "✓ all source files within $MAX lines (target $TARGET + ${TOL}% tolerance)"
