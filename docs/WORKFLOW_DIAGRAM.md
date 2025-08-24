# GitHub MCP Server Workflow Diagram

## Architecture Overview

```mermaid
graph TB
    subgraph "Client Layer"
        C1[Claude/Cursor Client]
        C2[Web Browser]
    end
    
    subgraph "Security Layer"
        TF[Traefik Reverse Proxy]
        LE[Let's Encrypt SSL]
        F2B[Fail2ban]
    end
    
    subgraph "Application Layer"
        MCP[GitHub MCP Server]
        AUTH[OAuth 2.0 Handler]
        API[GitHub API Client]
    end
    
    subgraph "Data Layer"
        DB[(SQLite Database)]
        LOGS[Audit Logs]
    end
    
    subgraph "External Services"
        GH[GitHub API]
        GP[GitHub Projects]
        GR[Git Repositories]
    end
    
    C1 -->|MCP Protocol| TF
    C2 -->|HTTPS| TF
    TF -->|SSL Termination| MCP
    LE -->|Certificates| TF
    F2B -->|IP Blocking| TF
    
    MCP --> AUTH
    MCP --> API
    AUTH --> DB
    API --> GH
    API --> GP
    
    MCP --> LOGS
    LOGS --> DB
    
    GH --> GR
    GP --> GH
```

## Command Flow Diagrams

### 1. Push Command Workflow

```mermaid
sequenceDiagram
    participant C as Claude/Cursor
    participant M as MCP Server
    participant G as Git
    participant GH as GitHub API
    participant GP as GitHub Projects
    
    C->>M: push command
    M->>M: Validate authentication
    M->>G: Get current branch
    
    alt On main branch
        M->>C: Warning: pushing to main
        C->>M: User confirmation
    end
    
    M->>G: Check uncommitted changes
    
    alt Has uncommitted changes
        M->>G: git add . && git commit
    end
    
    M->>G: git push origin branch
    M->>GH: Check for existing PR
    
    alt PR exists
        M->>GH: Update PR status
        opt Ready for review
            M->>GH: Mark PR ready
        end
    else No PR
        M->>C: Suggest creating PR
    end
    
    M->>GP: Update project task status
    M->>C: Success response with PR info
```

### 2. Scan Tasks Command Workflow

```mermaid
sequenceDiagram
    participant C as Claude/Cursor
    participant M as MCP Server
    participant GP as GitHub Projects
    participant TODO as TODO.md
    
    C->>M: scan tasks command
    M->>M: Validate authentication
    
    alt Project number provided
        M->>M: Use provided number
    else No project number
        M->>TODO: Read project number from TODO.md
        alt Not found in TODO.md
            M->>C: Request project number
        end
    end
    
    M->>GP: Fetch project items via GraphQL
    M->>M: Filter by type/status/priority
    M->>M: Organize tasks by categories
    
    M->>C: Present organized task list
    C->>M: User selects task
    M->>M: Create development environment
    M->>TODO: Update with "In Progress" status
```

### 3. Merge Command Workflow

```mermaid
sequenceDiagram
    participant C as Claude/Cursor
    participant M as MCP Server
    participant G as Git
    participant GH as GitHub API
    participant GP as GitHub Projects
    
    C->>M: merge command
    M->>M: Validate authentication
    M->>G: Get current branch
    
    alt On main branch
        M->>C: Error: already on main
        Note over M,C: Exit workflow
    end
    
    M->>G: Check uncommitted changes
    
    alt Has changes
        M->>G: git add . && git commit
    end
    
    M->>G: git push origin branch
    M->>GH: Get PR for branch
    M->>GH: Mark PR as ready
    M->>M: Run tests (configurable)
    
    alt Tests pass
        M->>GH: Merge PR
        M->>G: git checkout main
        M->>G: git pull origin main
        
        opt Delete branch
            M->>G: git branch -d branch
            M->>GH: Delete remote branch
        end
        
        opt Cleanup work folder
            M->>M: Remove work/issue-X/ folder
        end
        
        M->>GP: Update project task to "Done"
        M->>C: Success with completion summary
    else Tests fail
        M->>C: Error: tests failed
    end
```

## Security Flow

```mermaid
graph TB
    subgraph "Request Flow"
        REQ[Incoming Request]
        RATE[Rate Limiting]
        AUTH[Authentication]
        AUTHZ[Authorization]
        PROC[Process Request]
        AUDIT[Audit Logging]
        RESP[Response]
    end
    
    REQ --> RATE
    RATE -->|Pass| AUTH
    RATE -->|Fail| BLOCK[Block Request]
    
    AUTH -->|Valid Token| AUTHZ
    AUTH -->|Invalid Token| UNAUTH[401 Unauthorized]
    
    AUTHZ -->|Authorized| PROC
    AUTHZ -->|Forbidden| FORBID[403 Forbidden]
    
    PROC --> AUDIT
    AUDIT --> RESP
    
    BLOCK --> AUDIT
    UNAUTH --> AUDIT
    FORBID --> AUDIT
```

