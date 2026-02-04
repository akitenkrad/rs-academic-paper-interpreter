#!/usr/bin/env bash
# install.sh - Install paper-export command to /usr/local/custom_command/
#
# USAGE:
#   ./scripts/install.sh           # Install by copying files (recommended)
#   ./scripts/install.sh --symlink # Install with symlink (for development)
#   ./scripts/install.sh --remove  # Uninstall
#
# DESCRIPTION:
#   Installs the paper-export command and academic-paper-interpreter binary
#   to /usr/local/custom_command/ for system-wide access.

set -euo pipefail

# Configuration
INSTALL_DIR="/usr/local/custom_command"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Check if running with sudo when needed
check_permissions() {
    if [[ ! -d "$INSTALL_DIR" ]]; then
        if ! mkdir -p "$INSTALL_DIR" 2>/dev/null; then
            print_error "Cannot create $INSTALL_DIR. Please run with sudo:"
            echo "  sudo $0 $*"
            exit 1
        fi
    elif [[ ! -w "$INSTALL_DIR" ]]; then
        print_error "Cannot write to $INSTALL_DIR. Please run with sudo:"
        echo "  sudo $0 $*"
        exit 1
    fi
}

# Build release binary if needed
ensure_binary() {
    local binary="$PROJECT_ROOT/target/release/academic-paper-interpreter"

    if [[ ! -x "$binary" ]]; then
        print_info "Release binary not found. Building..."
        if command -v cargo &> /dev/null; then
            (cd "$PROJECT_ROOT" && cargo build --release)
            print_success "Build completed"
        else
            print_error "Cargo not found. Please build manually: cargo build --release"
            exit 1
        fi
    fi

    if [[ ! -x "$binary" ]]; then
        print_error "Binary not found at: $binary"
        exit 1
    fi

    echo "$binary"
}

# Install with symlinks (for development only)
install_symlink() {
    print_info "Installing with symlinks to $INSTALL_DIR"

    check_permissions "$@"

    local binary
    binary=$(ensure_binary)

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Create symlinks (check for existing files)
    if [[ -e "$INSTALL_DIR/academic-paper-interpreter" ]]; then
        print_warning "Overwriting existing: academic-paper-interpreter"
    fi
    ln -sf "$binary" "$INSTALL_DIR/academic-paper-interpreter"
    print_success "Linked: academic-paper-interpreter"

    if [[ -e "$INSTALL_DIR/paper-export" ]]; then
        print_warning "Overwriting existing: paper-export"
    fi
    ln -sf "$SCRIPT_DIR/paper-export" "$INSTALL_DIR/paper-export"
    print_success "Linked: paper-export"

    print_success "Installation complete!"
}

# Install by copying files (recommended)
install_copy() {
    print_info "Installing by copying to $INSTALL_DIR"

    check_permissions "$@"

    local binary
    binary=$(ensure_binary)

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Copy binary (check for existing file)
    if [[ -e "$INSTALL_DIR/academic-paper-interpreter" ]]; then
        print_warning "Overwriting existing: academic-paper-interpreter"
    fi
    install -m 755 "$binary" "$INSTALL_DIR/academic-paper-interpreter"
    chmod +x "$INSTALL_DIR/academic-paper-interpreter"
    print_success "Copied: academic-paper-interpreter"

    # Copy and modify script to use installed binary
    local temp_script
    temp_script=$(mktemp)

    # Modify the script to use the installed binary path
    sed 's|^BINARY=.*|BINARY="/usr/local/custom_command/academic-paper-interpreter"|' \
        "$SCRIPT_DIR/paper-export" > "$temp_script"

    # Check for existing paper-export before overwriting
    if [[ -e "$INSTALL_DIR/paper-export" ]]; then
        print_warning "Overwriting existing: paper-export"
    fi

    # Remove the binary search logic and replace with direct path
    cat > "$INSTALL_DIR/paper-export" << 'SCRIPT_HEADER'
#!/usr/bin/env bash
# paper-export - Academic paper export command wrapper
# Installed version - uses binary at /usr/local/custom_command/

set -euo pipefail

BINARY="/usr/local/custom_command/academic-paper-interpreter"

if [[ ! -x "$BINARY" ]]; then
    echo "Error: Binary not found at $BINARY" >&2
    echo "Please reinstall: ./scripts/install.sh" >&2
    exit 1
fi

SCRIPT_HEADER

    # Append the rest of the script (skip the header and binary detection)
    awk '/^# Default values$/,0' "$SCRIPT_DIR/paper-export" >> "$INSTALL_DIR/paper-export"

    chmod +x "$INSTALL_DIR/paper-export"
    rm -f "$temp_script"
    print_success "Copied: paper-export"

    print_success "Installation complete!"
}

