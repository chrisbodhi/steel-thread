output "service_id" {
  description = "ID of the App Runner service"
  value       = aws_apprunner_service.main.service_id
}

output "service_arn" {
  description = "ARN of the App Runner service"
  value       = aws_apprunner_service.main.arn
}

output "service_url" {
  description = "URL of the App Runner service (with HTTPS)"
  value       = "https://${aws_apprunner_service.main.service_url}"
}

output "service_status" {
  description = "Current status of the service"
  value       = aws_apprunner_service.main.status
}
