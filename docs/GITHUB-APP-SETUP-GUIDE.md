# GitHub App Setup Guide

This is a detailed, step-by-step guide for creating a GitHub App for the GitHub MCP Server. Follow these instructions exactly to ensure proper configuration.

## 🎯 Overview

You need to create a **GitHub App** (not OAuth App) that will:
- Authenticate users securely via OAuth 2.0
- Access GitHub repositories and projects
- Receive webhooks for real-time updates
- Provide granular permission control

## 📝 Step-by-Step Instructions

### Step 1: Navigate to GitHub App Creation

1. **Go to GitHub Settings**:
   - Visit: https://github.com/settings/apps
   - Or: GitHub → Settings → Developer settings → GitHub Apps

2. **Click "New GitHub App"**

### Step 2: Basic Information

Fill out the form with these **exact** values:

#### Required Fields:

| Field | Value | Example |
|-------|-------|---------|
| **GitHub App name** | `GitHub MCP Server - [Your Organization]` | `GitHub MCP Server - Acme Corp` |
| **Description** | `Secure GitHub workflow automation for Claude/Cursor via MCP protocol` | (Optional but recommended) |
| **Homepage URL** | `https://[your-domain]` | `https://mcp.yourdomain.com` |

> **Note**: The GitHub App name must be **globally unique** across all of GitHub. Add your organization name to make it unique.

### Step 3: Identifying and Authorizing Users

This section configures OAuth authentication:

#### Callback URL (CRITICAL):
```
https://[your-domain]/auth/github/callback
```

**Example**: `https://mcp.yourdomain.com/auth/github/callback`

> **⚠️ WARNING**: This URL must be **EXACTLY** correct or authentication will fail!

#### Checkboxes to Enable:
- ☑️ **Request user authorization (OAuth) during installation** - **REQUIRED**
- ☑️ **Enable Device Flow** - **RECOMMENDED** (better user experience)
- ☐ **Expire user authorization tokens** - **LEAVE UNCHECKED** (for now)

### Step 4: Post Installation Setup

| Field | Value | Notes |
|-------|-------|-------|
| **Setup URL (optional)** | `https://[your-domain]/setup` | Helps users after installation |
| ☑️ **Redirect on update** | **CHECKED** | Recommended |

### Step 5: Webhook Configuration

Configure webhooks for real-time updates:

| Field | Value | Example |
|-------|-------|---------|
| ☑️ **Active** | **CHECKED** | Enable webhooks |
| **Webhook URL** | `https://[your-domain]/webhooks/github` | `https://mcp.yourdomain.com/webhooks/github` |
| **Webhook secret** | `[Random 32-character string]` | Generate with: `openssl rand -hex 32` |

> **Security Tip**: Save the webhook secret - you'll need it for configuration.

### Step 6: Repository Permissions

Set these **minimum required permissions**:

| Permission Category | Permission | Access Level | Why Needed |
|-------------------|------------|--------------|------------|
| Repository | **Contents** | **Read & Write** | Read/write repository files and code |
| Repository | **Issues** | **Read & Write** | Create and manage GitHub issues |
| Repository | **Metadata** | **Read** | Basic repository information |
| Repository | **Pull requests** | **Read & Write** | Create and manage pull requests |
| Repository | **Projects** | **Read & Write** | Access GitHub Projects (repository-level) |

### Step 7: Organization Permissions

| Permission | Access Level | Why Needed |
|------------|--------------|------------|
| **Members** | **Read** | Team and member information |
| **Projects** | **Read & Write** | Organization-level GitHub Projects |

### Step 8: User Permissions

| Permission | Access Level | Why Needed |
|------------|--------------|------------|
| **Email addresses** | **Read** | User identification and notifications |

### Step 9: Subscribe to Events

Select these events for real-time webhook notifications:

#### Required Events:
- ☑️ **Issues** - Issue creation, updates, comments
- ☑️ **Pull request** - PR creation, updates, reviews
- ☑️ **Push** - Code push events
- ☑️ **Repository** - Repository changes and settings

#### Recommended Events:
- ☑️ **Project** - GitHub Projects updates
- ☑️ **Installation** - App installation events
- ☑️ **Installation repositories** - Repository access changes

#### Optional Events:
- ☑️ **Release** - Release creation and updates
- ☑️ **Workflow run** - GitHub Actions workflow events

### Step 10: Installation Target

Choose based on your use case:

#### Option A: Personal/Small Team Use
- **"Only on this account"** - Restricts installation to your personal account only

#### Option B: Organization/Enterprise Use  
- **"Any account"** - Allows installation on any GitHub account/organization

