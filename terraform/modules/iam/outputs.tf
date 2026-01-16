output "access_role_arn" {
  description = "ARN of the App Runner access role (for ECR)"
  value       = aws_iam_role.apprunner_access_role.arn
}

output "instance_role_arn" {
  description = "ARN of the App Runner instance role (for application runtime)"
  value       = aws_iam_role.apprunner_instance_role.arn
}

output "access_role_name" {
  description = "Name of the App Runner access role"
  value       = aws_iam_role.apprunner_access_role.name
}

output "instance_role_name" {
  description = "Name of the App Runner instance role"
  value       = aws_iam_role.apprunner_instance_role.name
}