# Uninstall
uninstall() {
    print_info "Uninstalling from $INSTALL_DIR"

    local removed=0

    if [[ -e "$INSTALL_DIR/academic-paper-interpreter" ]]; then
        rm -f "$INSTALL_DIR/academic-paper-interpreter"
        print_success "Removed: academic-paper-interpreter"
        ((removed++))
    fi

    if [[ -e "$INSTALL_DIR/paper-export" ]]; then
        rm -f "$INSTALL_DIR/paper-export"
        print_success "Removed: paper-export"
        ((removed++))
    fi

    if [[ $removed -eq 0 ]]; then
        print_warning "Nothing to remove"
    else
        print_success "Uninstallation complete!"
    fi
}

# Check PATH configuration
check_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo ""
        print_warning "$INSTALL_DIR is not in your PATH"
        echo ""
        echo "Add the following to your shell configuration file:"
        echo ""

        local shell_name
        shell_name=$(basename "$SHELL")

        case "$shell_name" in
            zsh)
                echo -e "  ${GREEN}echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.zshrc${NC}"
                echo -e "  ${GREEN}source ~/.zshrc${NC}"
                ;;
            bash)
                echo -e "  ${GREEN}echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.bashrc${NC}"
                echo -e "  ${GREEN}source ~/.bashrc${NC}"
                ;;
            *)
                echo -e "  ${GREEN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
                ;;
        esac
        echo ""
    else
        print_success "$INSTALL_DIR is already in PATH"
    fi
}

# Show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Install paper-export command to $INSTALL_DIR

OPTIONS:
    --copy, -c       Install by copying files (default, recommended)
    --symlink, -s    Install using symlinks (for development only)
    --remove, -r     Uninstall
    --check          Check installation status
    --help, -h       Show this help

EXAMPLES:
    $0               # Install by copying (default)
    $0 --symlink     # Install with symlinks (development)
    sudo $0          # Install with sudo if permission denied
    $0 --remove      # Uninstall

EOF
}

# Check installation status
check_installation() {
    echo "Installation status for $INSTALL_DIR:"
    echo ""

    if [[ -e "$INSTALL_DIR/academic-paper-interpreter" ]]; then
        if [[ -L "$INSTALL_DIR/academic-paper-interpreter" ]]; then
            local target
            target=$(readlink "$INSTALL_DIR/academic-paper-interpreter")
            print_success "academic-paper-interpreter -> $target"
        else
            print_success "academic-paper-interpreter (copied)"
        fi
    else
        print_warning "academic-paper-interpreter: not installed"
    fi

    if [[ -e "$INSTALL_DIR/paper-export" ]]; then
        if [[ -L "$INSTALL_DIR/paper-export" ]]; then
            local target
            target=$(readlink "$INSTALL_DIR/paper-export")
            print_success "paper-export -> $target"
        else
            print_success "paper-export (copied)"
        fi
    else
        print_warning "paper-export: not installed"
    fi

    echo ""
    check_path
}

# Main
main() {
    local mode="copy"

    while [[ $# -gt 0 ]]; do
        case $1 in
            --symlink|-s)
                mode="symlink"
                shift
                ;;
            --copy|-c)
                mode="copy"
                shift
                ;;
            --remove|-r|--uninstall)
                mode="remove"
                shift
                ;;
            --check)
                mode="check"
                shift
                ;;
            --help|-h)
                show_usage
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    case $mode in
        symlink)
            install_symlink "$@"
            check_path
            ;;
        copy)
            install_copy "$@"
            check_path
            ;;
        remove)
            uninstall
            ;;
        check)
            check_installation
            ;;
    esac
}

main "$@"
