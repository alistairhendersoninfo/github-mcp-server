# Network Architecture - All Traffic Through Traefik

## 🔄 **Complete Traffic Flow**

**EVERYTHING goes through Traefik** - this is the core security principle of our architecture.

```
Internet Users
     ↓ (HTTPS Port 443 ONLY)
┌─────────────────────────────────────────────────────────────────┐
│                        🛡️ TRAEFIK REVERSE PROXY                │
│                                                                 │
│  • SSL Termination (Let's Encrypt + AWS Route 53)             │
│  • Request Routing by Host Headers                             │
│  • Security Headers & Rate Limiting                            │
│  • Authentication & Authorization                              │
│  • Load Balancing & Health Checks                             │
│                                                                 │
│  ONLY CONTAINER WITH EXTERNAL ACCESS                           │
└─────────────────────────────────────────────────────────────────┘
     ↓ (Internal Docker Network - github-mcp-network)
┌─────────────────────────────────────────────────────────────────┐
│                    🔒 INTERNAL SERVICES                        │
│                   (NO EXTERNAL ACCESS)                         │
│                                                                 │
│  your-domain.com          → MCP Server (8443)                 │
│  traefik.your-domain.com  → Traefik Dashboard (8080)          │
│  dashboard.your-domain.com → Grafana (3000)                   │
│  metrics.your-domain.com  → Prometheus (9090)                 │
│  static.your-domain.com   → Nginx (80)                        │
│                                                                 │
│  All services isolated in Docker network                       │
│  No direct internet access possible                            │
└─────────────────────────────────────────────────────────────────┘
```

## 🔒 **Security Benefits**

### **Single Point of Entry:**
- ✅ **Only Traefik exposed** to internet (ports 80, 443)
- ✅ **All other services internal** (Docker network isolation)
- ✅ **No direct service access** from internet
- ✅ **Centralized security policies** at Traefik level

### **Defense in Depth:**
1. **Firewall Level**: UFW blocks all except 80, 443, SSH
2. **Traefik Level**: SSL, rate limiting, authentication
3. **Network Level**: Docker network isolation
4. **Container Level**: Non-root users, security policies
5. **Application Level**: Input validation, audit logging

## 📋 **Routing Configuration**

### **Host-Based Routing:**
```yaml
# Main application
your-domain.com → github-mcp-server:8443

# Admin interfaces (with authentication)
traefik.your-domain.com → traefik:8080
dashboard.your-domain.com → grafana:3000
metrics.your-domain.com → prometheus:9090

# Static assets (optional)
static.your-domain.com → nginx:80
```

### **Path-Based Routing (Alternative):**
```yaml
# All on single domain
your-domain.com/ → github-mcp-server:8443
your-domain.com/traefik → traefik:8080
your-domain.com/dashboard → grafana:3000
your-domain.com/metrics → prometheus:9090
your-domain.com/static → nginx:80
```

## 🛡️ **Traefik Security Features**

### **SSL/TLS Security:**
```yaml
# Automatic HTTPS redirect
entryPoints:
  web:
    address: ":80"
    http:
      redirections:
        entrypoint:
          to: websecure
          scheme: https

# Strong TLS configuration
tls:
  options:
    default:
      minVersion: "VersionTLS12"
      maxVersion: "VersionTLS13"
      cipherSuites:
        - "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384"
        - "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305"
```

### **Security Headers:**
```yaml
# Applied to all services
security-headers:
  headers:
    customResponseHeaders:
      X-Content-Type-Options: "nosniff"
      X-Frame-Options: "DENY"
      X-XSS-Protection: "1; mode=block"
      Strict-Transport-Security: "max-age=31536000; includeSubDomains; preload"
      Content-Security-Policy: "default-src 'self'"
```

### **Rate Limiting:**
```yaml
# Per-service rate limiting
rate-limit:
  rateLimit:
    average: 100    # requests per minute
    burst: 50       # burst capacity
```

### **Authentication:**
```yaml
# Basic auth for admin interfaces
admin-auth:
  basicAuth:
    users:
      - "admin:$2y$10$..."  # Secure bcrypt hash
```

