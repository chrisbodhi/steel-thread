output "bucket_id" {
  description = "ID of the S3 bucket"
  value       = aws_s3_bucket.generated_files.id
}

output "bucket_arn" {
  description = "ARN of the S3 bucket"
  value       = aws_s3_bucket.generated_files.arn
}

output "bucket_name" {
  description = "Name of the S3 bucket"
  value       = aws_s3_bucket.generated_files.bucket
}

output "bucket_regional_domain_name" {
  description = "Regional domain name of the S3 bucket"
  value       = aws_s3_bucket.generated_files.bucket_regional_domain_name
}
