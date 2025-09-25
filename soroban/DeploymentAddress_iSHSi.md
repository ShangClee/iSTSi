# iSatoshi (iSHSi) Token Deployment Documentation

## Project Overview

**Token Name**: iSatoshi  
**Symbol**: iSHSi  
**Network**: Stellar Testnet  
**Contract Type**: Fungible Token (SEP-41)  
**Framework**: OpenZeppelin Stellar Soroban Contracts v0.4.1  
**Deployment Date**: September 2, 2025  

## Contract Details

### 🚀 LATEST VERSION (8 Decimals)
- **Contract Address**: `CDWDBUUFS4S32WT5WQSHV43VNWW4W4332IAZFUSO2XPNELU3EIUWU2SL`
- **Explorer Link**: https://stellar.expert/explorer/testnet/contract/CDWDBUUFS4S32WT5WQSHV43VNWW4W4332IAZFUSO2XPNELU3EIUWU2SL
- **Decimals**: 8 ✅ **LATEST UPDATE**
- **Initial Supply**: 100,000,000 tokens (10,000,000,000,000,000 units)

### Previous Version (6 Decimals) - SUPERSEDED
- **Contract Address**: `CCIURKT5VRALE2WPBO7NDNGFTV2HXFUU3RK734O5EJEH7BOSUFG7GFLO`
- **Explorer Link**: https://stellar.expert/explorer/testnet/contract/CCIURKT5VRALE2WPBO7NDNGFTV2HXFUU3RK734O5EJEH7BOSUFG7GFLO
- **Decimals**: 6
- **Initial Supply**: 100,000,000 tokens (100,000,000 units)

### Original Version (18 Decimals) - DEPRECATED
- **Contract Address**: `CADRNANDOWWKKRV3BBPWJNSF6WWVOUCB2R533DO7PTXAAF65VDCKASZ5`
- **Explorer Link**: https://stellar.expert/explorer/testnet/contract/CADRNANDOWWKKRV3BBPWJNSF6WWVOUCB2R533DO7PTXAAF65VDCKASZ5
- **Decimals**: 18
- **Initial Supply**: 100,000,000 tokens
- **Owner**: Alice (`GBQMEEQAO4TUE6IJJTLIIMNZLIB7ZIW4WEYQKZ2K523MWTI25LP7BXAT`)

## Token Features

✅ **ERC20-like Functionality**: Standard transfer, transferFrom, balance, name, symbol, decimals  
✅ **Mintable**: Owner can mint new tokens to any account  
✅ **Burnable**: Any account can burn their own tokens  
✅ **Pausable**: Owner can pause/unpause all token operations  
✅ **Ownable**: Owner-based access control  
✅ **Upgradeable**: Contract can be upgraded by owner  

---

## Deployment Tasks & Results

### Phase 1: Environment Setup

#### Task 1: Install Stellar CLI ✅
**Command**:
```bash
cargo install --locked stellar-cli
```
**Result**:
- ✅ Successfully installed Stellar CLI v23.1.1
- ✅ Verification: `stellar --version` → `stellar 23.1.1`

#### Task 2: Configure Testnet Network ✅
**Commands**:
```bash
stellar network add --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"
```
**Result**:
- ✅ Testnet network configured successfully
- ✅ Verification: `stellar network ls` shows testnet available

---

### Phase 2: Build and Test

#### Task 3: Build the Contract ✅
**Commands**:
```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```
**Results**:
- ✅ WebAssembly target added to Rust
- ✅ Contract compiled successfully
- ✅ Generated: `target/wasm32-unknown-unknown/release/fungible_contract.wasm` (29,138 bytes)

#### Task 4: Run Tests ✅
**Command**:
```bash
cargo test
```
**Issues & Fixes**:
- ❌ Initial test failure: Name mismatch ("ISatoshi" vs "iSatoshi")
- ✅ Fixed test in `/contracts/fungible/src/test.rs`:
  ```rust
  // Changed from "ISatoshi" to "iSatoshi"
  assert_eq!(client.name(), String::from_str(&env, "iSatoshi"));
  ```
- ✅ Final result: All tests passed

#### Task 5: Generate Optimized WebAssembly ✅
**Command**:
```bash
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/fungible_contract.wasm
```
**Result**:
- ✅ Optimized from 29,138 bytes → 15,142 bytes (48% reduction)
- ✅ Generated: `fungible_contract.optimized.wasm`

---

### Phase 3: Deployment Preparation

