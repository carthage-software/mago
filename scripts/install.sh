#!/usr/bin/env bash

set -euo pipefail

REPO="carthage-software/mago"
BIN_NAME="mago"
TMP_DIR=$(mktemp -d)
NEW_ISSUE="https://github.com/carthage-software/mago/issues/new"
SIGNER_WORKFLOW=".github/workflows/cd.yml"
INSTALL_DIR=""
VERSION=""
VERIFY_MODE="auto"
DO_VERIFY=0

function separator() {
    echo
    echo -e "\033[39m======================================================================\033[0m"
    echo
}

function cleanup() {
    rm -rf "$TMP_DIR";
}

trap cleanup EXIT

function red() { echo -e "\033[31m$1\033[0m"; }
function green() { echo -e "\033[32m$1\033[0m"; }
function yellow() { echo -e "\033[33m$1\033[0m"; }
function blue() { echo -e "\033[34m$1\033[0m"; }
function fail() {
  red "$1"
  exit 1
}

blue "Welcome to the Mago Installer!"
blue "This script will download and install Mago for your system."
echo
yellow "If you encounter any issues, please open a GitHub issue at ${NEW_ISSUE}."

for arg in "$@"; do
  case $arg in
    --install-dir=*)
      INSTALL_DIR="${arg#*=}"
      ;;
    --version=*)
      VERSION="${arg#*=}"
      ;;
    --always-verify)
      if [ "$VERIFY_MODE" = "never" ]; then
        fail "Cannot combine --always-verify with --no-verify."
      fi
      VERIFY_MODE="always"
      ;;
    --no-verify)
      if [ "$VERIFY_MODE" = "always" ]; then
        fail "Cannot combine --always-verify with --no-verify."
      fi
      VERIFY_MODE="never"
      ;;
    *)
      fail "Unknown argument: $arg"
      ;;
  esac
done

gh_supports_attestation() {
  command -v gh > /dev/null && gh attestation --help > /dev/null 2>&1
}

case "$VERIFY_MODE" in
  always)
    separator

    if ! command -v gh > /dev/null; then
      red "--always-verify requires the GitHub CLI ('gh'), but it was not found on PATH."
      red "Install it from https://cli.github.com/ and re-run."
      fail "Aborting: refusing to install without attestation verification."
    fi

    if ! gh attestation --help > /dev/null 2>&1; then
      red "--always-verify requires a 'gh' version that ships the 'gh attestation' subcommand."
      red "Your installed version is too old."
      red "Upgrade the GitHub CLI (https://cli.github.com/) and re-run."
      fail "Aborting: refusing to install without attestation verification."
    fi

    DO_VERIFY=1
    green "Attestation verification: ON (--always-verify). Using: $(command -v gh)"
    ;;

  never)
    separator
    yellow "Attestation verification: OFF (--no-verify)."
    yellow "The downloaded archive will be installed without checking its build attestation."
    ;;

  auto)
    if gh_supports_attestation; then
      DO_VERIFY=1
      separator
      green "Attestation verification: ON (auto). Using: $(command -v gh)"
    else
      separator
      yellow "Attestation verification: OFF."
      if command -v gh > /dev/null; then
        yellow "Found 'gh' but it does not support 'gh attestation' (version too old)."
        yellow "Upgrade the GitHub CLI (https://cli.github.com/) to enable verification,"
        yellow "or pass --always-verify to make this a hard requirement."
      else
        yellow "GitHub CLI ('gh') was not found on PATH. Install it from"
        yellow "https://cli.github.com/ to enable verification, or pass"
        yellow "--always-verify to make this a hard requirement."
      fi
    fi
    ;;
esac

separator

# Get the system's target triple
green "Detecting your system configuration..."
arch=$(uname -m)
os=$(uname -s | tr '[:upper:]' '[:lower:]')

case "$arch" in
  x86_64)
    arch="x86_64"
    ;;
  amd64)
    arch="x86_64"
    ;;
  arm64 | aarch64)
    arch="aarch64"
    ;;
  armv7l)
    arch="armv7"
    ;;
  i386 | i486 | i586 | i686)
    arch="i686"
    ;;
  ppc)
    arch="powerpc"
    ;;
  ppc64)
    arch="powerpc64"
    ;;
  ppc64le)
    arch="powerpc64le"
    ;;
  s390x)
    arch="s390x"
    ;;
  *)
    red "Unsupported architecture: ${arch}. Please open an issue on GitHub at ${NEW_ISSUE}."
    exit 1
    ;;
