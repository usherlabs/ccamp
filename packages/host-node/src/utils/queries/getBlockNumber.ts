export const buildBlockNumberQuery = (blockNumber: string) => `
    SELECT hash, "number", parent_hash, data
    FROM chain1.blocks
    WHERE number = ${blockNumber}
`;
