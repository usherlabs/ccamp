import environment from "@/config/env"

const IMPORTANT_PARAMETERS = ["chainId", "postgresConnectionString"];

export const isConfigValid = () => 
    IMPORTANT_PARAMETERS.every(
        (param) => (environment as Record<string, string>)[param]
    )