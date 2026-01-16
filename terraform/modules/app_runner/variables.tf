variable "service_name" {
  description = "Name of the App Runner service"
  type        = string
}

variable "ecr_repository_url" {
  description = "URL of the ECR repository"
  type        = string
}

variable "image_tag" {
  description = "Docker image tag to deploy"
  type        = string
  default     = "latest"
}

variable "access_role_arn" {
  description = "ARN of the IAM role for ECR access"
  type        = string
}

variable "instance_role_arn" {
  description = "ARN of the IAM role for instance runtime"
  type        = string
}

variable "cpu" {
  description = "Number of CPU units (256, 512, 1024, 2048, or 4096)"
  type        = string
  default     = "1024"
}

variable "memory" {
  description = "Amount of memory (2 GB, 3 GB, 4 GB, etc.)"
  type        = string
  default     = "2 GB"
}

variable "port" {
  description = "Port the application listens on"
  type        = string
  default     = "8080"
}

variable "environment_variables" {
  description = "Environment variables for the application"
  type        = map(string)
  default     = {}
}

variable "secrets" {
  description = "Secrets to inject from Secrets Manager"
  type        = map(string)
  default     = {}
}

variable "health_check" {
  description = "Health check configuration"
  type = object({
    path                = string
    protocol            = string
    interval            = number
    timeout             = number
    healthy_threshold   = number
    unhealthy_threshold = number
  })
  default = {
    path                = "/api/health"
    protocol            = "HTTP"
    interval            = 10
    timeout             = 5
    healthy_threshold   = 1
    unhealthy_threshold = 3
  }
}

variable "auto_scaling" {
  description = "Auto-scaling configuration"
  type = object({
    min_size                  = number
    max_size                  = number
    max_concurrent_requests   = number
  })
  default = {
    min_size                = 1
    max_size                = 3
    max_concurrent_requests = 10
  }
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "prod"
}
