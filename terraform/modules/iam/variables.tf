variable "project_name" {
  description = "Project name for resource naming"
  type        = string
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "prod"
}

variable "s3_bucket_arn" {
  description = "ARN of the S3 bucket for cached files"
  type        = string
}

variable "dynamodb_table_arn" {
  description = "ARN of the DynamoDB cache table"
  type        = string
}
