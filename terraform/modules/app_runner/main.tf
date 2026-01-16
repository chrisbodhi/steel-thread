resource "aws_apprunner_service" "main" {
  service_name = var.service_name

  source_configuration {
    authentication_configuration {
      access_role_arn = var.access_role_arn
    }

    image_repository {
      image_identifier      = "${var.ecr_repository_url}:${var.image_tag}"
      image_repository_type = "ECR"

      image_configuration {
        port = var.port

        runtime_environment_variables = merge(
          var.environment_variables,
          {
            # Secrets will be injected via environment variables
            # For now, we'll handle this separately
          }
        )
      }
    }

    auto_deployments_enabled = false
  }

  instance_configuration {
    cpu               = var.cpu
    memory            = var.memory
    instance_role_arn = var.instance_role_arn
  }

  health_check_configuration {
    path                = var.health_check.path
    protocol            = var.health_check.protocol
    interval            = var.health_check.interval
    timeout             = var.health_check.timeout
    healthy_threshold   = var.health_check.healthy_threshold
    unhealthy_threshold = var.health_check.unhealthy_threshold
  }

  auto_scaling_configuration_arn = aws_apprunner_auto_scaling_configuration_version.main.arn

  tags = {
    Name        = var.service_name
    Environment = var.environment
  }
}

resource "aws_apprunner_auto_scaling_configuration_version" "main" {
  auto_scaling_configuration_name = "${var.service_name}-autoscaling"

  min_size = var.auto_scaling.min_size
  max_size = var.auto_scaling.max_size

  max_concurrency = var.auto_scaling.max_concurrent_requests

  tags = {
    Name        = "${var.service_name}-autoscaling"
    Environment = var.environment
  }
}
