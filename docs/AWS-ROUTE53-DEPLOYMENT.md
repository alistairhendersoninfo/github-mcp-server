# AWS Route 53 DNS Challenge Deployment Guide

This guide explains how to deploy the GitHub MCP Server with AWS Route 53 DNS challenge for SSL certificates, providing a more secure and reliable alternative to HTTP challenge.

## 🔐 Why AWS Route 53 DNS Challenge?

### Advantages over HTTP Challenge:
- ✅ **Works behind firewalls** - No need to expose port 80
- ✅ **Wildcard certificates** - Can generate `*.your-domain.com` certificates
- ✅ **More reliable** - No dependency on web server availability during renewal
- ✅ **Automated renewal** - Traefik handles everything automatically
- ✅ **Better security** - No temporary HTTP endpoints needed

### How It Works:
1. Traefik requests a certificate from Let's Encrypt
2. Let's Encrypt provides a DNS challenge token
3. Traefik uses AWS Route 53 API to create the required DNS TXT record
4. Let's Encrypt verifies the DNS record and issues the certificate
5. Traefik automatically renews certificates before expiration (30 days)

## 🏗️ Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Internet        │────│ AWS Route 53     │    │ Let's Encrypt   │
│ (Users)         │    │ DNS Management   │    │ Certificate     │
└─────────────────┘    └──────────────────┘    │ Authority       │
         │                        │             └─────────────────┘
         │                        │                       │
         ▼                        ▼                       ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Traefik         │────│ DNS Challenge    │────│ Certificate     │
│ Reverse Proxy   │    │ Automation       │    │ Management      │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Docker Services                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐ │
│  │ MCP Server  │ │ Nginx       │ │ Prometheus  │ │ Grafana   │ │
│  │ (Rust)      │ │ (Static)    │ │ (Metrics)   │ │ (Dashbrd) │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └───────────┘ │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐ │
│  │ Loki        │ │ Fail2ban    │ │ Secrets     │ │ Volumes   │ │
│  │ (Logs)      │ │ (Security)  │ │ Management  │ │ Storage   │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └───────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## 📋 Prerequisites

### 1. AWS Account Setup
- AWS account with Route 53 service
- Domain registered and managed by Route 53
- IAM user with Route 53 permissions

### 2. Server Requirements
- Linux server (Ubuntu 20.04+ recommended)
- Docker and Docker Compose installed
- Minimum 2GB RAM, 20GB storage
- Ports 80, 443 open to internet

### 3. Domain Configuration
- Domain DNS managed by AWS Route 53
- A record pointing to your server IP

## 🚀 Deployment Steps

### Step 1: Clone and Setup

```bash
# Clone the repository
git clone https://github.com/your-username/github-mcp-server.git
cd github-mcp-server

# Make scripts executable
chmod +x docker/scripts/setup-secrets.sh
```

### Step 2: AWS IAM User Setup

1. **Create IAM User**:
   ```bash
   # Using AWS CLI
   aws iam create-user --user-name github-mcp-route53
   ```

2. **Create IAM Policy**:
   ```json
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
   ```

3. **Attach Policy and Create Access Keys**:
   ```bash
   # Create policy
   aws iam create-policy --policy-name GitHubMCPRoute53Policy --policy-document file://route53-policy.json
   
   # Attach policy to user
   aws iam attach-user-policy --user-name github-mcp-route53 --policy-arn arn:aws:iam::YOUR-ACCOUNT:policy/GitHubMCPRoute53Policy
   
   # Create access keys
   aws iam create-access-key --user-name github-mcp-route53
   ```

### Step 3: Secure Secrets Setup

```bash
# Run the secure setup script
./docker/scripts/setup-secrets.sh
```

This script will:
- Create secure secrets directory with proper permissions
- Prompt for AWS credentials (Access Key ID and Secret)
- Generate secure JWT secret
- Prompt for GitHub OAuth client secret
- Generate Grafana admin password
- Create Traefik basic auth credentials
- Generate production environment template

### Step 4: GitHub OAuth App Setup

