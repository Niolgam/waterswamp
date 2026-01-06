-- Add down migration script here
DROP TABLE IF EXISTS requisition_items;
DROP TABLE IF EXISTS requisitions;
DROP TABLE IF EXISTS stock_movements;
DROP TABLE IF EXISTS warehouse_stocks;
DROP TABLE IF EXISTS warehouses;

DROP TYPE IF EXISTS requisition_status;
DROP TYPE IF EXISTS movement_type;
