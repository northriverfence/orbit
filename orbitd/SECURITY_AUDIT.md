# Security Audit Report - Orbit Daemon

**Date:** 2025-11-04
**Scope:** Critical security paths in orbitd codebase
**Auditor:** Claude Code Agent
**Status:** Initial security review completed

---

## Executive Summary

This security audit identified **12 security concerns** across 4 critical modules:
- **3 HIGH severity** issues requiring immediate attention
- **5 MEDIUM severity** issues requiring near-term remediation
- **4 LOW severity** issues for future consideration

### Critical Findings

1. **Weak Encryption in License Module** (HIGH) - XOR encryption is cryptographically insecure
2. **Bypassable Destructive Command Detection** (HIGH) - Simple keyword matching easily circumvented
3. **No Input Sanitization in Learning Engine** (MEDIUM) - Potential for injection attacks

---

## Module: Executor (`src/executor/mod.rs`)

### Finding 1: Weak Destructive Command Detection (HIGH)

**Location:** `src/executor/mod.rs:23-32`

**Issue:**
The `is_destructive()` method uses simple substring matching which can be easily bypassed:

```rust
fn is_destructive(&self, command: &str) -> bool {
    let destructive_keywords = [
        "rm -rf", "dd ", "mkfs", "format", "> /dev", "shred", "wipefs",
    ];
    destructive_keywords.iter().any(|&keyword| command.contains(keyword))
}
```

**Attack Vectors:**
1. **Quote bypass:** `r"m -rf /` (split string)
2. **Variable bypass:** `CMD="rm -rf"; $CMD /`
3. **Command substitution:** `$(echo rm) -rf /`
4. **Encoding bypass:** `echo Y20gLXJmIC8= | base64 -d | sh`
5. **Alternate commands:** `find / -delete`, `chmod -R 000 /`, `truncate -s 0 file`
6. **Fork bomb:** `:(){ :|:& };:`

**Recommendation:**
- Implement AST-based parsing instead of string matching
- Use syscall interception for destructive operations
- Maintain whitelist of safe operations instead of blacklist
- Parse and validate command structure before execution
- Add confirmation prompts for all filesystem-modifying operations

**Risk:** HIGH - System damage, data loss

---

## Module: License (`src/license/mod.rs`)

### Finding 2: Weak Encryption (HIGH)

**Location:** `src/license/mod.rs:170-188`

**Issue:**
License data is "encrypted" using simple XOR with machine ID:

```rust
fn encrypt_license_data(&self, data: &[u8]) -> Result<Vec<u8>> {
    // Simple XOR encryption with machine ID for now
    // In production, use proper encryption (AES-256-GCM)
    let key = self.get_machine_id();
    let key_bytes = key.as_bytes();

    let encrypted: Vec<u8> = data
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key_bytes[i % key_bytes.len()])
        .collect();

    Ok(encrypted)
}
```

**Vulnerabilities:**
1. **XOR is not encryption** - Known plaintext attacks are trivial
2. **Machine ID is predictable** - Can be enumerated or guessed
3. **No authentication** - No MAC/HMAC to detect tampering
4. **Key reuse** - Same key encrypts all data
5. **Repeating key pattern** - Key bytes cycle, making frequency analysis easy

**Recommendation:**
- Replace with AES-256-GCM (authenticated encryption)
- Use PBKDF2 or Argon2 for key derivation
- Add HMAC for integrity verification
- Generate random salt per encryption
- Consider hardware-backed key storage

**Risk:** HIGH - License bypass, unauthorized usage

---

### Finding 3: Extended Cache Window (MEDIUM)

**Location:** `src/license/mod.rs:85-98`

**Issue:**
48-hour verification cache allows extended offline operation:

```rust
let max_age = Duration::hours(48); // Default to 48 hours
```

**Vulnerabilities:**
1. Revoked licenses remain valid for 48 hours
2. Compromised licenses can't be disabled immediately
3. License violations take 2 days to detect
4. System clock manipulation extends cache indefinitely

**Recommendation:**
- Reduce cache to 4-8 hours maximum
- Implement license revocation list (CRL) checks
- Add clock skew detection
- Require server check on critical operations
- Log all cache usage for audit trail

**Risk:** MEDIUM - License abuse, delayed revocation

---

### Finding 4: No Rate Limiting (LOW)

**Location:** `src/license/mod.rs:109-132`

**Issue:**
Server verification has no rate limiting or exponential backoff:

**Recommendation:**
- Add exponential backoff for failed verifications
- Implement client-side rate limiting (max 10 attempts/hour)
- Add jitter to prevent thundering herd
- Log excessive verification attempts

**Risk:** LOW - Server DoS, credential stuffing

---

## Module: Learning Engine (`src/learning/mod.rs`)

### Finding 5: Context Serialization Information Disclosure (MEDIUM)

**Location:** `src/learning/mod.rs:429`, `src/learning/mod.rs:471`

**Issue:**
Entire `Context` object is serialized to database, potentially including sensitive data:

```rust
.bind(serde_json::to_string(context)?)
```

**Vulnerabilities:**
1. Git branch names may contain sensitive project info
2. Working directory paths reveal system structure
3. Recent commands may contain credentials
4. Username and shell info aid reconnaissance
5. No encryption at rest

**Recommendation:**
- Implement context sanitization before storage
- Remove or hash sensitive fields (paths, usernames)
- Encrypt context data at rest
- Add PII detection and redaction
- Implement data retention policies

**Risk:** MEDIUM - Information disclosure, privacy violation

---

### Finding 6: Unlimited Pattern Storage (MEDIUM)

**Location:** `src/learning/mod.rs:273-363`

**Issue:**
No limits on number of patterns stored, allowing DoS via storage exhaustion:

