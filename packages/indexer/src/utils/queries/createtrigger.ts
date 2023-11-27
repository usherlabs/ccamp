import { PG_NOTIFY_EVENT_NAME } from '../constants';

// TODO: Rather than replacing trigger each time,it might be more efficient to check if it exists before creating by combining multiple queryies since there is no CREATE IF NOT EXISTS
export const CREATE_TRIGGER_QUERY =  `
    CREATE OR REPLACE FUNCTION notify_new_index()
    RETURNS trigger AS
    $BODY$
        BEGIN
            PERFORM pg_notify('${PG_NOTIFY_EVENT_NAME}', row_to_json(NEW)::text);
            RETURN NULL;
        END; 
    $BODY$
    LANGUAGE plpgsql VOLATILE
    COST 100;

    CREATE OR REPLACE TRIGGER notify_new_index
    AFTER INSERT
    ON sgd1.canceled_withdraw
    FOR EACH ROW
    EXECUTE PROCEDURE notify_new_index();

    CREATE OR REPLACE TRIGGER notify_new_index
    AFTER INSERT
    ON sgd1.deposited_fund
    FOR EACH ROW
    EXECUTE PROCEDURE notify_new_index();

    CREATE OR REPLACE TRIGGER notify_new_index
    AFTER INSERT
    ON sgd1.withdrawn_fund
    FOR EACH ROW
    EXECUTE PROCEDURE notify_new_index();
`;
