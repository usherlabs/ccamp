// run the script to request remittance
import { requestRemittance } from './dist/remittance.js';

(async function main() {
    // random pk gotten from vanity.eth
	const pk = 'a398177fe8519dd130a16a29a4b5825677c71121fc1551868cb6fc10d435b484';
    const response = await requestRemittance(pk);
	console.log({ response });
})();
