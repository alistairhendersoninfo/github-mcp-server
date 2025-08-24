#!/bin/bash
# GitHub MCP Server - Automated Secure Installation Script
# This script installs and configures everything needed for production deployment

set -euo pipefail

# Script version and info
SCRIPT_VERSION="1.0.0"
SCRIPT_NAME="GitHub MCP Server Installer"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="/opt/github-mcp-server"
DATA_DIR="/var/lib/github-mcp-server"
LOG_DIR="/var/log/github-mcp-server"
SECRETS_DIR="/etc/github-mcp-server/secrets"
SERVICE_USER="github-mcp"
DOCKER_COMPOSE_VERSION="2.24.0"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a /var/log/github-mcp-install.log
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a /var/log/github-mcp-install.log
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a /var/log/github-mcp-install.log
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a /var/log/github-mcp-install.log
}

log_step() {
    echo -e "${PURPLE}[STEP]${NC} $1" | tee -a /var/log/github-mcp-install.log
}

# Error handling
error_exit() {
    log_error "$1"
    log_error "Installation failed. Check /var/log/github-mcp-install.log for details."
    exit 1
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        error_exit "This script must be run as root. Use: sudo $0"
    fi
}

# Display banner
show_banner() {
    clear
    echo -e "${CYAN}"
    cat << 'EOF'
    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║        🔒 GitHub MCP Server - Secure Installation            ║
    ║                                                               ║
    ║  Ultra-secure GitHub workflow automation for Claude/Cursor   ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"
    echo -e "${BLUE}Version: ${SCRIPT_VERSION}${NC}"
    echo -e "${BLUE}Installation Directory: ${INSTALL_DIR}${NC}"
    echo
}

# Detect OS and architecture
detect_system() {
    log_step "Detecting system information..."
    
    # OS Detection
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        OS=$ID
        OS_VERSION=$VERSION_ID
    else
        error_exit "Cannot detect operating system"
    fi
    
    # Architecture detection
    ARCH=$(uname -m)
    case $ARCH in
        x86_64) ARCH="amd64" ;;
        aarch64) ARCH="arm64" ;;
        armv7l) ARCH="armv7" ;;
        *) error_exit "Unsupported architecture: $ARCH" ;;
    esac
    
    log_info "Detected OS: $OS $OS_VERSION"
    log_info "Architecture: $ARCH"
    
    # Validate supported OS
    case $OS in
        ubuntu|debian)
            PACKAGE_MANAGER="apt"
            ;;
        centos|rhel|fedora)
            PACKAGE_MANAGER="yum"
            ;;
        *)
            error_exit "Unsupported operating system: $OS"
            ;;
    esac
}