> **Recommendation**: Start with "Only on this account" for testing, then change to "Any account" for production use.

### Step 11: Create the App

1. **Review all settings** carefully
2. **Click "Create GitHub App"**
3. **You'll be redirected to the app's settings page**

## 🔑 After Creation - Collect Credentials

After creating the app, collect these credentials:

### From the General Tab:

1. **App ID**: Found at the top of the settings page
   - Example: `123456`

2. **Client ID**: Found in the "OAuth credentials" section
   - Example: `Iv1.a1b2c3d4e5f6g7h8`

3. **Client Secret**: 
   - Click "Generate a new client secret"
   - **Copy immediately** - you won't see it again!
   - Example: `1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t`

4. **Private Key** (for advanced use):
   - Click "Generate a private key"
   - Downloads a `.pem` file
   - Store securely

### Save These Values:

```bash
# GitHub App Configuration
GITHUB_APP_ID=123456
GITHUB_CLIENT_ID=Iv1.a1b2c3d4e5f6g7h8
GITHUB_CLIENT_SECRET=1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t
WEBHOOK_SECRET=a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6
```

## 🏢 Organization Installation

If you're using this in an organization:

### Step 1: Install the App
1. **Go to your organization**: `https://github.com/[org-name]`
2. **Settings → Third-party Access → GitHub Apps**
3. **Click "Install App"** next to your GitHub App
4. **Choose repository access**:
   - **All repositories** (easiest)
   - **Selected repositories** (more secure)

### Step 2: Grant Permissions
1. **Review requested permissions**
2. **Click "Install"**
3. **Organization owners may need to approve**

## ✅ Verification Checklist

Before proceeding with installation, verify:

- [ ] **App created successfully** with unique name
- [ ] **Callback URL** is exactly: `https://[your-domain]/auth/github/callback`
- [ ] **OAuth enabled** during installation
- [ ] **Required permissions** granted (Contents, Issues, PRs, Projects)
- [ ] **Webhook URL** configured: `https://[your-domain]/webhooks/github`
- [ ] **Client ID** starts with `Iv1.`
- [ ] **Client Secret** is 40 characters long
- [ ] **App installed** on target organization/account
- [ ] **Credentials saved** securely

## 🔧 Configuration for Installation Script

When running the GitHub MCP Server installation script, you'll need:

```bash
# From your GitHub App:
GITHUB_CLIENT_ID=Iv1.a1b2c3d4e5f6g7h8
GITHUB_CLIENT_SECRET=1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t
```

## 🚨 Common Mistakes to Avoid

### ❌ Wrong App Type
- **Don't create** an "OAuth App" - you need a "GitHub App"
- GitHub Apps provide better security and permissions

### ❌ Incorrect Callback URL
- Must be **exactly**: `https://[your-domain]/auth/github/callback`
- No trailing slashes, no typos
- Must match your actual domain

### ❌ Missing OAuth Enable
- **Must check** "Request user authorization (OAuth) during installation"
- Without this, users can't authenticate

### ❌ Insufficient Permissions
- Need **Read & Write** for Contents, Issues, PRs, Projects
- **Read** permission is not enough for workflow automation

### ❌ Forgetting Installation
- Creating the app is not enough
- Must **install** the app on your organization/account

## 🆘 Troubleshooting

### Problem: "App not found" during authentication
**Solution**: Verify the Client ID is correct and the app is installed on the target organization.

### Problem: "Invalid callback URL" error
**Solution**: Double-check the callback URL in your GitHub App matches exactly: `https://[your-domain]/auth/github/callback`

### Problem: "Insufficient permissions" errors
**Solution**: Review and update the app permissions, then reinstall the app.

### Problem: Organization blocks the app
**Solution**: Work with organization administrators to approve the app installation. See [policies documentation](policies-and-governance.md).

## 🔄 Next Steps

After completing GitHub App setup:

1. **Continue with** [Installation Prerequisites](INSTALLATION-PREREQUISITES.md)
2. **Run the installation script** with your GitHub App credentials
3. **Test authentication** by visiting your domain
4. **Configure Claude/Cursor** with your MCP server

## 📚 Additional Resources

- [GitHub Apps Documentation](https://docs.github.com/en/apps)
- [GitHub App Permissions Reference](https://docs.github.com/en/apps/creating-github-apps/registering-a-github-app/choosing-permissions-for-a-github-app)
- [OAuth 2.0 Flow Documentation](https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app)
- [Webhook Events Reference](https://docs.github.com/en/webhooks/webhook-events-and-payloads)

---

**Need help?** Open an issue on the GitHub MCP Server repository with your specific problem and we'll help you troubleshoot.