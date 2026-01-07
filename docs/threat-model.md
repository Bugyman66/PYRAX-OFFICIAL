# PYRAX Threat Model

> **Version:** 1.0.0  
> **Last Updated:** 2026-01-02

---

## 1. Overview

This document identifies security threats to the PYRAX blockchain and its components, along with mitigations.

---

## 2. Threat Categories

### 2.1 Key Theft

| Threat | Vector | Impact | Likelihood | Mitigation |
|--------|--------|--------|------------|------------|
| Memory dump | Malware reads process memory | Critical | Medium | Secure memory wiping, encrypted key storage |
| Disk theft | Physical access to keystore | Critical | Low | DPAPI encryption, hardware wallets |
| Clipboard sniffing | Malware monitors clipboard | High | Medium | Auto-clear clipboard, never copy full keys |
| Screen capture | Malware captures screenshots | High | Medium | Mask sensitive data in UI |
| IPC interception | Malware intercepts node-wallet communication | Critical | Low | Named pipe ACLs, authentication tokens |
| Keylogger | Malware captures password | Critical | Medium | Virtual keyboard option, hardware keys |

### 2.2 Network Attacks

| Threat | Vector | Impact | Likelihood | Mitigation |
|--------|--------|--------|------------|------------|
| Eclipse attack | Attacker controls all peer connections | Critical | Medium | Diverse peer selection, outbound connections |
| Sybil attack | Attacker creates many fake nodes | High | Medium | Connection limits, peer reputation |
| DoS amplification | Attacker triggers expensive operations | High | High | Rate limiting, proof-of-work on requests |
| Man-in-middle | Attacker intercepts P2P traffic | High | Low | TLS for RPC, signed P2P messages |
| DNS hijacking | Attacker redirects bootnode DNS | High | Low | Hardcoded fallback IPs, DNSSEC |
| BGP hijacking | Attacker reroutes network traffic | Critical | Low | Monitor for anomalies, diverse connectivity |

### 2.3 Consensus Attacks

| Threat | Vector | Impact | Likelihood | Mitigation |
|--------|--------|--------|------------|------------|
| 51% attack | Attacker controls majority hashrate | Critical | Low | KAWPOW ASIC resistance, monitoring |
| Selfish mining | Attacker withholds blocks | Medium | Medium | Uncle tracking, orphan metrics |
| Time-warp | Attacker manipulates timestamps | High | Low | Strict timestamp validation |
| Block withholding | Pool members withhold valid blocks | Medium | Medium | Decentralized pool protocols |
| Long-range attack | Attacker forks from old block | High | Low | Checkpoints, finality rules |
| Nothing-at-stake | Validators sign multiple chains | N/A | N/A | PoW-based (not applicable) |

### 2.4 Smart Contract Risks

| Threat | Vector | Impact | Likelihood | Mitigation |
|--------|--------|--------|------------|------------|
| Reentrancy | Malicious callback during execution | Critical | Medium | Checks-effects-interactions, reentrancy guards |
| Integer overflow | Arithmetic bugs | High | Medium | SafeMath, Rust's checked arithmetic |
| Access control | Missing authorization | Critical | Medium | Role-based access, multi-sig |
| Front-running | MEV extraction | Medium | High | Commit-reveal, encrypted mempools |
| Oracle manipulation | Fake price data | High | Medium | Multiple oracles, TWAP |

### 2.5 Supply Chain Attacks

| Threat | Vector | Impact | Likelihood | Mitigation |
|--------|--------|--------|------------|------------|
| Binary tampering | Modified releases | Critical | Medium | Code signing, reproducible builds |
| Dependency poisoning | Malicious npm/crate | Critical | Medium | Vendored deps, audit pipeline |
| Update hijacking | Fake update server | Critical | Low | Signed updates, multiple mirrors |
| Typosquatting | Similar package names | High | Medium | Dependency review, lockfiles |
| Compromised CI | Attacker modifies build | Critical | Low | Isolated build environment, audit logs |

### 2.6 AI Compute Specific

| Threat | Vector | Impact | Likelihood | Mitigation |
|--------|--------|--------|------------|------------|
| Result manipulation | Worker submits fake results | High | High | Redundant execution, ZK proofs |
| Model theft | Worker steals model weights | High | Medium | Encrypted models, TEE execution |
| Data poisoning | Malicious training data | Medium | Medium | Data validation, provenance tracking |
| Denial of service | Worker accepts but doesn't complete | Medium | High | Timeouts, slashing, reputation |
| Resource exhaustion | Job consumes excessive resources | Medium | Medium | Resource limits, sandboxing |

---

## 3. Component-Specific Threats

### 3.1 Node (pyrax-node)

**Attack Surface:**
- P2P network interface
- RPC API
- Database files
- Configuration files

