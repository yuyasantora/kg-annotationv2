-- Fix bucket names
UPDATE images 
SET s3_bucket = 'kgbacket' 
WHERE s3_bucket IN ('kg-annotation-bucket', 'test-bucket');