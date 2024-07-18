import { createActor as createDCACTOR } from '@ccamp/canisters/src/declarations/data_collection';
import { createActor as createPDCActor } from '@ccamp/canisters/src/declarations/protocol_data_collection';
import { createActor as createRemittanceActor } from '@ccamp/canisters/src/declarations/remittance';
import { createActor as createICAFActor } from '@ccamp/canisters/src/declarations/ic_af';
import { CanisterType, Environment } from '../types';

export const ENV = {
	prod: 'prod',
	dev: 'dev',
} as Record<Environment, Environment>;

export const HOSTS = {
	[ENV.dev]: 'http://127.0.0.1:4943',
	[ENV.prod]: 'https://ic0.app',
};

export const canisterActors = {
	dataCollection: createDCACTOR,
	protocolDataCollection: createPDCActor,
	remittance: createRemittanceActor,
	icaf: createICAFActor,
};

type keyType = "DATA_COLLECTION" | "PROTOCOL_DATA_COLLECTION" | "REMITTANCE" | "ICAF";
export const CANISTER_TYPES: Record<keyType, CanisterType> = {
	DATA_COLLECTION: 'dataCollection',
	PROTOCOL_DATA_COLLECTION: 'protocolDataCollection',
	REMITTANCE: 'remittance',
	ICAF: 'icaf',
};
