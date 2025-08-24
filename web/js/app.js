// GitHub MCP Server - Client-side JavaScript
// Minimal, secure, and focused on user experience

(function() {
    'use strict';

    // Security: Prevent clickjacking
    if (window.top !== window.self) {
        window.top.location = window.self.location;
    }

    // DOM ready handler
    function ready(fn) {
        if (document.readyState !== 'loading') {
            fn();
        } else {
            document.addEventListener('DOMContentLoaded', fn);
        }
    }

    // Initialize app
    ready(function() {
        initializeApp();
    });

    function initializeApp() {
        console.log('GitHub MCP Server - Client initialized');
        
        // Add loading states to auth button
        setupAuthButton();
        
        // Add copy functionality for tokens (if present)
        setupTokenCopy();
        
        // Add health check indicator
        setupHealthCheck();
        
        // Add keyboard navigation
        setupKeyboardNavigation();
        
        // Add security warnings
        setupSecurityWarnings();
    }

    function setupAuthButton() {
        const authButton = document.querySelector('.auth-button');
        if (!authButton) return;

        authButton.addEventListener('click', function(e) {
            // Add loading state
            this.classList.add('loading');
            this.innerHTML = `
                <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                    <circle cx="12" cy="12" r="3" opacity="0.3"/>
                    <path d="M12 1a11 11 0 1 0 11 11A11 11 0 0 0 12 1zm0 19a8 8 0 1 1 8-8 8 8 0 0 1-8 8z" opacity="0.6"/>
                    <path d="M12 4a8 8 0 0 1 8 8" opacity="1">
                        <animateTransform attributeName="transform" type="rotate" dur="1s" values="0 12 12;360 12 12" repeatCount="indefinite"/>
                    </path>
                </svg>
                Connecting to GitHub...
            `;
            
            // Security: Add CSRF protection
            const csrfToken = generateSecureToken();
            sessionStorage.setItem('oauth_state', csrfToken);
            
            // Modify href to include state parameter
            const url = new URL(this.href, window.location.origin);
            url.searchParams.set('state', csrfToken);
            this.href = url.toString();
        });
    }

    function setupTokenCopy() {
        const tokenElement = document.getElementById('token');
        const copyButton = document.querySelector('.copy-btn');
        
        if (!tokenElement || !copyButton) return;

        copyButton.addEventListener('click', async function() {
            try {
                const token = tokenElement.textContent;
                await navigator.clipboard.writeText(token);
                
                // Show success feedback
                const originalText = this.textContent;
                this.textContent = '✓ Copied!';
                this.style.background = '#28a745';
                
                setTimeout(() => {
                    this.textContent = originalText;
                    this.style.background = '#007bff';
                }, 2000);
                
            } catch (err) {
                console.error('Failed to copy token:', err);
                
                // Fallback: select text
                const range = document.createRange();
                range.selectNode(tokenElement);
                window.getSelection().removeAllRanges();
                window.getSelection().addRange(range);
                
                // Show fallback message
                this.textContent = 'Selected - Press Ctrl+C';
                setTimeout(() => {
                    this.textContent = 'Copy Token';
                }, 3000);
            }
        });
    }

    function setupHealthCheck() {
        // Add health check indicator
        const healthIndicator = document.createElement('div');
        healthIndicator.className = 'health-indicator';
        healthIndicator.innerHTML = `
            <span class="health-status">●</span>
            <span class="health-text">Checking server status...</span>
        `;
        
        // Add CSS for health indicator
        const style = document.createElement('style');
        style.textContent = `
            .health-indicator {
                position: fixed;
                top: 1rem;
                right: 1rem;
                background: white;
                padding: 0.5rem 1rem;
                border-radius: 20px;
                box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
                font-size: 0.8rem;
                display: flex;
                align-items: center;
                gap: 0.5rem;
                z-index: 1000;
            }
            .health-status {
                font-size: 1rem;
                animation: pulse 2s infinite;
            }
            .health-status.online { color: #28a745; }
            .health-status.offline { color: #dc3545; }
            @keyframes pulse {
                0%, 100% { opacity: 1; }
                50% { opacity: 0.5; }
            }
        `;
        document.head.appendChild(style);
        document.body.appendChild(healthIndicator);

        // Check server health
        checkServerHealth(healthIndicator);
        
        // Check every 30 seconds
        setInterval(() => checkServerHealth(healthIndicator), 30000);
    }

    async function checkServerHealth(indicator) {
        try {
            const response = await fetch('/health', {
                method: 'GET',
                headers: {
                    'Accept': 'application/json',
                },
                signal: AbortSignal.timeout(5000) // 5 second timeout
            });

            const statusElement = indicator.querySelector('.health-status');
            const textElement = indicator.querySelector('.health-text');

            if (response.ok) {
                const data = await response.json();
                statusElement.className = 'health-status online';
                textElement.textContent = `Server healthy (${data.version || 'unknown'})`;
            } else {
                statusElement.className = 'health-status offline';
                textElement.textContent = `Server error (${response.status})`;
            }
        } catch (error) {
            const statusElement = indicator.querySelector('.health-status');
            const textElement = indicator.querySelector('.health-text');
            
            statusElement.className = 'health-status offline';
            textElement.textContent = 'Server offline';
            
            console.warn('Health check failed:', error.message);
        }
    }

    function setupKeyboardNavigation() {
        // Add keyboard shortcuts
        document.addEventListener('keydown', function(e) {
            // Alt + G: Go to GitHub auth
            if (e.altKey && e.key === 'g') {
                e.preventDefault();
                const authButton = document.querySelector('.auth-button');
                if (authButton) {
                    authButton.click();
                }
            }
            
            // Alt + H: Focus health check
            if (e.altKey && e.key === 'h') {
                e.preventDefault();
                const healthLink = document.querySelector('a[href="/health"]');
                if (healthLink) {
                    healthLink.focus();
                }
            }
            
            // Escape: Clear focus
            if (e.key === 'Escape') {
                document.activeElement.blur();
            }
        });

        // Add focus indicators
        const style = document.createElement('style');
        style.textContent = `
            .auth-button:focus-visible,
            .footer-links a:focus-visible {
                outline: 2px solid #0366d6;
                outline-offset: 2px;
                box-shadow: 0 0 0 4px rgba(3, 102, 214, 0.1);
            }
        `;
        document.head.appendChild(style);
    }

    function setupSecurityWarnings() {
        // Warn about insecure connections
        if (location.protocol !== 'https:' && location.hostname !== 'localhost') {
            showSecurityWarning('⚠️ Insecure connection detected. Please use HTTPS for security.');
        }

        // Warn about development mode
        if (location.hostname === 'localhost' || location.hostname === '127.0.0.1') {
            console.warn('🚧 Development mode detected. Do not use in production.');
        }

        // Check for mixed content
        if (location.protocol === 'https:' && document.querySelector('script[src^="http:"], link[href^="http:"]')) {
            showSecurityWarning('⚠️ Mixed content detected. Some resources may be blocked.');
        }
    }

    function showSecurityWarning(message) {
        const warning = document.createElement('div');
        warning.className = 'security-warning';
        warning.textContent = message;
        
        const style = document.createElement('style');
        style.textContent = `
            .security-warning {
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                background: #ffc107;
                color: #212529;
                padding: 0.75rem;
                text-align: center;
                font-weight: 600;
                z-index: 9999;
                box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
            }
        `;
        document.head.appendChild(style);
        document.body.insertBefore(warning, document.body.firstChild);

        // Auto-hide after 10 seconds
        setTimeout(() => {
            if (warning.parentNode) {
                warning.parentNode.removeChild(warning);
            }
        }, 10000);
    }

    function generateSecureToken() {
        // Generate cryptographically secure random token
        const array = new Uint8Array(32);
        crypto.getRandomValues(array);
        return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
    }

    // Security: Prevent common XSS vectors
    function sanitizeHTML(str) {
        const div = document.createElement('div');
        div.textContent = str;
        return div.innerHTML;
    }

    // Security: Content Security Policy violation reporting
    document.addEventListener('securitypolicyviolation', function(e) {
        console.error('CSP Violation:', {
            blockedURI: e.blockedURI,
            violatedDirective: e.violatedDirective,
            originalPolicy: e.originalPolicy
        });
        
        // Report to server (if endpoint exists)
        fetch('/csp-report', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                blockedURI: e.blockedURI,
                violatedDirective: e.violatedDirective,
                timestamp: new Date().toISOString()
            })
        }).catch(() => {
            // Ignore reporting errors
        });
    });

    // Export for testing (development only)
    if (location.hostname === 'localhost') {
        window.GitHubMCPApp = {
            checkServerHealth,
            generateSecureToken,
            sanitizeHTML
        };
    }

})();