**Mitigations:**
- [ ] Rate limiting on all endpoints
- [ ] Input validation for all messages
- [ ] Sandboxed execution environment
- [ ] Encrypted database option
- [ ] Secure default configuration

### 3.2 Wallet (pyrax-wallet)

**Attack Surface:**
- Key storage
- Signing operations
- User interface
- IPC communication

**Mitigations:**
- [ ] Hardware-backed key storage (DPAPI/Keychain)
- [ ] Memory encryption for keys
- [ ] Secure random number generation
- [ ] Transaction simulation before signing
- [ ] Address book verification

### 3.3 Miner (pyrax-miner)

**Attack Surface:**
- Stratum protocol
- GPU drivers
- DAG files
- Network communication

**Mitigations:**
- [ ] TLS for stratum connections
- [ ] DAG integrity verification
- [ ] Resource limits
- [ ] Isolated GPU access

### 3.4 Desktop (pyrax-desktop)

**Attack Surface:**
- Embedded node/wallet
- Auto-update mechanism
- Local storage
- External links

**Mitigations:**
- [ ] Code signing
- [ ] Sandboxed webview
- [ ] CSP headers
- [ ] Update signature verification
- [ ] Link confirmation dialogs

---

## 4. Risk Assessment Matrix

| Risk Level | Likelihood × Impact | Examples |
|------------|---------------------|----------|
| **Critical** | High × Critical | 51% attack, key theft, binary tampering |
| **High** | Medium × Critical or High × High | Eclipse attack, reentrancy, DoS |
| **Medium** | Low × Critical or Medium × Medium | Selfish mining, front-running |
| **Low** | Low × Low-Medium | Minor availability issues |

---

## 5. Security Controls

### 5.1 Preventive Controls

1. **Input validation** - All external input sanitized
2. **Authentication** - RPC requires API key for sensitive methods
3. **Authorization** - Role-based access for admin functions
4. **Encryption** - TLS for network, AES-GCM for storage
5. **Code signing** - All releases signed with verified keys

### 5.2 Detective Controls

1. **Logging** - Comprehensive audit logging
2. **Monitoring** - Real-time metrics and alerts
3. **Anomaly detection** - Unusual patterns flagged
4. **Integrity checks** - Periodic verification of stored data

### 5.3 Corrective Controls

1. **Incident response** - Documented procedures
2. **Rollback capability** - Database snapshots
3. **Emergency shutdown** - Kill switch for critical issues
4. **Communication plan** - User notification process

---

## 6. Mitigations Backlog

### Priority 1 (Phase 0)

- [ ] Implement secure key storage with DPAPI
- [ ] Add rate limiting to RPC endpoints
- [ ] Implement peer reputation system
- [ ] Set up code signing infrastructure
- [ ] Configure reproducible builds

### Priority 2 (Phase 1)

- [ ] Implement P2P message signing
- [ ] Add transaction simulation
- [ ] Implement checkpoint system
- [ ] Set up monitoring infrastructure
- [ ] Create incident response plan

### Priority 3 (Sprint I-II)

- [ ] Hardware wallet integration
- [ ] Multi-signature support
- [ ] Advanced anomaly detection
- [ ] Security audit preparation
- [ ] Bug bounty program design

### Priority 4 (Phase 16+)

- [ ] AI job verification proofs
- [ ] TEE integration for compute
- [ ] Encrypted model execution
- [ ] Cross-chain bridge security
- [ ] L2 security audit

---

## 7. Security Testing

### 7.1 Required Tests

- [ ] Fuzzing of P2P message parsing
- [ ] Fuzzing of transaction parsing
- [ ] Chaos testing (network partitions)
- [ ] Load testing (DoS resistance)
- [ ] Penetration testing

### 7.2 Audit Checklist

- [ ] Consensus implementation review
- [ ] Cryptographic implementation review
- [ ] Key management review
- [ ] Network protocol review
- [ ] Smart contract review

---

## 8. Incident Response

### 8.1 Severity Levels

| Level | Description | Response Time |
|-------|-------------|---------------|
| P0 | Active exploitation, fund loss | Immediate |
| P1 | Critical vulnerability discovered | < 4 hours |
| P2 | High-severity issue | < 24 hours |
| P3 | Medium-severity issue | < 1 week |
| P4 | Low-severity issue | Next release |

### 8.2 Response Procedures

1. **Triage** - Assess severity and impact
2. **Contain** - Limit damage (emergency patches, announcements)
3. **Eradicate** - Remove threat (fix vulnerability)
4. **Recover** - Restore normal operations
5. **Learn** - Post-incident review and improvements

---

## 9. Security Contacts

- **Security Email:** security@pyrax.org
- **Bug Bounty:** https://pyrax.org/security/bounty
- **PGP Key:** [To be published]

---

*This document should be reviewed and updated quarterly.*
