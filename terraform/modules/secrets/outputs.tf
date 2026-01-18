output "secret_id" {
  description = "ID of the secret"
  value       = aws_secretsmanager_secret.zoo_token.id
}

output "secret_arn" {
  description = "ARN of the secret"
  value       = aws_secretsmanager_secret.zoo_token.arn
}

output "secret_name" {
  description = "Name of the secret"
  value       = aws_secretsmanager_secret.zoo_token.name
}
