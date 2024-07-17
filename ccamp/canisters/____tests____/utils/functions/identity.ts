import { HttpAgent } from '@dfinity/agent';
import { Secp256k1KeyIdentity } from '@dfinity/identity-secp256k1';
import fetch from 'isomorphic-fetch';

// generate an agent from the privatekey

export const generateRandomIdentity = () => Secp256k1KeyIdentity.generate();

export const fetchLocalIdentity = () =>
	Secp256k1KeyIdentity.fromSeedPhrase(process.env.SEED_PHRASE);

export function getAgent({ identity } = { identity: fetchLocalIdentity() }) {
	const RPC_URL = 'http://127.0.0.1:4943';

	const agent = new HttpAgent({
		identity: identity,
		host: RPC_URL,
		fetch,
	});

	return agent;
}