## 🔧 **Network Configuration**

### **Docker Networks:**
```yaml
networks:
  github-mcp-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16
  traefik-public:
    external: true
```

### **Service Network Assignment:**
- **Traefik**: `github-mcp-network` + `traefik-public`
- **All other services**: `github-mcp-network` ONLY

### **Port Exposure:**
```yaml
# ONLY Traefik exposes ports
traefik:
  ports:
    - "80:80"     # HTTP (redirects to HTTPS)
    - "443:443"   # HTTPS (main entry point)

# All other services: NO PORTS EXPOSED
github-mcp-server:
  # ports: []  # NO EXTERNAL PORTS
  
nginx:
  # ports: []  # NO EXTERNAL PORTS
  
prometheus:
  # ports: []  # NO EXTERNAL PORTS
```

## 🔍 **Traffic Inspection**

### **Request Flow Monitoring:**
```bash
# View Traefik access logs
docker logs github-mcp-traefik | grep -E "(GET|POST|PUT|DELETE)"

# Monitor specific service routing
docker logs github-mcp-traefik | grep "github-mcp-server"

# Check SSL certificate status
curl -I https://your-domain.com
```

### **Network Connectivity Testing:**
```bash
# Test internal service connectivity (from within network)
docker exec github-mcp-traefik curl http://github-mcp-server:8443/health

# Test external connectivity (should fail for internal services)
curl http://server-ip:8443  # Should timeout/fail
curl http://server-ip:3000  # Should timeout/fail
```

## 🚨 **Security Validation**

### **Verify No Direct Access:**
```bash
# These should all FAIL (timeout or connection refused)
curl http://your-server-ip:8443    # MCP Server
curl http://your-server-ip:3000    # Grafana
curl http://your-server-ip:9090    # Prometheus
curl http://your-server-ip:80      # Nginx

# Only these should work
curl https://your-domain.com                # ✅ Via Traefik
curl https://dashboard.your-domain.com      # ✅ Via Traefik
```

### **Port Scan Verification:**
```bash
# From external machine, scan your server
nmap -p 1-65535 your-server-ip

# Should only show:
# 22/tcp   open  ssh
# 80/tcp   open  http     (Traefik - redirects to HTTPS)
# 443/tcp  open  https    (Traefik - main entry point)
```

## 🔧 **Troubleshooting**

### **Service Not Accessible:**
1. **Check Traefik routing:**
   ```bash
   docker logs github-mcp-traefik | grep "your-domain.com"
   ```

2. **Verify service health:**
   ```bash
   docker exec github-mcp-traefik curl http://github-mcp-server:8443/health
   ```

3. **Check Traefik dashboard:**
   ```bash
   # Visit https://traefik.your-domain.com
   # Look for service status and routing rules
   ```

### **SSL Certificate Issues:**
```bash
# Check certificate status
openssl s_client -connect your-domain.com:443 -servername your-domain.com

# Check Traefik ACME logs
docker logs github-mcp-traefik | grep -i acme
```

### **Network Connectivity Issues:**
```bash
# Check Docker networks
docker network ls
docker network inspect github-mcp-network

# Test internal connectivity
docker exec github-mcp-server ping github-mcp-traefik
```

## 📊 **Monitoring Traffic Flow**

### **Traefik Metrics:**
- Request count per service
- Response times
- Error rates
- SSL certificate status

### **Access Logs:**
- All requests logged with source IP
- Response codes and timing
- User agent and referrer information
- Security event detection

### **Health Checks:**
- Automatic service health monitoring
- Failed service detection and alerting
- Load balancing based on health status

---

## 🎯 **Key Takeaways**

1. **EVERYTHING goes through Traefik** - no exceptions
2. **No direct service access** from internet
3. **Single SSL termination point** at Traefik
4. **Centralized security policies** and monitoring
5. **Network isolation** via Docker networks
6. **Defense in depth** at multiple layers

This architecture ensures **maximum security** while maintaining **ease of management** and **scalability**.