# GitHub MCP Server - Installation Prerequisites

🚨 **IMPORTANT: READ THIS COMPLETELY BEFORE RUNNING THE INSTALLATION SCRIPT** 🚨

This guide explains everything you need to prepare **before** running the `install.sh` script. The installation will ask for various credentials and configurations, so having everything ready will make the process smooth and secure.

## 📋 Quick Checklist

Before starting installation, ensure you have:

- [ ] **Domain name** pointed to your server's IP address
- [ ] **AWS Route 53** hosting your domain's DNS
- [ ] **AWS IAM credentials** with Route 53 permissions
- [ ] **GitHub App** created and configured (detailed steps below)
- [ ] **Email address** for Let's Encrypt certificates
- [ ] **SSH access configuration** planned
- [ ] **Server requirements** met (Ubuntu/Debian/CentOS/RHEL/Fedora)

## 🌐 Domain & DNS Requirements

### What You Need:
1. **A domain name** (e.g., `mcp.yourdomain.com`)
2. **Domain hosted on AWS Route 53** (required for automatic SSL certificates)
3. **DNS A record** pointing your domain to your server's public IP

### Why AWS Route 53?
The installation uses **DNS-01 challenge** for Let's Encrypt SSL certificates, which allows:
- ✅ Automatic certificate renewal
- ✅ Wildcard certificates for subdomains
- ✅ Works behind firewalls/NAT
- ✅ No need to expose port 80 during renewal

### Setup Steps:
1. **Transfer domain to Route 53** (if not already there):
   - Go to AWS Route 53 → Hosted Zones
   - Create hosted zone for your domain
   - Update nameservers at your domain registrar

2. **Create DNS A record**:
   - In Route 53 hosted zone, create A record
   - Name: `mcp` (or whatever subdomain you want)
   - Value: Your server's public IP address
   - TTL: 300 seconds

## 🔑 AWS Credentials Setup

### Required IAM Permissions:
Create an IAM user with **Route 53 DNS permissions** for certificate management:

```json
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
```

### Setup Steps:
1. **Go to AWS IAM Console**
2. **Create new user**: `github-mcp-dns`
3. **Attach policy**: Create custom policy with above permissions
4. **Create access keys**: Save Access Key ID and Secret Access Key
5. **Note the AWS region** where your Route 53 zone is hosted

### What the installer will ask for:
- `AWS Access Key ID`: `AKIA...`
- `AWS Secret Access Key`: `wJalrXUt...`
- `AWS Region`: `us-east-1` (or your region)

## 🐙 GitHub App Creation (CRITICAL STEP)

This is the most complex part. You need to create a **GitHub App** (not OAuth App) for secure authentication.

### 📖 Detailed Setup Guide

**For complete step-by-step instructions with examples, see:**
👉 **[GitHub App Setup Guide](GITHUB-APP-SETUP-GUIDE.md)**

This detailed guide includes:
- ✅ Exact form field values to use
- ✅ Permission settings with explanations
- ✅ Common mistakes to avoid
- ✅ Troubleshooting tips
- ✅ Verification checklist

### 🚀 Quick Summary

If you're experienced with GitHub Apps, here's the quick version:

1. **Create GitHub App** at: `https://github.com/settings/apps/new`
2. **Basic Info**:
   - Name: `GitHub MCP Server - [Your Org]`
   - Homepage: `https://mcp.yourdomain.com`
   - Callback: `https://mcp.yourdomain.com/auth/github/callback` ⚠️ **EXACT**
3. **Enable OAuth** during installation ✅
4. **Permissions**: Contents, Issues, PRs, Projects (Read/Write)
5. **Webhooks**: `https://mcp.yourdomain.com/webhooks/github`
6. **Install** the app on your organization/account

### 🔑 What You'll Need for Installation

After creating the GitHub App, collect these values:

- **Client ID**: `Iv1.a1b2c3d4e5f6g7h8` (starts with `Iv1.`)
- **Client Secret**: `1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t` (40 characters)

> **⚠️ IMPORTANT**: The callback URL must be **exactly**: `https://mcp.yourdomain.com/auth/github/callback`

## 🔒 Security Configuration Planning

### SSH Access Planning
The installer will ask which IP addresses should have SSH access. Plan this in advance:

