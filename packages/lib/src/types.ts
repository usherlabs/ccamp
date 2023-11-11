import { data_collection } from '@ccamp/canisters/src/declarations/data_collection';
import { protocol_data_collection } from '@ccamp/canisters/src/declarations/protocol_data_collection';
import { remittance } from '@ccamp/canisters/src/declarations/remittance';
import { Actor } from '@dfinity/agent';

export type Environment = 'prod' | 'dev';

export type CanisterType =
	| 'dataCollection'
	| 'protocolDataCollection'
	| 'remittance';

export type CanisterIds = {
	dataCollection: string;
	protocolDataCollection: string;
	remittance: string;
};

export type CanisterActors = {
	dataCollection: Actor;
	protocolDataCollection: Actor;
	remittance: Actor;
};

export type DataCollectionCanister = typeof data_collection;
export type ProtocolDataCollectionCanister = typeof protocol_data_collection;
export type RemittanceCanister = typeof remittance;
