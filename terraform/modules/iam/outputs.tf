output "user_name" {
  description = "Name of the IAM user for Lightsail"
  value       = aws_iam_user.lightsail.name
}

output "user_arn" {
  description = "ARN of the IAM user for Lightsail"
  value       = aws_iam_user.lightsail.arn
}
