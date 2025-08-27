# GitHub MCP Server

🔒 **Ultra-secure GitHub MCP server with workflow automation for Claude & Cursor**

A self-hosted Model Context Protocol (MCP) server that provides secure GitHub integration with intelligent workflow automation. Built with Rust for maximum security and performance.

## ✨ Features

- 🔐 **Ultra-secure architecture** with Traefik, Let's Encrypt SSL, and comprehensive security headers
- ⚡ **Intelligent workflow commands**: `push`, `scan tasks`, `merge` with smart automation
- 🛡️ **OAuth 2.0 authentication** with encrypted token storage and audit logging
- 📋 **GitHub Projects integration** with task scanning and status updates
- 🚀 **Complete CI/CD workflow** from task selection to production deployment
- 🔄 **Real-time MCP protocol** support via WebSocket and HTTP
- 📊 **Comprehensive monitoring** with health checks and metrics
- 🐳 **Production-ready deployment** with Docker and Traefik

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Claude/Cursor   │────│ Traefik + SSL    │────│ GitHub MCP      │
│ Client          │    │ (AWS Route 53)   │    │ Server (Rust)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                       ┌──────────────────┐    ┌─────────────────┐
                       │ AWS Route 53     │    │ GitHub API      │
                       │ DNS Challenge    │    │ & Projects      │
                       └──────────────────┘    └─────────────────┘
                                │
                       ┌──────────────────┐
                       │ Let's Encrypt    │
                       │ Auto Renewal     │
                       └──────────────────┘
```

### Multi-Container Production Architecture:
- **Traefik**: Reverse proxy with AWS Route 53 DNS challenge for SSL
- **MCP Server**: Rust application with GitHub integration
- **Nginx**: Static asset serving (optional)
- **Prometheus**: Metrics collection and monitoring
- **Grafana**: Dashboards and alerting
- **Loki**: Log aggregation and analysis
- **Fail2ban**: Intrusion detection and prevention

## 🚀 Quick Start

### 📋 Prerequisites (REQUIRED READING)

**🚨 IMPORTANT: Read the prerequisites guide FIRST:**

📖 **[Installation Prerequisites Guide](docs/INSTALLATION-PREREQUISITES.md)**

This guide covers:
- Domain and AWS Route 53 setup
- GitHub App creation (step-by-step)
- AWS IAM credentials configuration
- Security planning
- All information you need before installation

### 🛠️ Automated Installation

Once you've completed the prerequisites:

```bash
# Download the installer
curl -fsSL https://raw.githubusercontent.com/your-username/github-mcp-server/main/install.sh -o install.sh
chmod +x install.sh

# Run the installation (requires sudo)
sudo ./install.sh
```

The installer will:
- ✅ Install Docker and dependencies
- ✅ Configure UFW firewall (only ports 80, 443 exposed)
- ✅ Set up Traefik with automatic SSL certificates
- ✅ Deploy the GitHub MCP Server with monitoring
- ✅ Create systemd service for automatic startup
- ✅ Configure log rotation and system monitoring

### 🔧 Manual Installation (Advanced)

If you prefer manual setup or need customization:

```bash
# Clone repository
git clone https://github.com/your-username/github-mcp-server.git
cd github-mcp-server

# Set up secrets and environment
./docker/scripts/setup-secrets.sh

# Edit configuration
nano docker/.env.production

# Deploy services
docker network create traefik-public
docker-compose -f docker/docker-compose.prod.yml up -d
```

### 4. Authenticate

1. Visit `https://your-domain.com`
2. Click "Connect with GitHub"
3. Complete OAuth flow
4. Copy your session token

### 5. Configure Claude/Cursor

Add the MCP server to your Claude/Cursor configuration:

```json
{
  "mcpServers": {
    "github-workflow": {
      "command": "mcp-client",
      "args": ["--server", "https://your-domain.com/mcp"],
      "env": {
        "GITHUB_MCP_TOKEN": "your-session-token"
      }
    }
  }
}
```

## 🔧 Workflow Commands

### `push` - Intelligent Git Push

```bash
# Basic push with branch detection
push

# Push with commit message
push --message "Fix authentication bug"

# Push and mark PR as ready for review
push --ready-for-review
```

**Features:**
- ✅ Detects current branch vs main branch
- ✅ Warns before pushing to main
- ✅ Auto-commits uncommitted changes
- ✅ Updates existing PRs
- ✅ Marks PRs ready for review

### `scan tasks` - GitHub Projects Integration

```bash
# Scan all tasks
scan tasks

# Filter by type
scan tasks --type bug

# Filter by status
scan tasks --status "In Progress"
```

**Features:**
- ✅ Fetches GitHub Project tasks via GraphQL
- ✅ Organizes by priority (Critical, High, Medium, Low)
- ✅ Groups by type (🐛 bug, ✨ feature, 🚀 enhancement)
- ✅ Shows assignees and recent activity
- ✅ Auto-detects project number from TODO.md

