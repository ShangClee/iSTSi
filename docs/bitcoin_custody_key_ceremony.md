# Bitcoin Custody Key Ceremony Plan for iSTSi

## Overview

This document outlines the comprehensive plan for implementing Bitcoin custody infrastructure using a 2-of-3 multisig P2WSH scheme with hardware wallets and PSBT workflows for the iSaToShi (iSTSi) Bitcoin anchor token system.

## 1. Multisig Architecture

### 1.1 Key Configuration
- **Scheme**: P2WSH 2-of-3 multisig
- **Script Type**: Native SegWit v0 (P2WSH)  
- **Derivation**: BIP32 HD wallets with BIP84 derivation paths
- **Descriptor Format**: Modern Bitcoin Core descriptors

### 1.2 Key Holder Roles

| Role | Responsibility | Jurisdiction | Hardware Wallet |
|------|---------------|--------------|-----------------|
| **Custody Ops** | Day-to-day operations, PSBT signing | USA/Canada | Coldcard Mk4 |
| **Compliance** | Compliance verification, co-signing | Panama/El Salvador | Ledger Nano X |
| **Recovery** | Emergency recovery, audit trail | Canada (backup) | Trezor Model T |

### 1.3 Security Requirements
- All hardware wallets must support PSBT (Partially Signed Bitcoin Transactions)
- Air-gapped signing (no direct network connectivity during signing)
- Tamper-evident physical storage
- Geographic distribution across jurisdictions
- Secure communication channels for PSBT exchange

## 2. Key Generation Ceremony

### 2.1 Pre-Ceremony Preparation

#### Hardware Procurement
- [ ] 3x Coldcard Mk4 (primary for each role)
- [ ] 3x Ledger Nano X (backup devices)  
- [ ] 3x Trezor Model T (tertiary backup)
- [ ] 3x Faraday bags for RF isolation
- [ ] 3x Tamper-evident storage containers
- [ ] Dice for entropy generation
- [ ] QR code scanners/cameras
- [ ] Air-gapped laptops (3x)

#### Software Setup
- Bitcoin Core 25.0+ with descriptor wallet support
- Hardware wallet firmware updates (air-gapped)
- PSBT tooling (HWI, Electrum, Sparrow)
- Checksum verification tools

### 2.2 Ceremony Protocol

#### Phase 1: Entropy Generation and Seed Creation
1. **Location**: Secure facility with video recording
2. **Participants**: All 3 key holders + 2 witnesses
3. **Process**:
   - Generate high-entropy seeds using dice rolls (256 bits)
   - Initialize hardware wallets with generated seeds
   - Verify seed backup on secondary device
   - Store seed phrases in tamper-evident containers

#### Phase 2: Extended Public Key (xpub) Exchange
1. **Derivation Path**: `m/84'/0'/0'` (BIP84 Native SegWit)
2. **xpub Extraction**:
   - Each key holder derives account-level xpub
   - xpubs exchanged via QR codes (air-gapped)
   - Manual verification of xpub checksums
   - Triple verification by all participants

#### Phase 3: Descriptor Creation and Verification
1. **Multisig Descriptor Generation**:
   ```
   wsh(sortedmulti(2,[fingerprint1/84'/0'/0']xpub1/*,[fingerprint2/84'/0'/0']xpub2/*,[fingerprint3/84'/0'/0']xpub3/*))
   ```
2. **Verification Process**:
   - Each participant independently derives first 10 addresses
   - Cross-verify all addresses match across all devices
   - Generate test transactions on regtest
   - Confirm 2-of-3 signing capabilities

#### Phase 4: Documentation and Storage
1. **Ceremony Minutes**: Signed attestation document
2. **Descriptor Storage**: Encrypted storage in multiple locations
3. **Address Generation**: First 100 receiving addresses pre-generated
4. **Emergency Procedures**: Recovery process documentation

### 2.3 Post-Ceremony Security
- Hardware wallets stored in tamper-evident containers
- Geographical distribution (USA, Panama, Canada)
- Annual re-verification ceremonies
- Quarterly address rotation for operational wallets

## 3. PSBT Workflow Implementation

### 3.1 Spending Policy Framework

#### Authorization Tiers
| Amount (BTC) | Approval Required | Timelock | Verification |
|--------------|------------------|----------|--------------|
| < 0.1 | Custody Ops + 1 | None | Automated checks |
| 0.1 - 1.0 | Custody Ops + Compliance | 4 hours | Manual review |
| 1.0 - 10.0 | All 2-of-3 | 24 hours | Multi-person approval |
| > 10.0 | All 2-of-3 | 72 hours | Board approval |

#### Daily Operational Limits
- Maximum daily spending: 5 BTC
- Signing window: 9 AM - 6 PM local time
- Emergency override: 2-of-3 + incident ticket

### 3.2 PSBT Creation Process

#### Input Validation
1. **Transaction Requirements**:
   - Minimum fee rate validation
   - Output address verification (no blacklisted addresses)
   - Amount bounds checking
   - UTXO age verification (prevents dust attacks)

2. **Business Logic Validation**:
   - Link to verified burn request from iSTSi contract
   - KYC compliance check for recipient
   - AML screening for destination address
   - Reserve ratio maintenance post-transaction

#### PSBT Structure
```json
{
  "inputs": [
    {
      "txid": "...",
      "vout": 0,
      "witness_utxo": "...",
      "witness_script": "...",
      "bip32_derivs": [...]
    }
  ],
  "outputs": [
    {
      "amount": 50000000,
      "script": "...",
      "bip32_derivs": [...]
    }
  ],
  "proprietary": {
    "iSTSi_burn_request_id": "...",
    "compliance_approval": "...",
    "created_timestamp": "..."
  }
}
```

