variable "table_name" {
  description = "Name of the DynamoDB table"
  type        = string
}

variable "hash_key" {
  description = "Attribute to use as the hash (partition) key"
  type        = string
  default     = "plate_hash"
}

variable "billing_mode" {
  description = "Billing mode for the table (PROVISIONED or PAY_PER_REQUEST)"
  type        = string
  default     = "PAY_PER_REQUEST"
}

variable "ttl_enabled" {
  description = "Whether to enable TTL for automatic item expiration"
  type        = bool
  default     = true
}

variable "ttl_attribute" {
  description = "Name of the table attribute to store the TTL timestamp"
  type        = string
  default     = "ttl"
}

variable "enable_point_in_time_recovery" {
  description = "Whether to enable point-in-time recovery (costs extra)"
  type        = bool
  default     = false
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "prod"
}
