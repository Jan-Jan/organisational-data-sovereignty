// Deploys OrgRegistry to a LIVE pallet-revive chain (Paseo Asset Hub or
// Polkadot Asset Hub) via `instantiateWithCode`, waits for finalization,
// then reads `Revive.PristineCode` for the deployed code hash and asserts it
// matches the locally-computed keccak-256 of the blob. Prints the deployed
// contract H160 (the value Stage 2 consumers anchor against). Exits 0 on
// success, non-zero on any failure.
//
// This is the live-chain sibling of sanity-deploy.mjs. The differences:
//   - The signing key comes from $DEPLOYER_SEED (a real, funded account),
//     not the hard-coded //Alice that only works on a chopsticks fork's
//     mock-signature-host.
//   - It waits for `isFinalized` (not just `isInBlock`) because a live
//     deployment must survive a re-org before we record the address.
//   - Key type is configurable via $KEY_TYPE (sr25519 default; pass
//     `ed25519` or `ecdsa` to match how your account was generated).
//
// Required env:
//   RPC_URL        ws(s):// endpoint of the target Asset Hub
//   DEPLOYER_SEED  mnemonic / //Derivation / 0x-seed of a FUNDED account
// Optional env:
//   BLOB_PATH      path to the resolc .pvm blob (default below)
//   KEY_TYPE       sr25519 (default) | ed25519 | ecdsa
//   SALT           0x-hex 32-byte salt for a deterministic CREATE2 address
//                  (default: none → CREATE1, address depends on deployer nonce)

import {ApiPromise, Keyring, WsProvider} from '@polkadot/api';
import {keccakAsHex} from '@polkadot/util-crypto';
import {readFileSync} from 'node:fs';

const RPC_URL   = process.env.RPC_URL;
const SEED      = process.env.DEPLOYER_SEED;
const BLOB_PATH = process.env.BLOB_PATH || 'tmp/revive/OrgRegistry.sol:OrgRegistry.pvm';
const KEY_TYPE  = process.env.KEY_TYPE  || 'sr25519';
const SALT      = process.env.SALT      || null;

if (!RPC_URL)  { console.error('FAIL: set RPC_URL (ws/wss endpoint of the target Asset Hub)'); process.exit(2); }
if (!SEED)     { console.error('FAIL: set DEPLOYER_SEED (mnemonic / //Derivation / 0x-seed of a funded account)'); process.exit(2); }

const blob      = readFileSync(BLOB_PATH);
const localHash = keccakAsHex(blob);
console.log(`local code keccak-256: ${localHash}`);
console.log(`blob size:            ${blob.length} bytes`);

const api     = await ApiPromise.create({provider: new WsProvider(RPC_URL), noInitWarn: true});
const chain   = (await api.rpc.system.chain()).toString();
const keyring = new Keyring({type: KEY_TYPE});
const deployer = keyring.addFromUri(SEED);
console.log(`chain:    ${chain}`);
console.log(`deployer: ${deployer.address} (${KEY_TYPE})`);

// Guard against a silent "deployed nothing because the account is empty"
// run: a live instantiate needs funds for fees + the code storage deposit.
const {data: balance} = await api.query.system.account(deployer.address);
console.log(`free balance: ${balance.free.toString()} planck`);
if (balance.free.isZero()) {
  console.error('FAIL: deployer has zero free balance — fund it first (see README: faucet / transfer)');
  process.exit(1);
}

// Live metadata arg order (verified via
// api.tx.revive.instantiateWithCode.meta.toHuman()):
//   value, weightLimit, storageDepositLimit, code, data, salt
// refTime / proofSize are kept within the Paseo AH per-extrinsic limits
// observed at Stage 1 (maxExtrinsic refTime 1,599,875,000,000 / proofSize
// 8,388,608). Polkadot AH limits are comparable; if instantiate fails with
// `exhaustsResources`, lower these or split the deploy.
const instantiate = api.tx.revive.instantiateWithCode(
  0,                                                    // value
  {refTime: 1_000_000_000_000n, proofSize: 4_000_000n}, // weightLimit
  10_000_000_000_000n,                                  // storageDepositLimit
  '0x' + blob.toString('hex'),                          // code
  '0x',                                                 // data (constructor args)
  SALT,                                                 // salt (null → CREATE1)
);

console.log(`submitting instantiateWithCode as ${deployer.address}...`);
const result = await new Promise((resolve, reject) => {
  instantiate.signAndSend(deployer, ({status, dispatchError, events}) => {
    if (dispatchError) {
      if (dispatchError.isModule) {
        const decoded = api.registry.findMetaError(dispatchError.asModule);
        reject(new Error(`dispatchError: ${decoded.section}.${decoded.name}: ${decoded.docs}`));
      } else {
        reject(new Error(`dispatchError: ${dispatchError.toString()}`));
      }
      return;
    }
    if (status.isInBlock) {
      console.log(`included in block: ${status.asInBlock.toString()} (awaiting finalization...)`);
    }
    if (status.isFinalized) {
      console.log(`finalized in block: ${status.asFinalized.toString()}`);
      resolve({status, events});
    }
  }).catch(reject);
});

let extrinsicSucceeded = false;
let deployedH160 = null;
for (const {event} of result.events) {
  if (event.section === 'system' && event.method === 'ExtrinsicSuccess') {
    extrinsicSucceeded = true;
  }
  if (event.section === 'system' && event.method === 'ExtrinsicFailed') {
    const [dispatchError] = event.data;
    let detail = dispatchError.toString();
    if (dispatchError.isModule) {
      const decoded = api.registry.findMetaError(dispatchError.asModule);
      detail = `${decoded.section}.${decoded.name}: ${decoded.docs}`;
    }
    console.error(`FAIL: ExtrinsicFailed — ${detail}`);
    process.exit(1);
  }
  if (event.section === 'revive') {
    console.log(`  event: ${event.section}.${event.method}`, event.data.toHuman());
    if (event.method === 'Instantiated') {
      const data = event.data.toJSON();
      // toJSON shapes this as [deployer, contract] or {deployer, contract}
      // depending on runtime metadata. Accept both.
      deployedH160 = Array.isArray(data) ? data[1] : (data && data.contract);
    }
  }
}
if (!extrinsicSucceeded) {
  console.error('FAIL: no ExtrinsicSuccess event emitted; deploy outcome unknown');
  process.exit(1);
}

// Verify the on-chain code blob matches what we compiled locally.
const pristine     = await api.query.revive.pristineCode(localHash);
const pristineLen  = pristine.toU8a(true).length;
if (pristine.isEmpty || pristineLen === 0) {
  console.error(`FAIL: no PristineCode entry at ${localHash} (len=${pristineLen})`);
  process.exit(1);
}
const onChainHash = keccakAsHex(pristine.toU8a(true));
if (onChainHash !== localHash) {
  console.error(`FAIL: on-chain code hash ${onChainHash} != local ${localHash}`);
  process.exit(1);
}

console.log('');
console.log('OK — OrgRegistry deployed and verified on-chain.');
console.log(`  chain:         ${chain}`);
console.log(`  code hash:     ${localHash}`);
if (deployedH160) {
  console.log(`  contract H160: ${deployedH160}`);
  console.log(`DEPLOYED_H160=${deployedH160}`);
} else {
  console.log('  contract H160: (no Instantiated event captured — check the log above)');
}
await api.disconnect();
process.exit(0);
