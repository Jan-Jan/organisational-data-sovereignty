// Deploy OrgRegistry's EVM bytecode (solc/forge output) to a running chopsticks
// Paseo-AH fork via pallet-revive's `instantiateWithCode`. The dual-VM runtime
// accepts EVM creation bytecode directly (REVM backend) — no resolc/PVM blob and
// no eth-rpc proxy needed. Prints `DEPLOYED_H160=<0x..>` for the caller to parse.
//
// Used by the org-node `chain_genesis_e2e` test; mirrors how the contract is
// deployed live (the forge-built EVM artifact), so the test exercises the same
// VM backend as production.
import {ApiPromise, Keyring, WsProvider} from '@polkadot/api';
import {readFileSync} from 'node:fs';

const RPC = process.env.RPC_URL || 'ws://localhost:8000';
const ARTIFACT =
  process.env.ARTIFACT_PATH ||
  new URL('../out/OrgRegistry.sol/OrgRegistry.json', import.meta.url);

let artifact;
try {
  artifact = JSON.parse(readFileSync(ARTIFACT));
} catch (e) {
  console.error(
    `FAIL: cannot read forge artifact (${ARTIFACT}). Run \`forge build\` in on-chain/ first. (${e.message})`,
  );
  process.exit(1);
}
const code = artifact?.bytecode?.object;
if (!code || !code.startsWith('0x')) {
  console.error('FAIL: artifact has no bytecode.object');
  process.exit(1);
}
console.log(`EVM creation bytecode: ${(code.length - 2) / 2} bytes`);

const provider = new WsProvider(RPC);
const api = await ApiPromise.create({provider, noInitWarn: true});
const alice = new Keyring({type: 'sr25519'}).addFromUri('//Alice');

// Arg order verified against api.tx.revive.instantiateWithCode.meta:
//   (value, weightLimit, storageDepositLimit, code, data, salt)
// refTime/proofSize kept under Paseo AH's per-extrinsic limits.
const tx = api.tx.revive.instantiateWithCode(
  0,
  {refTime: 1_000_000_000_000n, proofSize: 4_000_000n},
  10_000_000_000_000n,
  code,
  '0x', // no constructor args
  null, // salt: None → CREATE1 (deployer + nonce)
);

let done = false;
let failed = null;
let contract = null;
let unsub = () => {};
try {
  unsub = await tx.signAndSend(alice, ({status, dispatchError, events}) => {
    if (dispatchError) {
      failed = dispatchError.isModule
        ? JSON.stringify(api.registry.findMetaError(dispatchError.asModule))
        : dispatchError.toString();
    }
    for (const {event} of events) {
      if (event.section === 'system' && event.method === 'ExtrinsicFailed') {
        const [de] = event.data;
        failed = de.isModule
          ? JSON.stringify(api.registry.findMetaError(de.asModule))
          : de.toString();
      }
      if (event.section === 'revive' && event.method === 'Instantiated') {
        const d = event.data.toJSON();
        contract = Array.isArray(d) ? d[1] : d && d.contract;
      }
    }
    if (status.isInBlock || status.isFinalized) done = true;
  });
  // chopsticks builds a block on demand.
  await provider.send('dev_newBlock', [{count: 1}]);
  for (let i = 0; i < 60 && !done; i++) await new Promise((r) => setTimeout(r, 500));
} catch (e) {
  failed = `submit rejected: ${e.message || e}`;
}
unsub();

if (failed) {
  console.error(`FAIL: instantiateWithCode — ${failed}`);
  await api.disconnect();
  process.exit(1);
}
if (!contract) {
  console.error(`FAIL: no Revive.Instantiated event observed (inBlock=${done})`);
  await api.disconnect();
  process.exit(1);
}
console.log(`DEPLOYED_H160=${contract}`);
await api.disconnect();
process.exit(0);
