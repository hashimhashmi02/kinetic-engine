#!/usr/bin/env bash
# Kinetic dev setup for macOS and Linux.
# Checks required tools and installs Claude Code plugins at project scope.
set -u
missing=()

check() {
  if command -v "$1" >/dev/null 2>&1; then
    printf '  ok       %s\n' "$1"
  else
    printf '  MISSING  %s  ->  %s\n' "$1" "$2"
    missing+=("$1")
  fi
}

echo 'Tools'
check git    'https://git-scm.com'
check cargo  'https://rustup.rs'
check rustup 'https://rustup.rs'
check ffmpeg 'install with your package manager (needed for MP4 export)'
check node   'https://nodejs.org (needed for hooks and the site)'
check pnpm   'npm install -g pnpm (needed for the site)'
check gh     'https://cli.github.com (needed for CI checks)'

if command -v rustup >/dev/null 2>&1; then
  echo 'Rust components'
  rustup component add rustfmt clippy rust-analyzer >/dev/null 2>&1
  echo '  ok       rustfmt, clippy, rust-analyzer'
fi

echo 'Claude Code plugins (project scope)'
if command -v claude >/dev/null 2>&1; then
  for p in rust-analyzer-lsp typescript-lsp code-review feature-dev commit-commands frontend-design; do
    if claude plugin install "$p@claude-plugins-official" --scope project >/dev/null 2>&1; then
      printf '  ok       %s\n' "$p"
    else
      printf '  FAILED   %s  ->  run /plugin inside claude and install it by hand\n' "$p"
    fi
  done
else
  echo '  MISSING  claude  ->  https://claude.com/claude-code'
  missing+=(claude)
fi

if [ "${#missing[@]}" -gt 0 ]; then
  printf '\nMissing: %s\n' "${missing[*]}"
  exit 1
fi
printf '\nReady. Start with: claude, then /milestone 1\n'
