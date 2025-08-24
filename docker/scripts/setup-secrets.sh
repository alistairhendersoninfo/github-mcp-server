#!/bin/bash
# Secure AWS credentials and secrets setup script
# This script helps create and manage Docker secrets securely

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SECRETS_DIR="$SCRIPT_DIR/../secrets"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
check_root() {
    if [[ $EUID -eq 0 ]]; then
        log_error "This script should not be run as root for security reasons"
        exit 1
    fi
}

# Create secrets directory with proper permissions
create_secrets_dir() {
    log_info "Creating secrets directory..."
    
    if [[ ! -d "$SECRETS_DIR" ]]; then
        mkdir -p "$SECRETS_DIR"
        chmod 700 "$SECRETS_DIR"
        log_success "Created secrets directory: $SECRETS_DIR"
    else
        log_info "Secrets directory already exists"
    fi
    
    # Ensure proper permissions
    chmod 700 "$SECRETS_DIR"
}

# Generate secure random password
generate_password() {
    local length=${1:-32}
    openssl rand -base64 $length | tr -d "=+/" | cut -c1-$length
}

# Create AWS credentials
setup_aws_credentials() {
    log_info "Setting up AWS credentials for Route 53 DNS challenge..."
    
    local aws_access_key_file="$SECRETS_DIR/aws_access_key_id.txt"
    local aws_secret_key_file="$SECRETS_DIR/aws_secret_access_key.txt"
    
    if [[ -f "$aws_access_key_file" ]] && [[ -f "$aws_secret_key_file" ]]; then
        log_warning "AWS credentials already exist. Skipping..."
        return 0
    fi
    
    echo
    log_info "You need to create an AWS IAM user with Route 53 permissions."
    log_info "Required IAM policy for the user:"
    echo
    cat << 'EOF'
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "route53:GetChange",
                "route53:ChangeResourceRecordSets",
                "route53:ListHostedZonesByName"
            ],
            "Resource": [
                "arn:aws:route53:::hostedzone/*",
                "arn:aws:route53:::change/*"
            ]
        },
        {
            "Effect": "Allow",
            "Action": [
                "route53:ListHostedZones"
            ],
            "Resource": "*"
        }
    ]
}
EOF
    echo
    
    read -p "Enter AWS Access Key ID: " aws_access_key_id
    read -s -p "Enter AWS Secret Access Key: " aws_secret_access_key
    echo
    
    # Validate inputs
    if [[ -z "$aws_access_key_id" ]] || [[ -z "$aws_secret_access_key" ]]; then
        log_error "AWS credentials cannot be empty"
        exit 1
    fi
    
    # Save credentials to files
    echo -n "$aws_access_key_id" > "$aws_access_key_file"
    echo -n "$aws_secret_access_key" > "$aws_secret_key_file"
    
    # Set secure permissions
    chmod 600 "$aws_access_key_file" "$aws_secret_key_file"
    
    log_success "AWS credentials saved securely"
}

# Create JWT secret
setup_jwt_secret() {
    log_info "Setting up JWT secret..."
    
    local jwt_secret_file="$SECRETS_DIR/jwt_secret.txt"
    
    if [[ -f "$jwt_secret_file" ]]; then
        log_warning "JWT secret already exists. Skipping..."
        return 0
    fi
    
    local jwt_secret
    jwt_secret=$(generate_password 64)
    echo -n "$jwt_secret" > "$jwt_secret_file"
    chmod 600 "$jwt_secret_file"
    
    log_success "JWT secret generated and saved"
}

# Create GitHub client secret
setup_github_secret() {
    log_info "Setting up GitHub OAuth client secret..."
    
    local github_secret_file="$SECRETS_DIR/github_client_secret.txt"
    
    if [[ -f "$github_secret_file" ]]; then
        log_warning "GitHub client secret already exists. Skipping..."
        return 0
    fi
    
    echo
    log_info "You need to create a GitHub OAuth App at:"
    log_info "https://github.com/settings/applications/new"
    echo
    log_info "Use these settings:"
    log_info "- Application name: GitHub MCP Server"
    log_info "- Homepage URL: https://your-domain.com"
    log_info "- Authorization callback URL: https://your-domain.com/auth/github/callback"
    echo
    
    read -s -p "Enter GitHub OAuth Client Secret: " github_client_secret
    echo
    
    if [[ -z "$github_client_secret" ]]; then
        log_error "GitHub client secret cannot be empty"
        exit 1
    fi
    
    echo -n "$github_client_secret" > "$github_secret_file"
    chmod 600 "$github_secret_file"
    
    log_success "GitHub client secret saved securely"
}

