# Grafana Dashboards - Complete Monitoring Suite

## 📊 **Dashboard Overview**

I've created **comprehensive Grafana dashboards** that automatically monitor all aspects of your GitHub MCP Server:

### **🎯 Available Dashboards**

1. **📋 GitHub MCP Server - Overview**
   - Service status and health
   - Request rates and response times
   - HTTP status code distribution
   - Real-time traffic monitoring

2. **🔗 GitHub MCP Server - API Metrics**
   - GitHub API usage and rate limits
   - MCP command execution statistics
   - API response times and errors
   - Workflow command tracking

3. **💻 GitHub MCP Server - System Resources**
   - Container CPU and memory usage
   - Network I/O and disk usage
   - Resource utilization trends
   - Performance bottleneck detection

## 🚀 **Automatic Setup**

### **✅ What's Already Configured:**

1. **Auto-Provisioning**: Dashboards automatically load on startup
2. **Data Sources**: Prometheus and Loki pre-configured
3. **Metrics Collection**: Built into the Rust MCP server
4. **Alerting Rules**: 15+ alert conditions configured
5. **Real-time Updates**: 5-second refresh intervals

### **📈 Metrics Collected:**

```rust
// HTTP Request Metrics
http_requests_total              // Total requests by method/path/status
http_request_duration_seconds    // Request latency distribution

// GitHub API Metrics  
github_api_requests_total        // API calls by endpoint/method
github_api_request_duration_seconds  // GitHub API response times
github_api_rate_limit_remaining  // Current rate limit status

// MCP Command Metrics
mcp_commands_total              // Commands by type (push/scan/merge)
mcp_command_duration_seconds    // Command execution times

// System Metrics
active_connections              // Current active connections
database_connections           // Database connection pool usage
```

## 📊 **Dashboard Details**

### **1. Overview Dashboard**

**Purpose**: High-level service health monitoring

**Key Panels**:
- 🟢 **Service Status**: Real-time up/down status for all services
- 📈 **Request Rate**: Requests per second by service
- ⏱️ **Response Times**: 50th and 95th percentile latencies
- 🚦 **Status Codes**: 2xx success, 4xx client errors, 5xx server errors

**Use Cases**:
- Quick health check of entire system
- Identify traffic spikes or drops
- Spot error rate increases
- Monitor overall system performance

### **2. API Metrics Dashboard**

**Purpose**: Detailed GitHub API and MCP command monitoring

**Key Panels**:
- 🔢 **Rate Limit Gauge**: GitHub API calls remaining (critical at <500)
- 📊 **API Requests**: GitHub API usage by endpoint
- ⚡ **MCP Commands**: Usage of push/scan tasks/merge commands
- ⏰ **API Response Times**: GitHub API latency tracking

**Use Cases**:
- Monitor GitHub API quota usage
- Track MCP command popularity
- Identify slow GitHub API endpoints
- Plan API usage optimization

### **3. System Resources Dashboard**

**Purpose**: Container and system resource monitoring

**Key Panels**:
- 🖥️ **CPU Usage**: Per-container CPU utilization
- 💾 **Memory Usage**: Container memory consumption
- 🌐 **Network I/O**: Inbound/outbound traffic
- 💿 **Disk Usage**: Storage utilization by container

**Use Cases**:
- Identify resource bottlenecks
- Plan capacity scaling
- Monitor container health
- Detect resource leaks

## 🚨 **Alerting System**

### **📢 Alert Categories:**

#### **🔴 Critical Alerts (Immediate Action Required)**
- **Service Down**: Any core service offline for >1 minute
- **SSL Certificate Expiring**: <7 days until expiration
- **GitHub Rate Limit Critical**: <100 API calls remaining

#### **🟡 Warning Alerts (Monitor Closely)**
- **High Error Rate**: >10% of requests returning 5xx errors
- **High Response Time**: 95th percentile >2 seconds
- **Resource Usage High**: CPU/Memory/Disk >80%
- **GitHub Rate Limit Low**: <500 API calls remaining

#### **🔵 Info Alerts (Awareness)**
- **Rate Limiting Triggered**: 429 responses detected
- **Authentication Failures**: Unusual 401 response patterns

### **📧 Alert Delivery**

Alerts can be configured to send to:
- **Email**: SMTP configuration
- **Slack**: Webhook integration
- **Discord**: Webhook integration
- **PagerDuty**: Critical incident management
- **Custom Webhooks**: Any HTTP endpoint

## 🔧 **Dashboard Access**

### **🌐 URLs:**
- **Grafana**: `https://dashboard.your-domain.com`
- **Prometheus**: `https://metrics.your-domain.com`
- **Traefik**: `https://traefik.your-domain.com`

### **🔐 Credentials:**
```bash
# Grafana Login
Username: admin
Password: [Generated during installation - check /root/.github-mcp-credentials]

# Traefik Dashboard
Username: admin  
Password: [Generated during installation - check /root/.github-mcp-credentials]
```

