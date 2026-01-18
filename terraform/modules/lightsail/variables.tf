variable "instance_name" {
  description = "Name of the Lightsail instance"
  type        = string
}

variable "availability_zone" {
  description = "Availability zone for the instance"
  type        = string
  default     = "us-east-1a"
}

variable "blueprint_id" {
  description = "Lightsail blueprint (OS image)"
  type        = string
  default     = "ubuntu_22_04"
}

variable "bundle_id" {
  description = "Lightsail bundle (instance size)"
  type        = string
  default     = "nano_3_0"  # $3.50/month
}

variable "s3_bucket_name" {
  description = "S3 bucket name for generated files"
  type        = string
}

variable "dynamodb_table" {
  description = "DynamoDB table name for caching"
  type        = string
}

variable "aws_region" {
  description = "AWS region"
  type        = string
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "prod"
}