# Create Grafana admin password
setup_grafana_password() {
    log_info "Setting up Grafana admin password..."
    
    local grafana_password_file="$SECRETS_DIR/grafana_admin_password.txt"
    
    if [[ -f "$grafana_password_file" ]]; then
        log_warning "Grafana admin password already exists. Skipping..."
        return 0
    fi
    
    local grafana_password
    grafana_password=$(generate_password 24)
    echo -n "$grafana_password" > "$grafana_password_file"
    chmod 600 "$grafana_password_file"
    
    log_success "Grafana admin password generated: $grafana_password"
    log_info "Save this password - you'll need it to access Grafana dashboard"
}

# Create Traefik basic auth
setup_traefik_auth() {
    log_info "Setting up Traefik dashboard authentication..."
    
    if command -v htpasswd >/dev/null 2>&1; then
        read -p "Enter username for Traefik dashboard [admin]: " traefik_user
        traefik_user=${traefik_user:-admin}
        
        read -s -p "Enter password for Traefik dashboard: " traefik_password
        echo
        
        if [[ -z "$traefik_password" ]]; then
            log_error "Traefik password cannot be empty"
            exit 1
        fi
        
        local traefik_auth
        traefik_auth=$(htpasswd -nb "$traefik_user" "$traefik_password")
        
        echo
        log_success "Add this to your .env file:"
        echo "TRAEFIK_AUTH='$traefik_auth'"
    else
        log_warning "htpasswd not found. Install apache2-utils to generate Traefik auth"
        log_info "You can generate it online at: https://hostingcanada.org/htpasswd-generator/"
    fi
}

# Validate secrets
validate_secrets() {
    log_info "Validating secrets..."
    
    local required_secrets=(
        "aws_access_key_id.txt"
        "aws_secret_access_key.txt"
        "jwt_secret.txt"
        "github_client_secret.txt"
        "grafana_admin_password.txt"
    )
    
    local missing_secrets=()
    
    for secret in "${required_secrets[@]}"; do
        if [[ ! -f "$SECRETS_DIR/$secret" ]]; then
            missing_secrets+=("$secret")
        fi
    done
    
    if [[ ${#missing_secrets[@]} -eq 0 ]]; then
        log_success "All required secrets are present"
    else
        log_error "Missing secrets: ${missing_secrets[*]}"
        exit 1
    fi
}

# Create .env template
create_env_template() {
    log_info "Creating .env template..."
    
    local env_file="$SCRIPT_DIR/../.env.production"
    
    cat > "$env_file" << 'EOF'
# GitHub MCP Server Production Environment Configuration

# Domain Configuration
DOMAIN=your-domain.com
ACME_EMAIL=your-email@domain.com

# AWS Configuration for Route 53 DNS Challenge
AWS_REGION=us-east-1

# GitHub OAuth Configuration
GITHUB_CLIENT_ID=your-github-oauth-app-client-id

# Timezone
TZ=UTC

# Traefik Basic Auth (generate with setup-secrets.sh)
TRAEFIK_AUTH=admin:$2y$10$...

# Optional: GitHub Project Number
GITHUB_PROJECT_NUMBER=123
EOF
    
    chmod 600 "$env_file"
    log_success "Created .env template: $env_file"
    log_info "Please edit this file with your actual values"
}

# Main function
main() {
    log_info "GitHub MCP Server - Secure Secrets Setup"
    log_info "========================================"
    
    check_root
    create_secrets_dir
    
    setup_aws_credentials
    setup_jwt_secret
    setup_github_secret
    setup_grafana_password
    setup_traefik_auth
    
    validate_secrets
    create_env_template
    
    echo
    log_success "Secrets setup completed successfully!"
    echo
    log_info "Next steps:"
    log_info "1. Edit docker/.env.production with your domain and configuration"
    log_info "2. Ensure your domain's DNS is managed by AWS Route 53"
    log_info "3. Run: docker-compose -f docker/docker-compose.prod.yml up -d"
    echo
    log_warning "Keep the secrets directory secure and backed up!"
    log_warning "Never commit secrets to version control!"
}

# Run main function
main "$@"