esac

case "$os" in
  darwin)
    vendor="apple"
    os_suffix=""
    ;;
  linux)
    vendor="unknown"
    os_suffix=""
    if command -v ldd > /dev/null; then
      ldd_version=$(ldd --version 2>&1)
      if echo "$ldd_version" | grep -q "musl"; then
        case "$arch" in
          x86_64 | aarch64 | i686)
            os_suffix="musl"
            ;;
          arm | armv7)
            if grep -q "hard" /proc/cpuinfo 2> /dev/null; then
              os_suffix="musleabihf"
            else
              os_suffix="musleabi"
            fi
            ;;
          *)
            fail "Unsupported architecture for musl: ${arch}"
            ;;
        esac
      else
        case "$arch" in
          x86_64 | aarch64 | i686 | powerpc | powerpc64 | powerpc64le | s390x)
            os_suffix="gnu"
            ;;
          arm | armv7)
            if grep -q "hard" /proc/cpuinfo 2> /dev/null; then
              os_suffix="gnueabihf"
            else
              os_suffix="gnueabi"
            fi
            ;;
          *)
            fail "Unsupported architecture for glibc: ${arch}"
            ;;
        esac
      fi
    else
      os_suffix="musl"
    fi
    ;;
  freebsd)
    vendor="unknown"
    os_suffix=""
    ;;
  *)
    fail "Unsupported operating system: ${os}. Please open an issue on GitHub at ${NEW_ISSUE}."
    ;;
esac

# If the os_suffix is empty, we use `{arch}-{vendor}-{os}` as the target triple
if [ -z "$os_suffix" ]; then
  target_triple="${arch}-${vendor}-${os}"
else
  target_triple="${arch}-${vendor}-${os}-${os_suffix}"
fi

green "Detected target: ${target_triple}"

separator

# Determine installation directory
binary_dir=""
if [ -n "$INSTALL_DIR" ]; then
  binary_dir="$INSTALL_DIR"
  if [ ! -d "$binary_dir" ]; then
    fail "The provided installation directory does not exist: $binary_dir"
  elif [ ! -w "$binary_dir" ]; then
    fail "The provided installation directory is not writable: $binary_dir"
  fi
else
    possible_dirs=("/usr/local/bin" "/usr/bin" "/bin")
    for dir in "${possible_dirs[@]}"; do
      if [ ! -d "$dir" ]; then
        yellow "The directory $dir does not exist. Trying the next directory..."

        continue
      fi

      if [ ! -w "$dir" ]; then
        yellow "The directory $dir is not writable. Trying the next directory..."

        continue
      fi

      binary_dir="$dir"
      break
    done

    if [ -z "$binary_dir" ]; then
      yellow "No suitable installation directory found. Using the current directory instead."
      binary_dir=$(pwd)
      echo
    fi
fi

green "Binary will be installed to: $binary_dir"

separator

# Determine which version to install
if [ -n "$VERSION" ]; then
  # Use the specified version
  latest_tag="$VERSION"
  green "Installing specified version: ${latest_tag}"
else
  # Fetch the latest release tag
  green "Fetching the latest release of Mago..."
  if command -v curl > /dev/null; then
    response=$(curl -s -f "https://api.github.com/repos/${REPO}/releases/latest") || {
      red "Failed to fetch the latest release. Please check your internet connection or try again later."
      fail "Open an issue on GitHub at ${NEW_ISSUE} if the issue persists."
    }
  elif command -v wget > /dev/null; then
    response=$(wget -q -O - "https://api.github.com/repos/${REPO}/releases/latest") || {
      red "Failed to fetch the latest release. Please check your internet connection or try again later."
      fail "Open an issue on GitHub at ${NEW_ISSUE} if the issue persists."
    }
  else
    fail "Neither 'curl' nor 'wget' are installed. Please install one of these tools to proceed."
  fi

  latest_tag=$(echo "$response" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)
  if [ -z "$latest_tag" ]; then
    red "Failed to extract the latest release tag from the GitHub API response."
    fail "Please open an issue on GitHub at ${NEW_ISSUE}."
  fi
  green "Latest release: ${latest_tag}"
