# Platerator Terraform Infrastructure

## What's Deployed

This Terraform configuration creates a cost-optimized AWS infrastructure for Platerator:

### Resources

- **Lightsail Instance** - Ubuntu 22.04 nano ($3.50/month) running the Rust/React app
  - SSH key automatically configured from `~/.ssh/id_rsa.pub`
  - Access as `ubuntu` user (not root)
  - AWS region auto-set to `us-east-1`
- **Static IP** - Fixed public IP for the instance
- **S3 Bucket** - Generated STEP/glTF file storage with lifecycle policies
- **DynamoDB Table** - Plate configuration cache (on-demand billing)
- **Secrets Manager** - Zoo CLI token storage
- **CloudWatch** - Logs (7-day retention) + monitoring dashboard

### Cost Estimate

**~$5-8/month** (Lightsail nano instance + S3 storage + DynamoDB on-demand)

## Quick Commands

From project root:

```bash
# Initialize (first time only)
just tf-init

# Plan changes (review before applying)
just tf-plan

# Create infrastructure ⚠️ (incurs costs)
just tf-apply

# Deploy app after infrastructure exists
just deploy
```

## File Structure

```
terraform/
├── main.tf                 # Root configuration
├── variables.tf            # Input variables (customizable)
├── outputs.tf              # Outputs (IP, SSH command)
├── terraform.tfvars.example # Template for custom values
└── modules/
    ├── lightsail/          # Ubuntu instance + static IP
    ├── s3/                 # File storage + lifecycle policies
    ├── dynamodb/           # Cache table + TTL
    ├── secrets/            # Secrets Manager
    └── cloudwatch/         # Logging + dashboard
```

## Customization

Copy and edit `terraform.tfvars.example`:

```bash
cp terraform.tfvars.example terraform.tfvars
vim terraform.tfvars
```

Available settings:
- `aws_region` - AWS region (default: us-east-1)
- `environment` - Environment name (default: prod)
- `log_retention_days` - CloudWatch log retention (default: 7 days)
- `log_level` - Application log level (default: info)

## Terraform Commands

### Via `just` (recommended)

```bash
just tf-init        # Initialize
just tf-validate    # Validate configuration
just tf-plan        # Preview changes
just tf-apply       # Apply changes
just tf-destroy     # Destroy infrastructure
```

### Direct Terraform

```bash
cd terraform
terraform init
terraform plan -out=platerator.tfplan
terraform apply "platerator.tfplan"
terraform destroy
```

## State Management

**Current setup**: Local state (simple for hobby projects)

For production, consider remote state:

```hcl
# Add to main.tf
terraform {
  backend "s3" {
    bucket = "my-terraform-state"
    key    = "platerator/terraform.tfstate"
    region = "us-east-1"
  }
}
```

## Security Notes

- IAM roles follow least-privilege principle
- S3 bucket blocks all public access
- Secrets Manager for sensitive data (zoo token)
- App Runner instance role limited to specific S3/DynamoDB resources

## Outputs

After `terraform apply`, get important values:

```bash
just info  # Show all outputs

# Individual outputs
terraform output lightsail_public_ip
terraform output lightsail_ssh_command
terraform output service_url
terraform output s3_bucket_name
terraform output dynamodb_table_name
```

## Troubleshooting

### "Error: No configuration files"

Run from project root, not terraform directory:
```bash
cd ..  # Go to project root
just tf-plan
```

### "Resource already exists"

Terraform state is out of sync. Import existing resource (example for Lightsail):
```bash
terraform import module.lightsail.aws_lightsail_instance.main platerator
```

### Plan shows unexpected changes

Check if AWS resources were modified outside Terraform (manual changes).

## Next Steps

1. ✅ Infrastructure ready
2. Run `just set-zoo-token` to store zoo CLI credentials in Secrets Manager
3. Run `just deploy-zoo-token` to deploy the token to the instance
4. Wait for Lightsail instance to finish setup (~5 minutes):
   ```bash
   ssh ubuntu@<public-ip> 'tail -f /var/log/cloud-init-output.log'
   ```
4. Deploy the application locally (credentials stay on your machine):
   ```bash
   just deploy
   ```
5. Visit `http://<public-ip>` from the output

See `../DEPLOYMENT.md` for full deployment guide.