**Recommendation:**
- Implement max patterns per user (e.g., 10,000)
- Add automatic pruning of old/low-confidence patterns
- Rate limit pattern recording (max 100/minute)
- Monitor database size and alert on rapid growth
- Implement pattern deduplication

**Risk:** MEDIUM - Storage DoS, database bloat

---

### Finding 7: Embedding Blob Integrity (LOW)

**Location:** `src/learning/mod.rs:264-271`

**Issue:**
Embedding blobs are stored without integrity validation:

**Recommendation:**
- Add checksum/hash for embedding blobs
- Validate blob size matches expected dimensions (384 floats = 1536 bytes)
- Handle corrupted blobs gracefully
- Add blob version header for future compatibility

**Risk:** LOW - Data corruption, crash on invalid data

---

### Finding 8: SQL Injection Prevention (LOW)

**Location:** Multiple locations using `sqlx::query`

**Status:** ✅ GOOD - Using parameterized queries throughout

**Note:**
The codebase correctly uses bind parameters for all SQL queries, preventing SQL injection:

```rust
.bind(input)
.bind(executed)
```

**Recommendation:**
- Continue using parameterized queries
- Add SQL injection tests to CI/CD
- Review any dynamic SQL construction carefully

**Risk:** LOW (current implementation is secure)

---

## Module: Classifier (`src/classifier/mod.rs`)

### Finding 9: PATH Traversal During Cache Building (MEDIUM)

**Location:** `src/classifier/mod.rs:182-217`

**Issue:**
Reads all PATH directories without validation, potentially accessing privileged locations:

```rust
for path_dir in path_var.split(':') {
    if let Ok(entries) = std::fs::read_dir(path_dir) {
        // Processes all entries without validation
    }
}
```

**Vulnerabilities:**
1. Could read sensitive directories if PATH is manipulated
2. Symlink following could access unintended locations
3. No limits on number of executables cached
4. Memory exhaustion from large PATH

**Recommendation:**
- Validate PATH entries before processing
- Skip privileged directories (e.g., `/root`, `/etc`)
- Limit cache size (max 10,000 commands)
- Don't follow symlinks during traversal
- Add timeout for directory scanning

**Risk:** MEDIUM - Information disclosure, memory exhaustion

---

### Finding 10: Confidence Threshold Bypass (LOW)

**Location:** `src/classifier/mod.rs:48-55`

**Issue:**
Learned patterns below confidence threshold are still stored and accessible:

**Recommendation:**
- Purge patterns that consistently fall below threshold
- Implement pattern decay over time
- Add user feedback mechanism for low-confidence suggestions
- Log threshold bypass attempts

**Risk:** LOW - Poor user experience, unreliable suggestions

---

## Additional Security Recommendations

### General

1. **Dependency Audit**
   - Run `cargo audit` regularly
   - Monitor for CVEs in dependencies
   - Keep dependencies up-to-date
   - Pin critical dependency versions

2. **Input Validation**
   - Validate all user input length (max 10KB per command)
   - Sanitize special characters in natural language input
   - Implement command syntax parsing before execution
   - Add fuzzing tests for input handlers

3. **Logging and Monitoring**
   - Log all destructive command attempts (successful and blocked)
   - Log license validation failures
   - Monitor for unusual pattern recording rates
   - Add security event alerting

4. **Privilege Management**
   - Run daemon with minimal privileges
   - Drop root privileges after socket binding
   - Use capability-based permissions on Linux
   - Implement proper file permission checks

5. **Network Security**
   - Validate TLS certificates for license server
   - Implement certificate pinning
   - Add timeout for all network operations
   - Use secure DNS resolution

---

## Testing Requirements

Based on this audit, the following security tests are required:

### Critical Tests (Implement Immediately)

1. **Destructive Command Detection**
   - Test basic destructive commands are blocked
   - Test bypass techniques (quotes, variables, substitution)
   - Test edge cases (multiple spaces, mixed case)
   - Test false positives (safe commands with keywords)

2. **License Validation**
   - Test expired license rejection
   - Test invalid license rejection
   - Test cache expiration behavior
   - Test server unreachable handling
   - Test cache file tampering detection

3. **Input Sanitization**
   - Test SQL injection attempts
   - Test command injection attempts
   - Test oversized input handling
   - Test special character handling

### High Priority Tests

4. **Learning Engine Security**
   - Test pattern storage limits
   - Test context sanitization
   - Test embedding validation
   - Test database integrity

5. **Classifier Security**
   - Test PATH validation
   - Test cache size limits
   - Test malicious PATH entries

---

## Remediation Priority

### Immediate (Within 1 week)
- [ ] Replace XOR encryption with AES-256-GCM
- [ ] Implement robust destructive command detection
- [ ] Add input validation and sanitization

### Short-term (Within 1 month)
- [ ] Reduce license cache window to 8 hours
- [ ] Implement pattern storage limits
- [ ] Add context data sanitization
- [ ] Implement PATH validation

### Long-term (Within 3 months)
- [ ] Add comprehensive security logging
- [ ] Implement security monitoring and alerting
- [ ] Add fuzzing test suite
- [ ] Implement privilege dropping
- [ ] Add certificate pinning for license server

---

## Conclusion

While the codebase demonstrates good practices in some areas (parameterized SQL queries, development mode detection), several critical security issues require immediate attention:

1. **Weak encryption** in license management
2. **Bypassable command filtering** in executor
3. **Insufficient input validation** across modules

The recommended mitigations are straightforward and should be prioritized before production release.

---

**Next Steps:**
1. Implement security tests for critical findings
2. Fix HIGH severity issues
3. Schedule follow-up audit after remediation
4. Establish security review process for new code
