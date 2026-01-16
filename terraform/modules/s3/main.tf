resource "aws_s3_bucket" "generated_files" {
  bucket = var.bucket_name

  lifecycle {
    prevent_destroy = true
  }

  tags = {
    Name        = var.bucket_name
    Environment = var.environment
  }
}

resource "aws_s3_bucket_versioning" "generated_files" {
  bucket = aws_s3_bucket.generated_files.id

  versioning_configuration {
    status = "Disabled"
  }
}

resource "aws_s3_bucket_lifecycle_configuration" "generated_files" {
  bucket = aws_s3_bucket.generated_files.id

  rule {
    id     = "transition-to-glacier"
    status = "Enabled"

    filter {
      prefix = "generated/"
    }

    transition {
      days          = var.lifecycle_rules.transition_to_glacier_days
      storage_class = "GLACIER_IR"
    }

    expiration {
      days = var.lifecycle_rules.expiration_days
    }
  }
}

resource "aws_s3_bucket_public_access_block" "generated_files" {
  bucket = aws_s3_bucket.generated_files.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_cors_configuration" "generated_files" {
  bucket = aws_s3_bucket.generated_files.id

  cors_rule {
    allowed_headers = ["*"]
    allowed_methods = ["GET", "HEAD"]
    allowed_origins = ["*"]
    expose_headers  = ["ETag"]
    max_age_seconds = 3600
  }
}
