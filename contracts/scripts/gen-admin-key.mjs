// Derive an sr25519 admin key for the ODS app (ODS_ADMIN_SEED).
//
// Usage:
//   node gen-admin-key.mjs                 # generate a fresh admin key
//   node gen-admin-key.mjs "<mnemonic>"    # derive from an existing recovery phrase
//
// Prints the 32-byte sr25519 secret seed (what `ODS_ADMIN_SEED` expects — the
// same value subkey calls "Secret seed") plus the account's SS58 address (fund
// this on Asset Hub) and public key. Deps come from this dir's package.json
// (`npm ci` first). For testnet use a dedicated key, not your main wallet's.

import {
  cryptoWaitReady,
  mnemonicGenerate,
  mnemonicToMiniSecret,
  sr25519PairFromSeed,
  encodeAddress,
} from '@polkadot/util-crypto';
import {u8aToHex} from '@polkadot/util';

await cryptoWaitReady();

const arg = process.argv[2];
const mnemonic = arg && arg.trim().length > 0 ? arg.trim() : mnemonicGenerate();
const generated = !arg;

// 32-byte mini-secret == sr25519 "secret seed" == subxt_signer::sr25519::Keypair::from_secret_key input.
const seed = mnemonicToMiniSecret(mnemonic);
const pair = sr25519PairFromSeed(seed);

console.log('');
if (generated) {
  console.log('Generated a fresh admin key. Store the mnemonic securely.');
  console.log('mnemonic        :', mnemonic);
}
console.log('ODS_ADMIN_SEED  : ' + u8aToHex(seed)); // 0x + 64 hex
console.log('public key      : ' + u8aToHex(pair.publicKey));
console.log('SS58 (fund this): ' + encodeAddress(pair.publicKey, 42));
console.log('');
console.log('export ODS_ADMIN_SEED=' + u8aToHex(seed));
