# IAM User for Lightsail instance to access S3 and DynamoDB
resource "aws_iam_user" "lightsail" {
  name = "${var.project_name}-lightsail"

  tags = {
    Name        = "${var.project_name}-lightsail"
    Environment = var.environment
  }
}

# S3 access policy
resource "aws_iam_user_policy" "s3_access" {
  name = "s3-cache-access"
  user = aws_iam_user.lightsail.name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = [
        "s3:PutObject",
        "s3:GetObject",
        "s3:HeadObject"
      ]
      Resource = ["${var.s3_bucket_arn}/*"]
    }]
  })
}

# DynamoDB access policy
resource "aws_iam_user_policy" "dynamodb_access" {
  name = "dynamodb-cache-access"
  user = aws_iam_user.lightsail.name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = [
        "dynamodb:GetItem",
        "dynamodb:PutItem"
      ]
      Resource = [var.dynamodb_table_arn]
    }]
  })
}