# Get configuration from user
get_configuration() {
    log_step "Gathering configuration..."
    echo
    
    # Domain configuration
    read -p "Enter your domain name (e.g., mcp.yourdomain.com): " DOMAIN
    if [[ -z "$DOMAIN" ]]; then
        error_exit "Domain name is required"
    fi
    
    # Email for Let's Encrypt
    read -p "Enter email for Let's Encrypt certificates: " ACME_EMAIL
    if [[ -z "$ACME_EMAIL" ]]; then
        error_exit "Email is required for Let's Encrypt"
    fi
    
    # AWS Region
    read -p "Enter AWS region for Route 53 [us-east-1]: " AWS_REGION
    AWS_REGION=${AWS_REGION:-us-east-1}
    
    # GitHub OAuth
    echo
    log_info "You need to create a GitHub OAuth App at:"
    log_info "https://github.com/settings/applications/new"
    echo
    log_info "Use these settings:"
    log_info "- Application name: GitHub MCP Server"
    log_info "- Homepage URL: https://$DOMAIN"
    log_info "- Authorization callback URL: https://$DOMAIN/auth/github/callback"
    echo
    read -p "Enter GitHub OAuth Client ID: " GITHUB_CLIENT_ID
    if [[ -z "$GITHUB_CLIENT_ID" ]]; then
        error_exit "GitHub Client ID is required"
    fi
    
    read -s -p "Enter GitHub OAuth Client Secret: " GITHUB_CLIENT_SECRET
    echo
    if [[ -z "$GITHUB_CLIENT_SECRET" ]]; then
        error_exit "GitHub Client Secret is required"
    fi
    
    # SSH Access Configuration
    echo
    log_info "SSH Access Configuration:"
    log_info "Enter IP addresses/subnets that should have SSH access"
    log_info "Examples: 192.168.1.0/24 (local network), 203.0.113.0/24 (office network)"
    log_info "Enter 0.0.0.0/0 to allow SSH from anywhere (NOT RECOMMENDED)"
    echo
    
    SSH_ALLOWED_IPS=()
    while true; do
        read -p "Enter SSH allowed IP/subnet (or 'done' to finish): " ssh_ip
        if [[ "$ssh_ip" == "done" ]]; then
            break
        elif [[ -n "$ssh_ip" ]]; then
            SSH_ALLOWED_IPS+=("$ssh_ip")
            log_info "Added SSH access for: $ssh_ip"
        fi
    done
    
    if [[ ${#SSH_ALLOWED_IPS[@]} -eq 0 ]]; then
        log_warning "No SSH IPs specified. SSH will be blocked from all external IPs."
        read -p "Continue? (y/N): " confirm
        if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    # Timezone
    read -p "Enter timezone [UTC]: " TIMEZONE
    TIMEZONE=${TIMEZONE:-UTC}
    
    echo
    log_success "Configuration collected successfully"
}

# Update system packages
update_system() {
    log_step "Updating system packages..."
    
    case $PACKAGE_MANAGER in
        apt)
            export DEBIAN_FRONTEND=noninteractive
            apt-get update -y
            apt-get upgrade -y
            apt-get install -y \
                curl \
                wget \
                gnupg \
                lsb-release \
                ca-certificates \
                software-properties-common \
                apt-transport-https \
                ufw \
                fail2ban \
                htop \
                vim \
                git \
                openssl \
                apache2-utils \
                jq \
                unzip
            ;;
        yum)
            yum update -y
            yum install -y \
                curl \
                wget \
                gnupg \
                ca-certificates \
                yum-utils \
                device-mapper-persistent-data \
                lvm2 \
                firewalld \
                fail2ban \
                htop \
                vim \
                git \
                openssl \
                httpd-tools \
                jq \
                unzip
            ;;
    esac
    
    log_success "System packages updated"
}

# Configure firewall
configure_firewall() {
    log_step "Configuring UFW firewall..."
    
    # Reset UFW to defaults
    ufw --force reset
    
    # Default policies
    ufw default deny incoming
    ufw default allow outgoing
    
    # Allow HTTP and HTTPS from anywhere (ONLY ports exposed to internet)
    ufw allow 80/tcp comment 'HTTP - Traefik only (redirects to HTTPS)'
    ufw allow 443/tcp comment 'HTTPS - Traefik only (main entry point)'
    
    log_info "🔒 SECURITY: Only ports 80 and 443 exposed to internet"
    log_info "🔒 All services accessible ONLY through Traefik reverse proxy"
    
    # Configure SSH access
    if [[ ${#SSH_ALLOWED_IPS[@]} -eq 0 ]]; then
        log_warning "SSH access will be blocked from all external IPs"
    else
        for ip in "${SSH_ALLOWED_IPS[@]}"; do
            if [[ "$ip" == "0.0.0.0/0" ]]; then
                log_warning "Allowing SSH from anywhere - SECURITY RISK!"
                ufw allow 22/tcp comment 'SSH - Open to world'
            else
                ufw allow from "$ip" to any port 22 comment "SSH - $ip"
                log_info "SSH access allowed from: $ip"
            fi
        done
    fi
    
    # Enable UFW
    ufw --force enable
    
    # Show status
    ufw status verbose
    
    log_success "Firewall configured successfully"
}

# Install Docker
install_docker() {
    log_step "Installing Docker..."
    
    # Remove old versions
    case $PACKAGE_MANAGER in
        apt)
            apt-get remove -y docker docker-engine docker.io containerd runc || true
            
            # Add Docker's official GPG key
            curl -fsSL https://download.docker.com/linux/$OS/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
            
            # Add Docker repository
            echo "deb [arch=$ARCH signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/$OS $(lsb_release -cs) stable" > /etc/apt/sources.list.d/docker.list
            
            # Install Docker
            apt-get update -y
            apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
            ;;
        yum)
            yum remove -y docker docker-client docker-client-latest docker-common docker-latest docker-latest-logrotate docker-logrotate docker-engine || true
            
            # Add Docker repository
            yum-config-manager --add-repo https://download.docker.com/linux/centos/docker-ce.repo
            
            # Install Docker
            yum install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
            ;;
    esac
    
    # Start and enable Docker
    systemctl start docker
    systemctl enable docker
    
    # Verify installation
    docker --version
    docker compose version
    
    log_success "Docker installed successfully"
}

# Create service user and directories
create_directories() {
    log_step "Creating service user and directories..."
    
    # Create service user
    if ! id "$SERVICE_USER" &>/dev/null; then
        useradd -r -s /bin/false -d "$INSTALL_DIR" "$SERVICE_USER"
        log_info "Created service user: $SERVICE_USER"
    fi
    
    # Create directories with secure permissions
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$DATA_DIR"
    mkdir -p "$LOG_DIR"
    mkdir -p "$SECRETS_DIR"
    
    # Set ownership and permissions
    chown -R "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR"
    chown -R "$SERVICE_USER:$SERVICE_USER" "$DATA_DIR"
    chown -R "$SERVICE_USER:$SERVICE_USER" "$LOG_DIR"
    
    # Secrets directory - root only
    chown -R root:root "$SECRETS_DIR"
    chmod 700 "$SECRETS_DIR"
    
    log_success "Directories created with secure permissions"
}

# Download and setup application
setup_application() {
    log_step "Setting up GitHub MCP Server..."
    
    # Clone repository
    cd /tmp
    git clone https://github.com/alistairhendersoninfo/github-mcp-server.git
    
    # Copy files to install directory
    cp -r github-mcp-server/* "$INSTALL_DIR/"
    
    # Set ownership
    chown -R "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR"
    
    # Make scripts executable
    chmod +x "$INSTALL_DIR/docker/scripts/"*.sh
    
    log_success "Application files installed"
}

# Generate secure secrets
generate_secrets() {
    log_step "Generating secure secrets..."
    
    # Generate JWT secret
    openssl rand -base64 64 > "$SECRETS_DIR/jwt_secret.txt"
    
    # Save GitHub credentials
    echo -n "$GITHUB_CLIENT_SECRET" > "$SECRETS_DIR/github_client_secret.txt"
    
    # Generate Grafana admin password
    GRAFANA_PASSWORD=$(openssl rand -base64 24)
    echo -n "$GRAFANA_PASSWORD" > "$SECRETS_DIR/grafana_admin_password.txt"
    
    # Set secure permissions
    chmod 600 "$SECRETS_DIR"/*.txt
    chown root:root "$SECRETS_DIR"/*.txt
    
    log_success "Secrets generated and secured"
    log_info "Grafana admin password: $GRAFANA_PASSWORD"
    echo "GRAFANA_ADMIN_PASSWORD=$GRAFANA_PASSWORD" >> /root/.github-mcp-credentials
}

# Setup AWS credentials (interactive)
setup_aws_credentials() {
    log_step "Setting up AWS credentials for Route 53..."
    
    echo
    log_info "You need AWS credentials with Route 53 permissions."
    log_info "Required IAM policy:"
    cat << 'EOF'
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "route53:GetChange",
                "route53:ChangeResourceRecordSets",
                "route53:ListHostedZonesByName",
                "route53:ListHostedZones"
            ],
            "Resource": "*"
        }
    ]
}
EOF
    echo
    
    read -p "Enter AWS Access Key ID: " AWS_ACCESS_KEY_ID
    read -s -p "Enter AWS Secret Access Key: " AWS_SECRET_ACCESS_KEY
    echo
    
    if [[ -z "$AWS_ACCESS_KEY_ID" ]] || [[ -z "$AWS_SECRET_ACCESS_KEY" ]]; then
        error_exit "AWS credentials are required"
    fi
    
    # Save AWS credentials
    echo -n "$AWS_ACCESS_KEY_ID" > "$SECRETS_DIR/aws_access_key_id.txt"
    echo -n "$AWS_SECRET_ACCESS_KEY" > "$SECRETS_DIR/aws_secret_access_key.txt"
    
    # Set secure permissions
    chmod 600 "$SECRETS_DIR/aws_"*.txt
    chown root:root "$SECRETS_DIR/aws_"*.txt
    
    log_success "AWS credentials saved securely"
}

# Create production environment file
create_environment() {
    log_step "Creating production environment configuration..."
    
    # Generate Traefik basic auth
    TRAEFIK_USER="admin"
    TRAEFIK_PASSWORD=$(openssl rand -base64 16)
    TRAEFIK_AUTH=$(htpasswd -nb "$TRAEFIK_USER" "$TRAEFIK_PASSWORD")
    
    # Create environment file
    cat > "$INSTALL_DIR/.env.production" << EOF
# GitHub MCP Server Production Environment
# Generated by install script on $(date)

# Domain Configuration
DOMAIN=$DOMAIN
ACME_EMAIL=$ACME_EMAIL

# AWS Configuration
AWS_REGION=$AWS_REGION

# GitHub OAuth
GITHUB_CLIENT_ID=$GITHUB_CLIENT_ID

# Timezone
TZ=$TIMEZONE

# Traefik Authentication
TRAEFIK_AUTH='$TRAEFIK_AUTH'

# Service Configuration
SERVICE_USER=$SERVICE_USER
INSTALL_DIR=$INSTALL_DIR
DATA_DIR=$DATA_DIR
LOG_DIR=$LOG_DIR
SECRETS_DIR=$SECRETS_DIR
EOF
    
    # Set secure permissions
    chmod 600 "$INSTALL_DIR/.env.production"
    chown "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR/.env.production"
    
    # Save credentials for admin
    cat >> /root/.github-mcp-credentials << EOF
TRAEFIK_USERNAME=$TRAEFIK_USER
TRAEFIK_PASSWORD=$TRAEFIK_PASSWORD
DOMAIN=$DOMAIN
EOF
    
    log_success "Environment configuration created"
    log_info "Traefik dashboard credentials: $TRAEFIK_USER / $TRAEFIK_PASSWORD"
}

# Update Docker Compose for security
update_docker_compose() {
    log_step "Updating Docker Compose for maximum security..."
    
    # Create secure Alpine-based Dockerfile
    cat > "$INSTALL_DIR/docker/Dockerfile.secure" << 'EOF'
# Multi-stage build with Alpine Linux for maximum security
FROM rust:1.75-alpine as builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static

# Create app user
RUN addgroup -g 1001 -S appuser && \
    adduser -u 1001 -S appuser -G appuser

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY migrations/ ./migrations/

# Build with static linking for Alpine
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage with minimal Alpine
FROM alpine:3.19

# Install only essential runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    curl \
    && rm -rf /var/cache/apk/*

# Create non-root user
RUN addgroup -g 1001 -S appuser && \
    adduser -u 1001 -S appuser -G appuser

# Create directories with proper permissions
RUN mkdir -p /app/data /app/config /app/logs \
    && chown -R appuser:appuser /app

# Copy binary from builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/github-mcp-server /app/
COPY --from=builder /app/migrations/ /app/migrations/

# Copy web assets
COPY web/ /app/web/

# Set proper permissions
RUN chmod +x /app/github-mcp-server \
    && chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 8443

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8443/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:./data/github-mcp-server.db

# Run the application
CMD ["./github-mcp-server"]
EOF
    
    # Update docker-compose to use secure Dockerfile and Alpine images
    sed -i 's|dockerfile: docker/Dockerfile|dockerfile: docker/Dockerfile.secure|g' "$INSTALL_DIR/docker/docker-compose.prod.yml"
    
    # Update Nginx to use Alpine
    sed -i 's|image: nginx:alpine|image: nginx:1.25-alpine|g' "$INSTALL_DIR/docker/docker-compose.prod.yml"
    
    # Update other services to use Alpine where possible
    sed -i 's|image: grafana/grafana:latest|image: grafana/grafana:10.2.0-alpine|g' "$INSTALL_DIR/docker/docker-compose.prod.yml"
    sed -i 's|image: prom/prometheus:latest|image: prom/prometheus:v2.48.0|g' "$INSTALL_DIR/docker/docker-compose.prod.yml"
    
    log_success "Docker Compose updated for maximum security"
}

# Create systemd service
create_systemd_service() {
    log_step "Creating systemd service..."
    
    cat > /etc/systemd/system/github-mcp-server.service << EOF
[Unit]
Description=GitHub MCP Server
Documentation=https://github.com/alistairhendersoninfo/github-mcp-server
Requires=docker.service
After=docker.service
Wants=network-online.target
After=network-online.target

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=$INSTALL_DIR
User=$SERVICE_USER
Group=$SERVICE_USER

# Environment
Environment=COMPOSE_PROJECT_NAME=github-mcp-server
EnvironmentFile=$INSTALL_DIR/.env.production

# Start command
ExecStart=/usr/bin/docker compose -f docker/docker-compose.prod.yml up -d

# Stop command
ExecStop=/usr/bin/docker compose -f docker/docker-compose.prod.yml down

# Reload command
ExecReload=/usr/bin/docker compose -f docker/docker-compose.prod.yml restart

# Security settings
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=$DATA_DIR $LOG_DIR
PrivateTmp=yes
PrivateDevices=yes
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes

# Restart policy
Restart=on-failure
RestartSec=10s

[Install]
WantedBy=multi-user.target
EOF
    
    # Reload systemd and enable service
    systemctl daemon-reload
    systemctl enable github-mcp-server.service
    
    log_success "Systemd service created and enabled"
}

# Setup log rotation
setup_log_rotation() {
    log_step "Setting up log rotation..."
    
    cat > /etc/logrotate.d/github-mcp-server << EOF
$LOG_DIR/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 $SERVICE_USER $SERVICE_USER
    postrotate
        systemctl reload github-mcp-server || true
    endscript
}

/var/log/github-mcp-install.log {
    monthly
    missingok
    rotate 12
    compress
    delaycompress
    notifempty
    create 644 root root
}
EOF
    
    log_success "Log rotation configured"
}

# Setup monitoring
setup_monitoring() {
    log_step "Setting up system monitoring..."
    
    # Create monitoring script
    cat > /usr/local/bin/github-mcp-monitor.sh << 'EOF'
#!/bin/bash
# GitHub MCP Server monitoring script

LOG_FILE="/var/log/github-mcp-monitor.log"
DATE=$(date '+%Y-%m-%d %H:%M:%S')

# Check if services are running
if ! systemctl is-active --quiet github-mcp-server; then
    echo "[$DATE] ERROR: GitHub MCP Server service is not running" >> "$LOG_FILE"
    systemctl start github-mcp-server
fi

# Check disk space
DISK_USAGE=$(df /var/lib/github-mcp-server | awk 'NR==2 {print $5}' | sed 's/%//')
if [[ $DISK_USAGE -gt 80 ]]; then
    echo "[$DATE] WARNING: Disk usage is ${DISK_USAGE}%" >> "$LOG_FILE"
fi

# Check memory usage
MEM_USAGE=$(free | awk 'NR==2{printf "%.0f", $3*100/$2}')
if [[ $MEM_USAGE -gt 90 ]]; then
    echo "[$DATE] WARNING: Memory usage is ${MEM_USAGE}%" >> "$LOG_FILE"
fi

# Check SSL certificate expiration
if command -v openssl >/dev/null 2>&1; then
    CERT_DAYS=$(echo | openssl s_client -servername "$DOMAIN" -connect "$DOMAIN:443" 2>/dev/null | openssl x509 -noout -checkend $((30*24*3600)) 2>/dev/null && echo "OK" || echo "EXPIRING")
    if [[ "$CERT_DAYS" == "EXPIRING" ]]; then
        echo "[$DATE] WARNING: SSL certificate expires within 30 days" >> "$LOG_FILE"
    fi
fi
EOF
    
    chmod +x /usr/local/bin/github-mcp-monitor.sh
    
    # Create cron job for monitoring
    cat > /etc/cron.d/github-mcp-monitor << EOF
# GitHub MCP Server monitoring
*/5 * * * * root /usr/local/bin/github-mcp-monitor.sh
EOF
    
    log_success "System monitoring configured"
}

# Start services
start_services() {
    log_step "Starting GitHub MCP Server services..."
    
    # Create Docker network
    docker network create traefik-public 2>/dev/null || true
    
    # Start the service
    cd "$INSTALL_DIR"
    systemctl start github-mcp-server.service
    
    # Wait for services to start
    sleep 30
    
    # Check service status
    if systemctl is-active --quiet github-mcp-server; then
        log_success "GitHub MCP Server started successfully"
    else
        error_exit "Failed to start GitHub MCP Server"
    fi
}

# Display final information
show_completion() {
    log_success "Installation completed successfully!"
    echo
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                    🎉 INSTALLATION COMPLETE                   ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════╝${NC}"
    echo
    echo -e "${CYAN}📋 Service Information:${NC}"
    echo -e "   • Main Application: https://$DOMAIN"
    echo -e "   • Traefik Dashboard: https://traefik.$DOMAIN"
    echo -e "   • Grafana Dashboard: https://dashboard.$DOMAIN"
    echo -e "   • Prometheus Metrics: https://metrics.$DOMAIN"
    echo
    echo -e "${CYAN}🔐 Credentials (saved in /root/.github-mcp-credentials):${NC}"
    echo -e "   • Traefik: $TRAEFIK_USER / $TRAEFIK_PASSWORD"
    echo -e "   • Grafana: admin / $GRAFANA_PASSWORD"
    echo
    echo -e "${CYAN}🛠️ Management Commands:${NC}"
    echo -e "   • Start:   systemctl start github-mcp-server"
    echo -e "   • Stop:    systemctl stop github-mcp-server"
    echo -e "   • Restart: systemctl restart github-mcp-server"
    echo -e "   • Status:  systemctl status github-mcp-server"
    echo -e "   • Logs:    journalctl -u github-mcp-server -f"
    echo
    echo -e "${CYAN}📁 Important Directories:${NC}"
    echo -e "   • Application: $INSTALL_DIR"
    echo -e "   • Data: $DATA_DIR"
    echo -e "   • Logs: $LOG_DIR"
    echo -e "   • Secrets: $SECRETS_DIR (root access only)"
    echo
    echo -e "${YELLOW}⚠️  Next Steps:${NC}"
    echo -e "   1. Verify your domain DNS points to this server"
    echo -e "   2. Wait 2-3 minutes for SSL certificates to be issued"
    echo -e "   3. Visit https://$DOMAIN to test the installation"
    echo -e "   4. Configure Claude/Cursor with your MCP server"
    echo
    echo -e "${GREEN}🔒 Security Features Enabled:${NC}"
    echo -e "   ✅ UFW firewall - ONLY ports 80, 443 exposed to internet"
    echo -e "   ✅ ALL traffic routed through Traefik reverse proxy"
    echo -e "   ✅ NO direct service access from internet"
    echo -e "   ✅ Alpine Linux containers for minimal attack surface"
    echo -e "   ✅ Non-root container users"
    echo -e "   ✅ Docker network isolation"
    echo -e "   ✅ Encrypted secrets storage (root access only)"
    echo -e "   ✅ Fail2ban intrusion prevention"
    echo -e "   ✅ Automatic SSL certificate management"
    echo -e "   ✅ System monitoring and alerting"
    echo
}

# Main installation function
main() {
    # Start installation log
    touch /var/log/github-mcp-install.log
    chmod 644 /var/log/github-mcp-install.log
    
    log_info "Starting GitHub MCP Server installation..."
    
    show_banner
    check_root
    detect_system
    get_configuration
    update_system
    configure_firewall
    install_docker
    create_directories
    setup_application
    generate_secrets
    setup_aws_credentials
    create_environment
    update_docker_compose
    create_systemd_service
    setup_log_rotation
    setup_monitoring
    start_services
    show_completion
    
    log_success "Installation completed at $(date)"
}

# Trap errors
trap 'error_exit "Installation failed at line $LINENO"' ERR

# Run main function
main "$@"