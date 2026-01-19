terraform {
  required_version = ">= 1.2"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.92"
    }
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project   = "platerator"
      ManagedBy = "Terraform"
    }
  }
}

locals {
  project_slug = "platerator"
}

# S3 Bucket for generated STEP and glTF files
module "s3" {
  source = "./modules/s3"

  bucket_name = "${local.project_slug}-generated-files-${var.aws_region}"
  environment = var.environment

  lifecycle_rules = {
    transition_to_glacier_days = 90
    expiration_days            = 365
  }
}

# DynamoDB table for caching plate configurations
module "dynamodb" {
  source = "./modules/dynamodb"

  table_name                    = "${local.project_slug}-cache"
  hash_key                      = "plate_hash"
  billing_mode                  = "PAY_PER_REQUEST"
  ttl_enabled                   = true
  ttl_attribute                 = "ttl"
  enable_point_in_time_recovery = false
  environment                   = var.environment
}

# Secrets Manager for zoo CLI token
module "secrets" {
  source = "./modules/secrets"

  secret_name = "${local.project_slug}/zoo-token"
  description = "Zoo CLI authentication token for CAD generation"
  environment = var.environment
}

# CloudWatch logs for Lightsail
module "cloudwatch" {
  source = "./modules/cloudwatch"

  log_group_name = "/aws/lightsail/${local.project_slug}"
  retention_days = var.log_retention_days
  project_name   = local.project_slug
  region         = var.aws_region
  environment    = var.environment
}

# Lightsail instance
module "lightsail" {
  source = "./modules/lightsail"

  instance_name     = local.project_slug
  availability_zone = "${var.aws_region}a"
  blueprint_id      = "ubuntu_22_04"
  bundle_id         = "nano_3_0"  # $3.50/month

  s3_bucket_name = module.s3.bucket_name
  dynamodb_table = module.dynamodb.table_name
  aws_region     = var.aws_region
  environment    = var.environment
  ssh_public_key = file(pathexpand(var.ssh_public_key_path))
}
