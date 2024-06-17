import { ENV } from "@ccamp/lib";

const environment = {
  env: ENV.dev,
  postgresConnectionString: process.env.POSTGRES_CONNECTION_URL,
  evmPrivateKey: process.env.EVM_PRIVATE_KEY,
  logstoreStreamId: process.env.LOGSTORE_STREAM_ID,
  contractAddress: process.env.CONTRACT_ADDRESS,
  chainId: process.env.CHAIN_ID,
  topicStream: "0xeb21022d952e5de09c30bfda9e6352ffa95f67be/topics",
  responseTreshold: 1, //number of required responses before submitting to the ccamp
  logLevel: process.env.LOG_LEVEL || "info",
  redisConnectionString:
    process.env.REDIS_CONNECTION_STRING || "redis://localhost:6379",
};

export default environment;