### `merge` - Complete Merge Workflow

```bash
# Complete merge with cleanup
merge

# Merge without deleting branch
merge --keep-branch

# Merge with work folder cleanup
merge --cleanup-folder
```

**Features:**
- ✅ Runs final tests before merge
- ✅ Merges PR via GitHub API
- ✅ Switches back to main and pulls latest
- ✅ Cleans up work folders
- ✅ Updates GitHub Project status to "Done"
- ✅ Provides complete audit trail

## 🛡️ Security Features

### Authentication & Authorization
- **OAuth 2.0** with GitHub for secure authentication
- **JWT tokens** with configurable expiration
- **Encrypted token storage** using industry-standard encryption
- **CSRF protection** for all OAuth flows

### Network Security
- **TLS 1.3** encryption for all communications
- **HSTS headers** with preload for enhanced security
- **Content Security Policy** to prevent XSS attacks
- **Rate limiting** to prevent abuse and DoS attacks

### Application Security
- **Input validation** and sanitization for all user inputs
- **SQL injection protection** with parameterized queries
- **Audit logging** for all security-relevant events
- **Secure headers** (X-Frame-Options, X-Content-Type-Options, etc.)

### Infrastructure Security
- **Traefik reverse proxy** with automatic SSL certificate management
- **Fail2ban integration** for intrusion prevention
- **Docker security** with non-root users and minimal attack surface
- **Regular security updates** with automated dependency scanning

## 📊 Monitoring & Observability

### Health Checks
- `/health` endpoint with detailed system status
- Container health checks with automatic restart
- Database connection monitoring
- GitHub API connectivity checks

### Metrics & Logging
- **Structured logging** with configurable levels
- **Audit trails** for all user actions
- **Performance metrics** via Prometheus (optional)
- **Error tracking** with detailed stack traces

### Alerting
- Rate limit violations
- Authentication failures
- API errors and timeouts
- Certificate expiration warnings

## 🔧 Configuration

### Server Configuration (`config/server.toml`)

```toml
[server]
host = "0.0.0.0"
port = 8443
workers = 4

[security]
rate_limit_requests_per_minute = 60
session_timeout_hours = 24
audit_log_enabled = true

[github]
api_timeout = 30
max_retries = 3
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GITHUB_CLIENT_ID` | GitHub OAuth App Client ID | Required |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth App Client Secret | Required |
| `JWT_SECRET` | Secret key for JWT token signing | Required |
| `DATABASE_URL` | SQLite database file path | `sqlite:./data/github-mcp-server.db` |
| `RATE_LIMIT_RPM` | Requests per minute limit | `60` |
| `AUDIT_LOG_ENABLED` | Enable audit logging | `true` |

## 🚀 Deployment

### Production Deployment

1. **Server Setup**
   ```bash
   # Install Docker and Docker Compose
   curl -fsSL https://get.docker.com | sh
   sudo usermod -aG docker $USER
   ```

2. **Domain Configuration**
   - Point your domain to the server IP
   - Ensure ports 80 and 443 are open

3. **Deploy**
   ```bash
   git clone https://github.com/your-username/github-mcp-server.git
   cd github-mcp-server
   cp .env.example .env
   # Edit .env with your configuration
   docker-compose -f docker/docker-compose.yml up -d
   ```

4. **Verify Deployment**
   ```bash
   curl -k https://your-domain.com/health
   ```

### Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/your-username/github-mcp-server.git
cd github-mcp-server

# Install dependencies
cargo build

# Run development server
cargo run

# Run tests
cargo test
```

## 🧪 Testing

### Unit Tests
```bash
cargo test
```

### Integration Tests
```bash
cargo test --test integration
```

### Security Tests
```bash
# Run security audit
cargo audit

# Check for vulnerabilities
cargo deny check
```

## 📚 Documentation

- [API Documentation](docs/API.md)
- [Security Architecture](docs/SECURITY.md)
- [Deployment Guide](docs/DEPLOYMENT.md)
- [Workflow Diagrams](docs/WORKFLOW_DIAGRAM.md)
- [Contributing Guide](CONTRIBUTING.md)

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Rust Programming Language](https://www.rust-lang.org/)
- [Axum Web Framework](https://github.com/tokio-rs/axum)
- [Traefik Reverse Proxy](https://traefik.io/)
- [Model Context Protocol](https://modelcontextprotocol.io/)
- [GitHub API](https://docs.github.com/en/rest)

## 📞 Support

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/your-username/github-mcp-server/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/your-username/github-mcp-server/discussions)
- 📧 **Security Issues**: security@your-domain.com

---

**Built with ❤️ and 🦀 Rust for maximum security and performance**