**Examples:**
- `192.168.1.0/24` - Your home network
- `10.0.0.0/8` - Your office VPN
- `203.0.113.0/24` - Your office public IP range
- `0.0.0.0/0` - **NOT RECOMMENDED** - Allows SSH from anywhere

**Security Best Practice:**
- Only allow SSH from known, trusted networks
- Use VPN if you need remote access
- Consider using SSH key authentication only

### Email for SSL Certificates
You'll need an email address for Let's Encrypt notifications:
- Certificate expiration warnings
- Important security updates
- Rate limit notifications

## 🖥️ Server Requirements

### Minimum System Requirements:
- **OS**: Ubuntu 20.04+, Debian 11+, CentOS 8+, RHEL 8+, or Fedora 35+
- **RAM**: 2GB minimum, 4GB recommended
- **Storage**: 20GB minimum, 50GB recommended
- **CPU**: 2 cores minimum
- **Network**: Public IP with ports 80 and 443 accessible

### Required Ports:
- **Port 80** (HTTP) - Let's Encrypt challenges and HTTP→HTTPS redirects
- **Port 443** (HTTPS) - Main application access
- **Port 22** (SSH) - Administrative access (restricted by IP)

### Pre-installation Commands:
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install curl (if not present)
sudo apt install -y curl

# Check available disk space
df -h

# Check memory
free -h

# Verify domain resolution
nslookup mcp.yourdomain.com
```

## 📝 Installation Information Summary

Before running `sudo ./install.sh`, gather this information:

### Domain Configuration:
- **Domain name**: `mcp.yourdomain.com`
- **Email for SSL**: `admin@yourdomain.com`
- **AWS Region**: `us-east-1`

### AWS Credentials:
- **Access Key ID**: `AKIA...`
- **Secret Access Key**: `wJalrXUt...`

### GitHub App:
- **Client ID**: `Iv1.a1b2c3d4e5f6g7h8`
- **Client Secret**: `1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t`

### SSH Access:
- **Allowed IPs**: `192.168.1.0/24, 10.0.0.0/8`

### System:
- **Timezone**: `UTC` (or your preferred timezone)

## 🚀 Ready to Install?

Once you have all the above information ready:

1. **Download the installer**:
   ```bash
   curl -fsSL https://raw.githubusercontent.com/your-username/github-mcp-server/main/install.sh -o install.sh
   chmod +x install.sh
   ```

2. **Run the installation**:
   ```bash
   sudo ./install.sh
   ```

3. **Follow the prompts** with the information you gathered above

## ❓ Common Questions

### Q: Can I use a different DNS provider?
**A:** Currently, the automated installer only supports AWS Route 53 for DNS challenges. You can manually configure other providers, but it requires modifying the Traefik configuration.

### Q: Do I need a GitHub App or OAuth App?
**A:** You need a **GitHub App** (not OAuth App). GitHub Apps provide better security and granular permissions.

### Q: Can I change these settings after installation?
**A:** Yes, but you'll need to update configuration files and restart services. It's easier to get them right the first time.

### Q: What if my organization blocks GitHub Apps?
**A:** Work with your GitHub organization administrators to approve the GitHub App installation. See the [policies documentation](docs/policies-and-governance.md) for details.

### Q: Can I install this on a local server?
**A:** Yes, but you'll still need a public domain name and Route 53 for SSL certificates. Consider using a subdomain specifically for this purpose.

## 🆘 Troubleshooting

### Domain Issues:
```bash
# Test DNS resolution
nslookup mcp.yourdomain.com
dig mcp.yourdomain.com

# Test connectivity
curl -I http://mcp.yourdomain.com
```

### AWS Issues:
```bash
# Test AWS credentials
aws route53 list-hosted-zones --region us-east-1
```

### GitHub App Issues:
- Verify callback URL is exactly: `https://mcp.yourdomain.com/auth/github/callback`
- Check that OAuth is enabled during installation
- Ensure required permissions are granted

---

## 🔄 Next Steps

After completing the prerequisites:
1. Run the installation script
2. Follow the [post-installation guide](docs/POST-INSTALLATION.md)
3. Configure your Claude/Cursor client
4. Test the workflow commands

**Need help?** Check the [troubleshooting guide](docs/TROUBLESHOOTING.md) or open an issue on GitHub.