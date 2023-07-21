import '@nomicfoundation/hardhat-toolbox';
import '@nomiclabs/hardhat-ethers';
import '@openzeppelin/hardhat-upgrades';
import '@typechain/hardhat';
import { config as dotenvConfig } from 'dotenv';
import { HardhatUserConfig } from 'hardhat/config';
import { resolve } from 'path';

dotenvConfig({ path: resolve(__dirname, './.env') });

const mnemonic: string | undefined = process.env.MNEMONIC;
const chainIds = {
	sepolia: 11155111,
};


const config: HardhatUserConfig = {
	solidity: '0.8.18',
	networks: {
		hardhat: {
			accounts: {
				mnemonic,
			},
			chainId: chainIds['sepolia'],
			forking: {
				url: String(process.env.FORK_URL),
				blockNumber: 8800522,
			},
		},
	},
};

export default config;
