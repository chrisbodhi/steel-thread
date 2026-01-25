variable "aws_region" {
  description = "AWS region for deployment"
  type        = string
  default     = "us-east-1"
}

variable "environment" {
  description = "Environment name (e.g., dev, staging, prod)"
  type        = string
  default     = "prod"
}

variable "image_tag" {
  description = "Docker image tag to deploy"
  type        = string
  default     = "latest"
}

variable "log_level" {
  description = "Rust application log level"
  type        = string
  default     = "info"
}

variable "log_retention_days" {
  description = "Number of days to retain CloudWatch logs"
  type        = number
  default     = 7
}

variable "app_runner_cpu" {
  description = "CPU units for App Runner (256, 512, 1024, 2048, 4096)"
  type        = string
  default     = "1024"
}

variable "app_runner_memory" {
  description = "Memory for App Runner (2 GB, 3 GB, 4 GB, 6 GB, 8 GB, 10 GB, 12 GB)"
  type        = string
  default     = "2 GB"
}

variable "min_instances" {
  description = "Minimum number of App Runner instances"
  type        = number
  default     = 1
}

variable "max_instances" {
  description = "Maximum number of App Runner instances"
  type        = number
  default     = 3
}

variable "max_concurrent_requests" {
  description = "Maximum concurrent requests per instance before scaling"
  type        = number
  default     = 10
}

variable "ssh_public_key_path" {
  description = "Path to SSH public key file for Lightsail instance access"
  type        = string
  default     = "~/.ssh/id_ed25519.pub"
}