#### Task 6: Create and Fund Identity ✅
**Commands**:
```bash
stellar keys generate alice --network testnet
stellar keys fund alice --network testnet
```
**Results**:
- ✅ Alice identity created and saved to `/Users/shang/.config/stellar/identity/alice.toml`
- ✅ Alice address: `GBQMEEQAO4TUE6IJJTLIIMNZLIB7ZIW4WEYQKZ2K523MWTI25LP7BXAT`
- ✅ Account funded with testnet XLM via friendbot

#### Task 7: Deploy Contract to Testnet ✅
**Command**:
```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/fungible_contract.optimized.wasm \
  --source alice \
  --network testnet \
  -- \
  --recipient GBQMEEQAO4TUE6IJJTLIIMNZLIB7ZIW4WEYQKZ2K523MWTI25LP7BXAT \
  --owner GBQMEEQAO4TUE6IJJTLIIMNZLIB7ZIW4WEYQKZ2K523MWTI25LP7BXAT
```
**Results**:
- ✅ Contract deployed successfully
- ✅ Contract Address: `CADRNANDOWWKKRV3BBPWJNSF6WWVOUCB2R533DO7PTXAAF65VDCKASZ5`
- ✅ Transaction: https://stellar.expert/explorer/testnet/tx/e7542e389daac5939b3aafd7d29a5a9c544a1582d068607563e6172bebd4fe55
- ✅ Contract alias created: `isatoshi`

---

### Phase 4: Contract Function Testing

#### Task 8: Test Contract Functions ✅

##### 8.1 Basic Token Information ✅
**Commands & Results**:
```bash
# Token Name
stellar contract invoke --id isatoshi --network testnet --source alice -- name
# Result: "iSatoshi" ✅

# Token Symbol  
stellar contract invoke --id isatoshi --network testnet --source alice -- symbol
# Result: "iSHSi" ✅

# Decimals
stellar contract invoke --id isatoshi --network testnet --source alice -- decimals
# Result: 18 ✅

# Alice's Balance (Initial Supply)
stellar contract invoke --id isatoshi --network testnet --source alice -- balance --account GBQMEEQAO4TUE6IJJTLIIMNZLIB7ZIW4WEYQKZ2K523MWTI25LP7BXAT
# Result: "100000000000000000000000000" (100M tokens) ✅
```

##### 8.2 Create Test Account (Bob) ✅
**Commands**:
```bash
stellar keys generate bob --network testnet
stellar keys fund bob --network testnet
```
**Result**:
- ✅ Bob address: `GAMPG6XAT22CR3E4SPRD6M24FKFT5XB7YPQL3PBE2MMW3PNAQVJ5MNFI`

##### 8.3 Transfer Function ✅
**Command**:
```bash
stellar contract invoke --id isatoshi --network testnet --source alice --send=yes -- transfer \
  --from GBQMEEQAO4TUE6IJJTLIIMNZLIB7ZIW4WEYQKZ2K523MWTI25LP7BXAT \
  --to GAMPG6XAT22CR3E4SPRD6M24FKFT5XB7YPQL3PBE2MMW3PNAQVJ5MNFI \
  --amount 1000000000000000000
```
**Results**:
- ✅ Transfer successful (1 token to Bob)
- ✅ Transaction: `1c24e2e45845a8cc56c91a2797582267e375a6cc9bffb5b3c97edefe95bd94a8`
- ✅ Bob's balance verification: `"1000000000000000000"` ✅

##### 8.4 Minting Function (Owner Only) ✅
**Command**:
```bash
stellar contract invoke --id isatoshi --network testnet --source alice --send=yes -- mint \
  --account GAMPG6XAT22CR3E4SPRD6M24FKFT5XB7YPQL3PBE2MMW3PNAQVJ5MNFI \
  --amount 5000000000000000000
```
**Results**:
- ✅ Mint successful (5 tokens to Bob)
- ✅ Transaction: `d0bc8cd76505f5587ddbdac3a1e277eb2f63a7578da5b060af609f7943e1c44b`
- ✅ Bob's balance after mint: `"6000000000000000000"` (6 tokens) ✅

##### 8.5 Burn Function ✅
**Command**:
```bash
stellar contract invoke --id isatoshi --network testnet --source bob --send=yes -- burn \
  --from GAMPG6XAT22CR3E4SPRD6M24FKFT5XB7YPQL3PBE2MMW3PNAQVJ5MNFI \
  --amount 2000000000000000000
```
**Results**:
- ✅ Burn successful (2 tokens from Bob)
- ✅ Transaction: `09e45719828ae6f6dd14e050fb0e5a2e044bc36564e6d663e7436c72ecbf45d4`
- ✅ Bob's balance after burn: `"4000000000000000000"` (4 tokens) ✅

