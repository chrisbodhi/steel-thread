variable "secret_name" {
  description = "Name of the secret in Secrets Manager"
  type        = string
}

variable "description" {
  description = "Description of the secret"
  type        = string
  default     = "Zoo CLI authentication token"
}

variable "secret_string" {
  description = "The secret value (optional, can be set manually after creation)"
  type        = string
  default     = ""
  sensitive   = true
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "prod"
}
