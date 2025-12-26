CREATE OR REPLACE FUNCTION notify_sensor_insert()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify('sensor_data', NEW.id::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER sensor_data_insert_trigger
    AFTER INSERT ON sensor_data
    FOR EACH ROW
    EXECUTE FUNCTION notify_sensor_insert();
