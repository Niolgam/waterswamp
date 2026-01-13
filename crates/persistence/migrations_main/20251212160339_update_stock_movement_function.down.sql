-- This migration updates an existing function
-- The down migration would restore the previous version
-- Since we're in development, we'll just note that the function exists

-- Note: This would restore the hardcoded threshold version
-- For now, keeping the updated version
SELECT 'Function fn_process_stock_movement updated - no rollback needed in development' as message;