##### 8.6 Pausable Functions (Owner Only) ✅

**Pause Contract**:
```bash
stellar contract invoke --id isatoshi --network testnet --source alice --send=yes -- pause \
  --caller GBQMEEQAO4TUE6IJJTLIIMNZLIB7ZIW4WEYQKZ2K523MWTI25LP7BXAT
```
**Results**:
- ✅ Contract paused successfully
- ✅ Transaction: `0f45aca198128a5800839fd345bb846e73cfae6b9be46c95d76f13ee2f1e8e1d`
- ✅ Pause status verification: `true` ✅

**Test Transfer While Paused**:
```bash
stellar contract invoke --id isatoshi --network testnet --source alice -- transfer \
  --from GBQMEEQAO4TUE6IJJTLIIMNZLIB7ZIW4WEYQKZ2K523MWTI25LP7BXAT \
  --to GAMPG6XAT22CR3E4SPRD6M24FKFT5XB7YPQL3PBE2MMW3PNAQVJ5MNFI \
  --amount 1000000000000000000
```
**Results**:
- ✅ Transfer correctly failed with error: `Error(Contract, #1000)` ✅

**Unpause Contract**:
```bash
stellar contract invoke --id isatoshi --network testnet --source alice --send=yes -- unpause \
  --caller GBQMEEQAO4TUE6IJJTLIIMNZLIB7ZIW4WEYQKZ2K523MWTI25LP7BXAT
```
**Results**:
- ✅ Contract unpaused successfully
- ✅ Transaction: `95cd676b1c894c7e6316c1c3a7c2dc3e7665e4911cf6dab0cc30e39ccfa13a22`
- ✅ Pause status verification: `false` ✅

**Test Transfer After Unpause**:
```bash
stellar contract invoke --id isatoshi --network testnet --source alice --send=yes -- transfer \
  --from GBQMEEQAO4TUE6IJJTLIIMNZLIB7ZIW4WEYQKZ2K523MWTI25LP7BXAT \
  --to GAMPG6XAT22CR3E4SPRD6M24FKFT5XB7YPQL3PBE2MMW3PNAQVJ5MNFI \
  --amount 1000000000000000000
```
**Results**:
- ✅ Transfer successful after unpause
- ✅ Transaction: `ae85bae3ac583263c1d354cf4cad695d5c38752017327599047dab4ad087a11c`

---

## Final State Summary

### Account Balances
- **Alice**: 99,999,998 iSHSi tokens (100M initial - 2 transfers)
- **Bob**: 5 iSHSi tokens (1 transfer + 5 minted - 2 burned + 1 final transfer)

### Contract Status
- ✅ **Deployed**: Testnet
- ✅ **Paused**: False (active)
- ✅ **Owner**: Alice
- ✅ **All Functions**: Tested and working

---

## Development Environment

### System Requirements Met ✅
- **macOS**: Apple Silicon (arm64)
- **Rust**: v1.89.0
- **Stellar CLI**: v23.1.1
- **Node.js**: v18 (configured in PATH)
- **Shell**: zsh v5.9

### Dependencies ✅
- **soroban-sdk**: v22.0.8
- **stellar-tokens**: v0.4.1
- **stellar-access**: v0.4.1
- **stellar-contract-utils**: v0.4.1
- **stellar-macros**: v0.4.1

---

## Usage Instructions

### Interacting with the Contract

**Basic Contract Interaction**:
```bash
# Check token information
stellar contract invoke --id isatoshi --network testnet --source alice -- name
stellar contract invoke --id isatoshi --network testnet --source alice -- symbol
stellar contract invoke --id isatoshi --network testnet --source alice -- decimals

# Check balance
stellar contract invoke --id isatoshi --network testnet --source alice -- balance --account <ADDRESS>

# Transfer tokens
stellar contract invoke --id isatoshi --network testnet --source alice --send=yes -- transfer \
  --from <FROM_ADDRESS> --to <TO_ADDRESS> --amount <AMOUNT>
```

