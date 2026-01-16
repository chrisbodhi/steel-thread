variable "bucket_name" {
  description = "Name of the S3 bucket"
  type        = string
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "prod"
}

variable "lifecycle_rules" {
  description = "S3 lifecycle rules for cost optimization"
  type = object({
    transition_to_glacier_days = number
    expiration_days            = number
  })
  default = {
    transition_to_glacier_days = 90
    expiration_days            = 365
  }
}
