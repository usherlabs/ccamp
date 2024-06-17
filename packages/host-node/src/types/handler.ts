import { CACHE_SUFFIXES } from "@/utils/constants"

export type NotificationResponseMessage = {
    length: number,
    processId: number,
    channel: string,
    payload: string,
    name: string
}