1. Go to [GitHub Developer Settings](https://github.com/settings/applications/new)
2. Create new OAuth App with:
   - **Application name**: GitHub MCP Server
   - **Homepage URL**: `https://your-domain.com`
   - **Authorization callback URL**: `https://your-domain.com/auth/github/callback`
3. Note the Client ID and Client Secret

### Step 5: Environment Configuration

```bash
# Edit the production environment file
nano docker/.env.production
```

Update with your values:
```bash
# Domain Configuration
DOMAIN=your-domain.com
ACME_EMAIL=your-email@domain.com

# AWS Configuration
AWS_REGION=us-east-1

# GitHub OAuth
GITHUB_CLIENT_ID=your-github-oauth-app-client-id

# Timezone
TZ=UTC

# Traefik Auth (from setup-secrets.sh output)
TRAEFIK_AUTH='admin:$2y$10$...'
```

### Step 6: Deploy Services

```bash
# Create external network for Traefik
docker network create traefik-public

# Deploy all services
docker-compose -f docker/docker-compose.prod.yml up -d

# Check service status
docker-compose -f docker/docker-compose.prod.yml ps
```

### Step 7: Verify Deployment

1. **Check Traefik Dashboard**:
   ```bash
   # Should show SSL certificate status
   curl -u admin:password https://traefik.your-domain.com
   ```

2. **Check MCP Server**:
   ```bash
   # Should return health status
   curl https://your-domain.com/health
   ```

3. **Check Certificate**:
   ```bash
   # Should show Let's Encrypt certificate
   openssl s_client -connect your-domain.com:443 -servername your-domain.com
   ```

## 🔧 Container Architecture

### Service Separation Benefits:

1. **Traefik Container**:
   - Handles SSL termination and routing
   - Manages Let's Encrypt certificates automatically
   - Provides load balancing and middleware

2. **MCP Server Container**:
   - Runs the Rust application
   - Handles GitHub API integration
   - Manages user authentication and workflows

3. **Nginx Container** (Optional):
   - Serves static assets efficiently
   - Provides additional caching layer
   - Can be removed if MCP server handles static files

4. **Monitoring Stack**:
   - **Prometheus**: Metrics collection
   - **Loki**: Log aggregation
   - **Grafana**: Dashboards and alerting

5. **Security Container**:
   - **Fail2ban**: Intrusion prevention
   - Monitors logs and blocks malicious IPs

### Container Communication:
- All containers communicate via Docker networks
- Secrets are managed via Docker secrets (not environment variables)
- Persistent data stored in Docker volumes

## 🔄 Certificate Renewal

### Automatic Renewal:
Traefik automatically handles certificate renewal:
- Checks certificates daily
- Renews when < 30 days remaining
- Uses AWS Route 53 DNS challenge
- Zero downtime renewal process

### Manual Renewal (if needed):
```bash
# Force certificate renewal
docker-compose -f docker/docker-compose.prod.yml restart traefik

# Check renewal logs
docker-compose -f docker/docker-compose.prod.yml logs traefik | grep -i acme
```

### Monitoring Renewal:
```bash
# Check certificate expiration
echo | openssl s_client -servername your-domain.com -connect your-domain.com:443 2>/dev/null | openssl x509 -noout -dates

# Check Traefik ACME storage
docker exec github-mcp-traefik cat /acme.json | jq '.letsencrypt.Certificates[0].certificate' | base64 -d | openssl x509 -noout -dates
```

## 🔍 Troubleshooting

### Common Issues:

1. **DNS Challenge Fails**:
   ```bash
   # Check AWS credentials
   docker exec github-mcp-traefik env | grep AWS
   
   # Check Route 53 permissions
   aws route53 list-hosted-zones --profile github-mcp
   
   # Check Traefik logs
   docker-compose -f docker/docker-compose.prod.yml logs traefik | grep -i error
   ```

2. **Certificate Not Issued**:
   ```bash
   # Check ACME storage
   docker exec github-mcp-traefik ls -la /acme.json
   
   # Verify domain DNS
   dig TXT _acme-challenge.your-domain.com
   
   # Check Let's Encrypt rate limits
   curl -s "https://crt.sh/?q=your-domain.com&output=json" | jq length
   ```

3. **Service Connection Issues**:
   ```bash
   # Check container networking
   docker network ls
   docker network inspect github-mcp-network
   
   # Test internal connectivity
   docker exec github-mcp-server curl -f http://traefik:8080/ping
   ```

### Debug Mode:
```bash
# Enable debug logging
docker-compose -f docker/docker-compose.prod.yml down
# Edit traefik.yml: log.level = DEBUG
docker-compose -f docker/docker-compose.prod.yml up -d traefik
docker-compose -f docker/docker-compose.prod.yml logs -f traefik
```

## 📊 Monitoring

### Access Dashboards:
- **Traefik Dashboard**: `https://traefik.your-domain.com`
- **Grafana Dashboard**: `https://dashboard.your-domain.com`
- **Prometheus Metrics**: `https://metrics.your-domain.com`

### Key Metrics to Monitor:
- Certificate expiration dates
- SSL handshake success rates
- Request response times
- Error rates and status codes
- Container resource usage

## 🔒 Security Considerations

### AWS Security:
- Use dedicated IAM user with minimal permissions
- Rotate access keys regularly
- Monitor CloudTrail for Route 53 API usage
- Consider using IAM roles instead of access keys

### Container Security:
- All containers run as non-root users
- Secrets stored securely (not in environment variables)
- Regular security updates via base image updates
- Network isolation between services

### SSL Security:
- TLS 1.3 enabled by default
- Strong cipher suites configured
- HSTS headers enforced
- Certificate transparency monitoring

## 📚 Additional Resources

- [Traefik Route 53 Documentation](https://doc.traefik.io/traefik/https/acme/#route53)
- [AWS Route 53 API Reference](https://docs.aws.amazon.com/Route53/latest/APIReference/)
- [Let's Encrypt Rate Limits](https://letsencrypt.org/docs/rate-limits/)
- [Docker Secrets Management](https://docs.docker.com/engine/swarm/secrets/)

---

**Next Steps**: After successful deployment, proceed to [Claude/Cursor Integration Guide](CLAUDE-CURSOR-INTEGRATION.md) to configure your AI assistants.