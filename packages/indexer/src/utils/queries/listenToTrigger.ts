import { PG_NOTIFY_EVENT_NAME } from '../constants';

export const LISTEN_TO_TRIGGER_QUERY = `LISTEN "${PG_NOTIFY_EVENT_NAME}"`;