fi

separator

# Download the release
file_name="${BIN_NAME}-${latest_tag}-${target_triple}"
file_url="https://github.com/${REPO}/releases/download/${latest_tag}/${file_name}.tar.gz"
destination="${TMP_DIR}/${file_name}"

green "Downloading ${file_name}..."
if command -v curl > /dev/null; then
  curl_output=$(curl -fL "$file_url" -o "${destination}.tar.gz" 2>&1) || {
    if echo "$curl_output" | grep -q "404"; then
      red "The requested asset (${file_name}.tar.gz) does not exist."
      fail "Open an issue on GitHub at ${NEW_ISSUE} with the detected target: ${target_triple}."
    else
      red "Failed to download the binary. Please check your internet connection."
      fail "Try again later or open an issue on GitHub at ${NEW_ISSUE}."
    fi
  }
elif command -v wget > /dev/null; then
  wget -q --show-progress "$file_url" -O "${destination}.tar.gz" || {
    red "Failed to download the binary. Please check your internet connection."
    fail "Try again later or open an issue on GitHub at ${NEW_ISSUE}."
  }
else
  fail "Neither 'curl' nor 'wget' are installed. Please install one of these tools to proceed."
fi

green "Download complete!"

separator

if [ "$DO_VERIFY" -eq 1 ]; then
  green "Verifying attestation for ${file_name}.tar.gz..."
  green "  Repository:      ${REPO}"
  green "  Signer workflow: ${SIGNER_WORKFLOW}"
  if ! gh attestation verify "${destination}.tar.gz" \
       --repo "$REPO" \
       --signer-workflow "${REPO}/${SIGNER_WORKFLOW}"; then
    quarantine="${PWD}/${file_name}.unverified.tar.gz"
    cp "${destination}.tar.gz" "$quarantine" || quarantine=""

    red "ATTESTATION VERIFICATION FAILED for ${file_name}.tar.gz."
    red "The downloaded archive could not be matched to a build attestation"
    red "signed by ${REPO}/${SIGNER_WORKFLOW}."
    red "Refusing to install."
    if [ -n "$quarantine" ]; then
      red "The archive has been preserved at:"
      red "  ${quarantine}"
      red "Inspect it manually; do not run anything from it."
    fi
    fail "Aborting installation."
  fi

  green "Attestation verified."

  separator
fi

green "Extracting ${file_name}.tar.gz..."
if ! tar -xzf "${destination}.tar.gz" -C "$TMP_DIR"; then
  red "Failed to extract the binary."
  fail "Please open an issue on GitHub at ${NEW_ISSUE}."
fi
green "Extraction complete!"

separator

green "Installing binary to ${binary_dir}..."
if mv "${destination}/${BIN_NAME}" "${binary_dir}/"; then
  chmod +x "${binary_dir}/${BIN_NAME}" || fail "Failed to make the binary executable."
  green "Installation complete!"

  if ! echo "$PATH" | grep -qE "(^|:)$binary_dir($|:)"; then
      echo
      yellow "> Note: The directory ${binary_dir} is not in your PATH."
      yellow "> This means you cannot run '${BIN_NAME}' directly from the terminal."
      echo
      yellow "To add ${binary_dir} to your PATH temporarily, run the following command:"
      echo   "  export PATH=${binary_dir}:\$PATH"
      echo
      yellow "To make this change permanent, add the following line to your shell configuration file (e.g., ~/.bashrc, ~/.zshrc, or ~/.profile):"
      echo   "  export PATH=${binary_dir}:\$PATH"
      echo
      yellow "Alternatively, you can move the binary to a directory already in your PATH, such as /usr/local/bin, by running:"
      echo "  sudo mv ${binary_dir}/${BIN_NAME} /usr/local/bin/"
  fi
else
    red "Failed to move the binary to ${binary_dir}."
    fail "Please open an issue on GitHub at ${NEW_ISSUE}."
fi

exit 0
