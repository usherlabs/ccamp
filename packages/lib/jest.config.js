// jest.config.js
module.exports = {
	transform: {
		'^.+\\.[t|j]s$': ['babel-jest', { configFile: './babel-jest.config.js' }],
	},
	modulePathIgnorePatterns: ['<rootDir>/dist/'],
	testTimeout: 300000,
};
