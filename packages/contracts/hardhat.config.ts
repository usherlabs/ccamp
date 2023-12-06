import '@nomicfoundation/hardhat-toolbox';
import '@nomiclabs/hardhat-ethers';
import '@openzeppelin/hardhat-upgrades';
import '@typechain/hardhat';
import { config as dotenvConfig } from 'dotenv';
import { HardhatUserConfig } from 'hardhat/config';
import { resolve } from 'path';

dotenvConfig({ path: resolve(__dirname, './.env') });

const mnemonic: string | undefined = process.env.HARDHAT_MNEMONIC;
const infuraKey: string | undefined = process.env.HARDHAT_INFURA_KEY;

if (!mnemonic)
	throw new Error('Please provide a valid mnemonic as an env variable');
if (!infuraKey)
	throw new Error('Please provide a valid infura key as an env variable');

const chain = {
	sepolia: {
		chainId: 11155111,
		url: `https://sepolia.infura.io/v3/${infuraKey}`,
	},
	goerli: {
		chainId: 5,
		url: `https://goerli.infura.io/v3/${infuraKey}`,
	},
	polygon: {
		chainId: 137,
		url: `https://polygon-mainnet.infura.io/v3/${infuraKey}`,
	},
};

const config: HardhatUserConfig = {
	solidity: '0.8.18',
	networks: {
		hardhat: {
			accounts: {
				mnemonic,
			}
		},
		...chain,
	},
};

export default config;
