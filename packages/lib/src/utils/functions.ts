// Add the '0x' to the start of any string that doesnt begin with it
export const prependKeyWith0x = (privateKey: string) =>
	privateKey.startsWith('0x') ? privateKey : `0x${privateKey}`;
