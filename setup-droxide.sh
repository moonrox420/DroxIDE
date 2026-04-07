#!/bin/bash
# DroxIDE Setup Script
# This script sets up the complete development environment for DroxIDE
# including all dependencies, build tools, and configuration

set -e

# Colors for output
RED='\\033[0;31m'
GREEN='\\033[0;32m'
YELLOW='\\033[1;33m'
BLUE='\\033[0;34m'
NC='\\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   print_error "This script should not be run as root. Please run as regular user."
   exit 1
fi

# Detect OS
OS=$(uname -s)
case "$OS" in
   Linux*)
      OS_TYPE="Linux"
      ;;
   Darwin*)
      OS_TYPE="macOS"
      ;;
   *)
      OS_TYPE="Unknown"
      ;;
esac

print_status "Detected OS: $OS_TYPE"

# Check if we're in the right directory
if [[! -f "CMakeLists.txt" ]]; then
    print_error "This script must be run from the DroxIDE project root directory"
    exit 1
fi

print_status "Starting DroxIDE setup..."

# 1. Install system dependencies
print_status "Installing system dependencies..."

case "$OS_TYPE" in
    "Linux")
        # Detect package manager
        if command -v apt-get &> /dev/null; then
            print_status "Using apt package manager"
            sudo apt-get update
            sudo apt-get install -y \
                build-essential \
                cmake \
                git \
                curl \
                wget \
                python3 \
                python3-pip \
                libssl-dev \
                pkg-config \
                libclang-dev \
                llvm \
                clang \
                libxcb-xinerama0 \
                libxcb-icccm4 \
                libxcb-image0 \
                libxcb-keysyms1 \
                libxcb-render-util0 \
                libxcb-xkb1 \
                libxkbcommon-x11-0 \
                libxkbcommon0
        elif command -v dnf &> /dev/null; then
            print_status "Using dnf package manager"
            sudo dnf install -y \
                make \
                cmake \
                gcc \
                gcc-c++ \
                git \
                curl \
                wget \
                python3 \
                python3-pip \
                openssl-devel \
                pkgconf-pkg-config \
                clang \
                llvm \
                libxcb-devel \
                libxcb-xinerama-devel \
                libxcb-icccm-devel \
                libxcb-image-devel \
                libxcb-keysyms-devel \
                libxcb-render-util-devel \
                libxcb-xkb-devel \
                libxkbcommon-x11-devel \
                libxkbcommon-devel
        else
            print_error "Unsupported package manager. Please install dependencies manually."
            exit 1
        fi
        ;;
    "macOS")
        if! command -v brew &> /dev/null; then
            print_status "Installing Homebrew..."
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        fi
        print_status "Installing dependencies via Homebrew..."
        brew install \
            cmake \
            git \
            llvm \
            clang \
            pkg-config \
            openssl \
            python3
        ;;
    *)
        print_error "Unsupported OS. Please install dependencies manually."
        exit 1
        ;;
esac

# 2. Install Rust
print_status "Installing Rust..."
if! command -v rustc &> /dev/null; then
    print_status "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
else
    print_status "Rust already installed"
fi

# 3. Install vcpkg (for Windows) or system Qt (for Linux/macOS)
if [[ "$OS_TYPE" == "Linux" || "$OS_TYPE" == "macOS" ]]; then
    print_status "Installing Qt6..."
    if [[ "$OS_TYPE" == "Linux" ]]; then
        # Install Qt6 via package manager
        if command -v apt-get &> /dev/null; then
            sudo apt-get install -y \
                qt6-base-dev \
                qt6-tools-dev \
                qt6-qt5compat-dev
        elif command -v dnf &> /dev/null; then
            sudo dnf install -y \
                qt6-qtbase-devel \
                qt6-qttools-devel \
                qt6-qt5compat-devel
        fi
    else
        # macOS - install Qt6 via Homebrew
        brew install qt6
    fi
else
    print_error "Windows setup requires