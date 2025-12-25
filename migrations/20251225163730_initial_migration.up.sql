-- Add up migration script here



CREATE TABLE sensor_data (
	id SERIAL PRIMARY KEY,
	created_at TIMESTAMP NOT NULL DEFAULT NOW(),
	temperature FLOAT NOT NULL,
	humidity FLOAT NOT NULL,
	pressure FLOAT NOT NULL,
	water_level FLOAT NOT NULL,
	soil_moisture FLOAT NOT NULL
);
