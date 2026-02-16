# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Security Considerations

WebShell provides shell access to the host system. Please be aware of the following security considerations:

### ⚠️ Warning

**This application gives direct shell access to the server it runs on.** Use with caution and only in controlled environments.

### Recommendations

1. **Isolation**
   - Run in Docker containers with limited permissions
   - Use network isolation (private networks, VPNs)
   - Avoid exposing directly to the internet

2. **Authentication** (Not Implemented)
   - Current version has NO authentication
   - Add reverse proxy with auth (nginx, Traefik, etc.)
   - Consider implementing user authentication in future versions

3. **Authorization** (Not Implemented)
   - No access control in current version
   - Limit who can access the service at network level
   - Consider implementing RBAC in future versions

4. **Resource Limits**
   - Set MAX_TERMINALS to reasonable value
   - Configure IDLE_TIMEOUT appropriately
   - Use container resource limits (CPU, memory)

5. **Monitoring**
   - Enable logging (RUST_LOG=info or debug)
   - Monitor terminal sessions
   - Track resource usage
   - Set up alerts for suspicious activity

6. **Best Practices**
   - Use HTTPS in production
   - Keep dependencies updated
   - Run as non-root user when possible
   - Implement rate limiting
   - Regular security audits

## Reporting a Vulnerability

If you discover a security vulnerability, please:

1. **DO NOT** open a public issue
2. Email details to: [your-security-email@example.com]
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We will respond within 48 hours and work with you to address the issue.

## Known Limitations

- No built-in authentication or authorization
- No rate limiting
- No input sanitization beyond PTY layer
- Sessions persist after disconnection (by design, for reconnection)
- Command history not sanitized in logs

## Future Security Enhancements

Planned security improvements:

- [ ] User authentication (JWT, OAuth, etc.)
- [ ] Role-based access control
- [ ] Rate limiting
- [ ] Audit logging
- [ ] Command whitelisting/blacklisting options
- [ ] Session recording with security review tools
- [ ] Network policy enforcement
- [ ] Enhanced input validation
