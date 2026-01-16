output "table_id" {
  description = "ID of the DynamoDB table"
  value       = aws_dynamodb_table.cache.id
}

output "table_arn" {
  description = "ARN of the DynamoDB table"
  value       = aws_dynamodb_table.cache.arn
}

output "table_name" {
  description = "Name of the DynamoDB table"
  value       = aws_dynamodb_table.cache.name
}
