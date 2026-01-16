variable "log_group_name" {
  description = "Name of the CloudWatch log group"
  type        = string
}

variable "retention_days" {
  description = "Number of days to retain logs"
  type        = number
  default     = 7
}

variable "project_name" {
  description = "Project name for dashboard naming"
  type        = string
}

variable "region" {
  description = "AWS region for metrics"
  type        = string
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "prod"
}
