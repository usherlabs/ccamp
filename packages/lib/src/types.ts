import { _SERVICE as _DC_SERVICE } from '@ccamp/canisters/src/declarations/data_collection/data_collection.did';
import { _SERVICE as _PDC_SERVICE } from '@ccamp/canisters/src/declarations/protocol_data_collection/protocol_data_collection.did';
import { _SERVICE as _REMITTANCE_SERVICE } from '@ccamp/canisters/src/declarations/remittance/remittance.did';
import { Actor, ActorSubclass } from '@dfinity/agent';

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

export type DataCollectionCanister = ActorSubclass<_DC_SERVICE>;
export type ProtocolDataCollectionCanister = ActorSubclass<_PDC_SERVICE>;
export type RemittanceCanister = ActorSubclass<_REMITTANCE_SERVICE>;
