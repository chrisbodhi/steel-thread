resource "aws_secretsmanager_secret" "zoo_token" {
  name        = var.secret_name
  description = var.description

  tags = {
    Name        = var.secret_name
    Environment = var.environment
  }
}

# Note: The actual secret value must be set manually or via separate process
# This prevents sensitive data from being in terraform state
resource "aws_secretsmanager_secret_version" "zoo_token" {
  count         = var.secret_string != "" ? 1 : 0
  secret_id     = aws_secretsmanager_secret.zoo_token.id
  secret_string = var.secret_string
}
