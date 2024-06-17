import environment from "@/config/env";
import CCAMPClient, { ENV, ICAFCanister } from "@ccamp/lib";
import * as fs from "fs";

export async function start() {
    const tls_data = fs.readFileSync("./fixtures/twitter_proof.json");

    const env = ENV.dev;
    const evmPrivateKey = "";
    let ccamp = new CCAMPClient(evmPrivateKey, { env });
    const { icafCanister } : { icafCanister : ICAFCanister } = ccamp.getCCampCanisters();
    await icafCanister.verify_tls_data(
        tls_data
    );
}