## 📱 **Mobile-Friendly**

All dashboards are **responsive** and work perfectly on:
- 📱 Mobile phones
- 📱 Tablets  
- 💻 Desktop computers
- 🖥️ Large monitors

## 🎨 **Customization**

### **Adding Custom Panels:**

1. **Access Grafana**: `https://dashboard.your-domain.com`
2. **Edit Dashboard**: Click "Edit" on any dashboard
3. **Add Panel**: Use "Add Panel" button
4. **Query Metrics**: Use Prometheus as data source

### **Available Metrics:**

```promql
# Example queries you can use:

# Request rate by service
rate(traefik_service_requests_total[5m])

# Error percentage
rate(traefik_service_requests_total{code=~"5.."}[5m]) / rate(traefik_service_requests_total[5m]) * 100

# GitHub API usage
rate(github_api_requests_total[5m])

# MCP command success rate
rate(mcp_commands_total{status="success"}[5m]) / rate(mcp_commands_total[5m]) * 100

# Container resource usage
rate(container_cpu_usage_seconds_total{name=~"github-mcp.*"}[5m]) * 100
```

## 📊 **Sample Dashboard Views**

### **Overview Dashboard:**
```
┌─────────────────────────────────────────────────────────────────┐
│ 🟢 Service Status        │ 📈 Request Rate (req/s)              │
│ ├─ MCP Server: UP        │ ├─ github-mcp-server: 15.2           │
│ ├─ Traefik: UP           │ ├─ traefik: 8.7                      │
│ └─ Prometheus: UP        │ └─ nginx: 3.1                        │
├─────────────────────────────────────────────────────────────────┤
│ ⏱️ Response Times (ms)    │ 🚦 HTTP Status Codes (req/s)         │
│ ├─ 50th percentile: 45ms │ ├─ 2xx Success: 25.8                 │
│ └─ 95th percentile: 120ms│ ├─ 4xx Client Error: 1.2             │
│                          │ └─ 5xx Server Error: 0.0             │
└─────────────────────────────────────────────────────────────────┘
```

### **API Metrics Dashboard:**
```
┌─────────────────────────────────────────────────────────────────┐
│ 🔢 GitHub Rate Limit     │ 📊 GitHub API Requests (req/s)       │
│    Remaining: 4,847      │ ├─ /repos: 2.3                       │
│    Status: 🟢 Healthy    │ ├─ /issues: 1.8                      │
│                          │ └─ /pulls: 0.9                       │
├─────────────────────────────────────────────────────────────────┤
│ ⚡ MCP Commands (req/s)   │ ⏰ API Response Times (ms)           │
│ ├─ push: 0.8             │ ├─ GitHub API 50th: 180ms            │
│ ├─ scan_tasks: 0.3       │ └─ GitHub API 95th: 450ms            │
│ └─ merge: 0.1            │                                       │
└─────────────────────────────────────────────────────────────────┘
```

## 🔍 **Troubleshooting**

### **Dashboard Not Loading:**
```bash
# Check Grafana container
docker logs github-mcp-grafana

# Check Prometheus data source
curl http://prometheus:9090/api/v1/query?query=up

# Verify dashboard provisioning
docker exec github-mcp-grafana ls -la /etc/grafana/provisioning/dashboards/
```

### **No Metrics Data:**
```bash
# Check metrics endpoint
curl https://your-domain.com/metrics

# Verify Prometheus scraping
curl https://metrics.your-domain.com/targets

# Check MCP server metrics
docker logs github-mcp-server | grep -i metrics
```

### **Alerts Not Firing:**
```bash
# Check alert rules
curl https://metrics.your-domain.com/api/v1/rules

# Verify alert manager (if configured)
curl https://metrics.your-domain.com/api/v1/alerts
```

## 🎯 **Key Benefits**

### **✅ What You Get:**

1. **🔍 Complete Visibility**: Every aspect of your system monitored
2. **⚡ Real-time Alerts**: Know about issues before users do
3. **📊 Historical Data**: Track trends and plan capacity
4. **🎯 Performance Optimization**: Identify bottlenecks quickly
5. **🛡️ Security Monitoring**: Track authentication and rate limiting
6. **📱 Mobile Access**: Monitor from anywhere
7. **🔧 Easy Customization**: Add your own metrics and panels

### **🚀 Operational Excellence:**

- **MTTR Reduction**: Faster incident response
- **Proactive Monitoring**: Prevent issues before they occur
- **Capacity Planning**: Data-driven scaling decisions
- **Performance Optimization**: Identify and fix bottlenecks
- **Security Awareness**: Monitor for suspicious activity

---

## 🎉 **Ready to Use!**

Your Grafana dashboards are **automatically configured** and ready to use as soon as you deploy the system. Just visit `https://dashboard.your-domain.com` and start monitoring your GitHub MCP Server! 📊✨