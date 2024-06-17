import environment from "@/config/env";
import { DBEventPayload, EthereumBlockRow, NotificationResponseMessage } from "@/types";
import { CREATE_TRIGGER_QUERY, LISTEN_TO_TRIGGER_QUERY, buildBlockNumberQuery } from "@/utils/queries";
import CCAMPClient, { ENV, ProtocolDataCollectionCanister } from "@ccamp/lib";
import { Client as DBClient, QueryResult } from "pg";
import { createClient } from "redis";
import logger, { STANDARD_LEVELS } from "simple-node-logger";

export class Client {
    private _cache;
    private _db: DBClient;
    public ccampClient: CCAMPClient;
    public logger: logger.Logger;

    constructor (
        connectionString: string,
        evmPrivateKey: string,
        { env } = { env : ENV.prod }
    ) {
        this._db = new DBClient(connectionString);
        this._cache = createClient({
            url: environment.redisConnectionString,
        });
        this.ccampClient = new CCAMPClient(evmPrivateKey, { env });
        this.logger = logger.createSimpleLogger("ccamp.log");
        this.logger.setLevel(environment.logLevel as STANDARD_LEVELS);

        this._db.connect();
        this._cache.connect();
    }

    public async log(text: string) {
        this.logger.info(text);
    }

    private async _handleValidationData (

    ) {
        // push the data to ccamp
        const { pdcCanister } : { pdcCanister : ProtocolDataCollectionCanister } = this.ccampClient.getCCampCanisters();
        await pdcCanister.process_event(
            JSON.stringify(
                {

                }
            )
        );

        this.log(`proofs successfully pushed to ic_af`);
    }
}