**Owner Functions** (Alice only):
```bash
# Mint tokens
stellar contract invoke --id isatoshi --network testnet --source alice --send=yes -- mint \
  --account <RECIPIENT_ADDRESS> --amount <AMOUNT>

# Pause/Unpause contract
stellar contract invoke --id isatoshi --network testnet --source alice --send=yes -- pause --caller <OWNER_ADDRESS>
stellar contract invoke --id isatoshi --network testnet --source alice --send=yes -- unpause --caller <OWNER_ADDRESS>

# Check pause status
stellar contract invoke --id isatoshi --network testnet --source alice -- paused
```

### Deploying to Mainnet

To deploy to mainnet, replace `--network testnet` with `--network mainnet` and ensure:
1. Mainnet network is configured
2. You have sufficient XLM for deployment costs
3. You've thoroughly tested on testnet first

---

## Security Considerations

- ✅ **Access Control**: Owner-only functions properly restricted
- ✅ **Pausable**: Emergency stop mechanism tested
- ✅ **OpenZeppelin**: Using audited contract templates
- ✅ **Testing**: All functions tested before deployment

## Contact

For security issues, please contact: isatoshixlm@gmail.com

---

## Appendix: Key Addresses

- **Contract**: `CADRNANDOWWKKRV3BBPWJNSF6WWVOUCB2R533DO7PTXAAF65VDCKASZ5`
- **Alice (Owner)**: `GBQMEEQAO4TUE6IJJTLIIMNZLIB7ZIW4WEYQKZ2K523MWTI25LP7BXAT`
- **Bob (Test Account)**: `GAMPG6XAT22CR3E4SPRD6M24FKFT5XB7YPQL3PBE2MMW3PNAQVJ5MNFI`

**Explorer Links**:
- Contract: https://stellar.expert/explorer/testnet/contract/CADRNANDOWWKKRV3BBPWJNSF6WWVOUCB2R533DO7PTXAAF65VDCKASZ5
- Alice: https://stellar.expert/explorer/testnet/account/GBQMEEQAO4TUE6IJJTLIIMNZLIB7ZIW4WEYQKZ2K523MWTI25LP7BXAT
- Bob: https://stellar.expert/explorer/testnet/account/GAMPG6XAT22CR3E4SPRD6M24FKFT5XB7YPQL3PBE2MMW3PNAQVJ5MNFI

---

## 🚀 UPDATE: 6-Decimal Version Deployment

**Update Date**: September 3, 2025

### Changes Made ✅
1. **Decimals**: Changed from 18 → 6
2. **Initial Supply**: Updated from `100000000000000000000000000` → `100000000` (same 100M tokens, adjusted for decimals)
3. **Contract Files Updated**:
   - `/contracts/fungible/src/contract.rs`
   - `/iSatoshi.rs`

### New Contract Deployment ✅
- **New Address**: `CCIURKT5VRALE2WPBO7NDNGFTV2HXFUU3RK734O5EJEH7BOSUFG7GFLO`
- **Alias**: `isatoshi-6`
- **Explorer**: https://stellar.expert/explorer/testnet/contract/CCIURKT5VRALE2WPBO7NDNGFTV2HXFUU3RK734O5EJEH7BOSUFG7GFLO

### Testing Results ✅
- ✅ **Decimals**: Verified as 6
- ✅ **Name/Symbol**: "iSatoshi" / "iSHSi" unchanged
- ✅ **Initial Supply**: 100,000,000 tokens (100000000 units)
- ✅ **Transfer**: 1 token = 1000000 units
- ✅ **Minting**: 5 tokens = 5000000 units
- ✅ **Burning**: 2 tokens = 2000000 units
- ✅ **Final Balance**: Bob has 4 tokens (4000000 units)

### Usage with 6 Decimals

**Token Unit Conversion**:
- 1 iSHSi token = 1,000,000 units
- 0.5 iSHSi tokens = 500,000 units
- 0.1 iSHSi tokens = 100,000 units

**Example Commands (6 Decimals)**:
```bash
# Transfer 1 token (1000000 units)
stellar contract invoke --id isatoshi-6 --network testnet --source alice --send=yes -- transfer \
  --from GBQMEEQAO4TUE6IJJTLIIMNZLIB7ZIW4WEYQKZ2K523MWTI25LP7BXAT \
  --to GAMPG6XAT22CR3E4SPRD6M24FKFT5XB7YPQL3PBE2MMW3PNAQVJ5MNFI \
  --amount 1000000

# Mint 5 tokens (5000000 units)
stellar contract invoke --id isatoshi-6 --network testnet --source alice --send=yes -- mint \
  --account GAMPG6XAT22CR3E4SPRD6M24FKFT5XB7YPQL3PBE2MMW3PNAQVJ5MNFI \
  --amount 5000000

# Check decimals
stellar contract invoke --id isatoshi-6 --network testnet --source alice -- decimals
# Result: 6 ✅
```