### 3.3 Signing Workflow

#### Step 1: PSBT Creation (Automated)
- Watch-only wallet creates unsigned PSBT
- Include compliance metadata in proprietary fields
- Store PSBT in secure queue with status tracking

#### Step 2: First Signature (Custody Ops)
- Retrieve PSBT from secure queue
- Verify transaction details on hardware wallet screen
- Sign PSBT with Custody Ops key
- Update PSBT with signature

#### Step 3: Second Signature (Compliance/Recovery)
- Compliance officer reviews transaction justification
- Verify against burn request and KYC data
- Sign PSBT with second hardware wallet
- Complete 2-of-3 signature requirement

#### Step 4: Broadcast and Monitoring
- Finalize and broadcast transaction
- Monitor for confirmation
- Update internal ledger upon confirmation
- Emit proof-of-reserves update event

## 4. Cold/Warm Wallet Architecture

### 4.1 Address Hierarchy

#### Deep Cold Vault (95% of reserves)
- **Purpose**: Long-term storage, emergency-only access
- **Access Frequency**: Monthly rebalancing or emergency
- **Security**: Geographic distribution, bank-grade safes
- **Monitoring**: Watch-only nodes with alerting

#### Warm Operational Wallet (5% of reserves)
- **Purpose**: Daily redemption operations
- **Access Frequency**: Multiple times daily
- **Security**: Multisig with faster signing process
- **Refill Trigger**: When balance drops below 24-hour redemption estimate

#### Hot Lightning Channel Management (< 0.1% of reserves)
- **Purpose**: Lightning Network channel management only
- **Access Frequency**: Real-time for channel operations
- **Security**: Single-sig with strict amount limits
- **Monitoring**: Real-time balance and transaction alerts

### 4.2 UTXO Management Strategy

#### Coin Selection
1. **Privacy**: Avoid address reuse, minimize address linking
2. **Efficiency**: Batch multiple redemptions when possible
3. **Fee Optimization**: Use appropriate fee rates with RBF support
4. **Consolidation**: Regular UTXO consolidation during low-fee periods

#### Transaction Features
- **Replace-by-Fee (RBF)**: All transactions use RBF for fee bumping
- **Child-Pays-for-Parent (CPFP)**: Emergency fee acceleration
- **Batching**: Multiple redemptions in single transaction
- **Segwit**: All transactions use native segwit for fee savings

## 5. Monitoring and Security

### 5.1 Watch-Only Infrastructure
- **Full Nodes**: 3x Bitcoin Core nodes (geographically distributed)
- **Indexing**: Address and transaction indexing for fast lookups
- **API Layer**: Secure API for balance and transaction queries
- **Redundancy**: Failover between nodes for high availability

### 5.2 Alerting System
- **Balance Changes**: Any unexpected balance changes
- **Large Transactions**: Transactions above threshold amounts
- **Fee Anomalies**: Unusually high or low fee transactions
- **Confirmation Delays**: Transactions stuck in mempool
- **Hardware Failures**: Hardware wallet communication errors

### 5.3 Audit Trail
- **Transaction Log**: Complete record of all PSBT workflows
- **Approval Records**: Digital signatures on all approval steps
- **Access Log**: Hardware wallet access attempts and successes
- **Emergency Procedures**: All emergency access events logged

## 6. Emergency Procedures

### 6.1 Key Compromise Response
1. **Immediate**: Halt all new transactions
2. **Assessment**: Determine scope of compromise
3. **Migration**: Generate new multisig with remaining secure keys
4. **Recovery**: Move funds to new secure multisig
5. **Investigation**: Forensic analysis of compromise

### 6.2 Hardware Failure Recovery
1. **Device Failure**: Use backup hardware wallet with same seed
2. **Seed Compromise**: Emergency key rotation with 72-hour timelock
3. **Multiple Failures**: Invoke emergency recovery procedures
4. **Procedure Testing**: Quarterly disaster recovery exercises

## 7. Implementation Timeline

### Week 1: Procurement and Setup
- [ ] Hardware wallet procurement
- [ ] Security facility preparation
- [ ] Software and tooling setup
- [ ] Documentation review and approval

### Week 2: Dry Run Ceremony
- [ ] Practice ceremony with testnet
- [ ] PSBT workflow testing
- [ ] Emergency procedure drills
- [ ] Process refinements

### Week 3: Production Ceremony
- [ ] Main key generation ceremony
- [ ] xpub exchange and verification
- [ ] Descriptor creation and testing
- [ ] Initial funding of multisig addresses

### Week 4: Operational Deployment
- [ ] Integration with iSTSi contract
- [ ] Monitoring system deployment
- [ ] Staff training on procedures
- [ ] Go-live readiness review

## 8. Compliance and Legal

### 8.1 Regulatory Considerations
- **Custody Requirements**: Meet jurisdictional custody standards
- **Key Management**: Follow industry best practices (NIST, ISO 27001)
- **Audit Trail**: Maintain records for regulatory examination
- **Insurance**: Explore custody insurance options

### 8.2 Documentation Requirements
- **Custody Procedures**: Complete operational procedures
- **Emergency Plans**: Incident response and recovery plans
- **Training Materials**: Staff training and certification
- **Legal Opinions**: Custody structure legal review

This comprehensive plan ensures secure, compliant, and operationally robust Bitcoin custody infrastructure for the iSTSi token system, with proper controls, monitoring, and emergency procedures in place.
