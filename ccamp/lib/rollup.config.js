// Contents of the file /rollup.config.js
import dts from 'rollup-plugin-dts';
import json from '@rollup/plugin-json';
import { nodeResolve } from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';

const config = [
	{
		input: 'dist/src/index.js',
		output: {
			file: 'dist/pkg/ccamp-lib.cjs',
			format: 'cjs',
			sourcemap: true,
			exports: 'named',
		},
		plugins: [nodeResolve(), commonjs(), json()],
	},
	{
		input: 'dist/src/index.d.ts',
		output: {
			file: 'dist/pkg/ccamp-lib.d.ts',
			format: 'es',
		},
		plugins: [nodeResolve(), commonjs(), dts({ respectExternal: true })],
	},
];
export default config;