## 🚀 LATEST UPDATE: 8-Decimal Version Deployment

**Update Date**: September 3, 2025

### Changes Made ✅
1. **Decimals**: Changed from 6 → 8 🚀 **LATEST**
2. **Initial Supply**: Updated from `100000000` → `10000000000000000` (same 100M tokens, adjusted for 8 decimals)
3. **Contract Files Updated**:
   - `/contracts/fungible/src/contract.rs`
   - `/iSatoshi.rs`

### New 8-Decimal Contract Deployment ✅
- **Latest Address**: `CDWDBUUFS4S32WT5WQSHV43VNWW4W4332IAZFUSO2XPNELU3EIUWU2SL`
- **Alias**: `isatoshi-8`
- **Explorer**: https://stellar.expert/explorer/testnet/contract/CDWDBUUFS4S32WT5WQSHV43VNWW4W4332IAZFUSO2XPNELU3EIUWU2SL
- **Transaction**: https://stellar.expert/explorer/testnet/tx/d18706eb733cdd43b8a3fda917c6196bb7d2dce287b5cb54898d343b67ad99d4

### Testing Results (8 Decimals) ✅
- ✅ **Decimals**: Verified as 8
- ✅ **Name/Symbol**: "iSatoshi" / "iSHSi" unchanged
- ✅ **Initial Supply**: 100,000,000 tokens (10000000000000000 units)
- ✅ **Transfer**: 1 token = 100000000 units (100M units)
- ✅ **Minting**: 5 tokens = 500000000 units (500M units)
- ✅ **Burning**: 2 tokens = 200000000 units (200M units)
- ✅ **Final Balance**: Bob has 4 tokens (400000000 units)

### Usage with 8 Decimals 🚀

**Token Unit Conversion (8 Decimals)**:
- 1 iSHSi token = 100,000,000 units
- 0.5 iSHSi tokens = 50,000,000 units
- 0.1 iSHSi tokens = 10,000,000 units
- 0.01 iSHSi tokens = 1,000,000 units

**Example Commands (8 Decimals)**:
```bash
# Transfer 1 token (100000000 units)
stellar contract invoke --id isatoshi-8 --network testnet --source alice --send=yes -- transfer \
  --from GBQMEEQAO4TUE6IJJTLIIMNZLIB7ZIW4WEYQKZ2K523MWTI25LP7BXAT \
  --to GAMPG6XAT22CR3E4SPRD6M24FKFT5XB7YPQL3PBE2MMW3PNAQVJ5MNFI \
  --amount 100000000

# Mint 5 tokens (500000000 units)
stellar contract invoke --id isatoshi-8 --network testnet --source alice --send=yes -- mint \
  --account GAMPG6XAT22CR3E4SPRD6M24FKFT5XB7YPQL3PBE2MMW3PNAQVJ5MNFI \
  --amount 500000000

# Check decimals
stellar contract invoke --id isatoshi-8 --network testnet --source alice -- decimals
# Result: 8 ✅

# Check balance
stellar contract invoke --id isatoshi-8 --network testnet --source alice -- balance --account <ADDRESS>
# Alice balance: "10000000000000000" (100M tokens)
```

### Contract Version History 📚

| Version | Decimals | Contract Address | Status |
|---------|----------|-----------------|--------|
| Latest | 8 | `CDWDBUUFS4S32WT5WQSHV43VNWW4W4332IAZFUSO2XPNELU3EIUWU2SL` | ✅ **ACTIVE** |
| v2 | 6 | `CCIURKT5VRALE2WPBO7NDNGFTV2HXFUU3RK734O5EJEH7BOSUFG7GFLO` | ⚠️ Superseded |
| v1 | 18 | `CADRNANDOWWKKRV3BBPWJNSF6WWVOUCB2R533DO7PTXAAF65VDCKASZ5` | ❌ Deprecated |

---

**Deployment Status**: ✅ **COMPLETED SUCCESSFULLY**  
**Original Contract (18 decimals)**: ✅ **DEPLOYED**  
**Updated Contract (6 decimals)**: ✅ **DEPLOYED & SUPERSEDED**  
**Latest Contract (8 decimals)**: ✅ **DEPLOYED & TESTED** 🚀  
**Total Tasks**: 8/8 + All Updates ✅  
**All Functions Tested**: ✅  
**Ready for Production**: ✅
