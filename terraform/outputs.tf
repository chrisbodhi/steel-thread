output "lightsail_public_ip" {
  description = "Public IP address of the Lightsail instance"
  value       = module.lightsail.public_ip
}

output "lightsail_ssh_command" {
  description = "SSH command to connect to the instance"
  value       = module.lightsail.ssh_command
}

output "service_url" {
  description = "URL to access Platerator (HTTP)"
  value       = "http://${module.lightsail.public_ip}"
}

output "s3_bucket_name" {
  description = "S3 bucket name for generated files"
  value       = module.s3.bucket_name
}

output "dynamodb_table_name" {
  description = "DynamoDB cache table name"
  value       = module.dynamodb.table_name
}

output "zoo_token_secret_arn" {
  description = "Secrets Manager ARN for zoo token"
  value       = module.secrets.secret_arn
  sensitive   = true
}

output "cloudwatch_dashboard_name" {
  description = "CloudWatch dashboard name"
  value       = module.cloudwatch.dashboard_name
}

output "iam_user_name" {
  description = "IAM user name for Lightsail cache access"
  value       = module.iam.user_name
}

output "lightsail_key_pair_name" {
  description = "SSH key pair name for Lightsail instance"
  value       = module.lightsail.key_pair_name
}

output "next_steps" {
  description = "Next steps after terraform apply"
  value = <<-EOT

    🎉 Platerator infrastructure deployed!

    Next steps:
    1. Set zoo token in Secrets Manager:
       just set-zoo-token

    2. Wait for instance to finish setup (~5 minutes):
       ssh ubuntu@${module.lightsail.public_ip} 'tail -f /var/log/cloud-init-output.log'

    3. Deploy zoo token to instance:
       just deploy-zoo-token

    4. Create access keys for cache (IAM user: ${module.iam.user_name}):
       - Go to AWS Console > IAM > Users > ${module.iam.user_name}
       - Create access key, then SSH to instance and add to /opt/platerator/.env:
         AWS_ACCESS_KEY_ID=xxx
         AWS_SECRET_ACCESS_KEY=xxx

    5. Deploy application:
       # Push code to GitHub (triggers build)
       git push origin master

       # Wait for GitHub Actions to complete
       # Then download and deploy
       just deploy

    6. Visit your app:
       http://${module.lightsail.public_ip}

    7. SSH to instance:
       ${module.lightsail.ssh_command}

    8. View CloudWatch dashboard:
       https://console.aws.amazon.com/cloudwatch/home?region=${var.aws_region}#dashboards:name=${module.cloudwatch.dashboard_name}
  EOT
}