## Authentication Flow

```mermaid
sequenceDiagram
    participant U as User
    participant B as Browser
    participant M as MCP Server
    participant GH as GitHub OAuth
    participant DB as Database
    
    U->>B: Visit /auth/github
    B->>M: GET /auth/github
    M->>M: Generate CSRF token
    M->>DB: Store CSRF token
    M->>B: Redirect to GitHub OAuth
    B->>GH: OAuth authorization request
    
    GH->>U: Login prompt
    U->>GH: Enter credentials
    GH->>B: Redirect with auth code
    B->>M: GET /auth/github/callback?code=X&state=Y
    
    M->>DB: Validate CSRF token
    M->>GH: Exchange code for token
    GH->>M: Access token + refresh token
    M->>GH: Get user info
    GH->>M: User profile data
    
    M->>DB: Store encrypted tokens
    M->>M: Generate JWT session token
    M->>B: Success page with JWT
    B->>U: Display success + JWT token
```

## Data Flow Architecture

```mermaid
graph LR
    subgraph "Input Sources"
        MCP_REQ[MCP Requests]
        WEB_REQ[Web Requests]
        WEBHOOK[GitHub Webhooks]
    end
    
    subgraph "Processing Pipeline"
        VALIDATE[Request Validation]
        AUTHENTICATE[Authentication]
        AUTHORIZE[Authorization]
        EXECUTE[Command Execution]
    end
    
    subgraph "External Integrations"
        GH_API[GitHub API]
        GH_PROJECTS[GitHub Projects]
        GIT_OPS[Git Operations]
    end
    
    subgraph "Data Storage"
        TOKENS[Encrypted Tokens]
        AUDIT[Audit Logs]
        SESSIONS[User Sessions]
        WORKFLOW[Workflow State]
    end
    
    MCP_REQ --> VALIDATE
    WEB_REQ --> VALIDATE
    WEBHOOK --> VALIDATE
    
    VALIDATE --> AUTHENTICATE
    AUTHENTICATE --> AUTHORIZE
    AUTHORIZE --> EXECUTE
    
    EXECUTE --> GH_API
    EXECUTE --> GH_PROJECTS
    EXECUTE --> GIT_OPS
    
    AUTHENTICATE --> TOKENS
    EXECUTE --> AUDIT
    AUTHENTICATE --> SESSIONS
    EXECUTE --> WORKFLOW
```

## Deployment Architecture

```mermaid
graph TB
    subgraph "Internet"
        USER[Users]
        GITHUB[GitHub.com]
    end
    
    subgraph "Edge Layer"
        CDN[CloudFlare CDN]
        DNS[DNS]
    end
    
    subgraph "Server Infrastructure"
        LB[Load Balancer]
        
        subgraph "Docker Host"
            TRAEFIK[Traefik Container]
            MCP_APP[MCP Server Container]
            F2B[Fail2ban Container]
        end
        
        subgraph "Storage"
            DB_VOL[Database Volume]
            LOG_VOL[Logs Volume]
            CERT_VOL[Certificates Volume]
        end
    end
    
    subgraph "Monitoring"
        METRICS[Prometheus]
        ALERTS[Alertmanager]
        GRAFANA[Grafana]
    end
    
    USER --> CDN
    CDN --> DNS
    DNS --> LB
    LB --> TRAEFIK
    
    TRAEFIK --> MCP_APP
    TRAEFIK --> CERT_VOL
    MCP_APP --> DB_VOL
    MCP_APP --> LOG_VOL
    F2B --> LOG_VOL
    
    MCP_APP --> GITHUB
    
    MCP_APP --> METRICS
    METRICS --> ALERTS
    METRICS --> GRAFANA
```

## Draw.io XML Export

To import this into draw.io:

1. Copy the XML content below
2. Go to https://app.diagrams.net/
3. File → Import from → Text
4. Paste the XML content

```xml
<!-- This would contain the actual draw.io XML format -->
<!-- For now, use the mermaid diagrams above as reference -->
<!-- Convert using: https://mermaid.live/ → Export → draw.io -->
```

## Usage Instructions

1. **For Development**: Use the mermaid diagrams directly in your documentation
2. **For Presentations**: Export mermaid diagrams to PNG/SVG
3. **For Collaboration**: Import into draw.io for team editing
4. **For Documentation**: Include in README.md files

## Diagram Updates

When updating workflows:

1. Update the mermaid diagrams in this file
2. Export new versions to draw.io if needed
3. Update any related documentation
4. Commit changes to version control