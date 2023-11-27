type LogStorePayloadType = {
	logStoreStreamId: string; // Assuming it's always a string in the format '0x...' for the logStoreStreamId
	logStoreChainId: string; // Assuming it's always a string
	logStoreChannelId: string; // Assuming it's always a string

	address: string; // Assuming it's always a string representing an Ethereum address
	blockHash: string; // Assuming it's always a string representing a block hash
	data: string; // Assuming it's always a string representing raw data
	index: number; // Assuming it's always a number representing transaction index
	topics: string[]; // Assuming it's always an array of strings representing raw topics
	transactionHash: string; // Assuming it's always a string representing a transaction hash
};
