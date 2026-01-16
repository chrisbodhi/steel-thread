# Development: run both servers
dev:
    #!/usr/bin/env bash
    echo "Starting API and frontend servers..."
    echo "API will run on http://localhost:3030"
    echo "Frontend will run on http://localhost:3000"
    trap 'kill 0' INT
    cargo run -p web &
    cd frontend && bun dev &
    wait

# Run just the Bun frontend dev server
dev-frontend:
    cd frontend && bun dev

# Build frontend for production
build:
    cd frontend && bun run build

# Build everything for production
build-release: build
    cargo build -p web --release

# Run all tests
test:
    cargo test

# Clean build artifacts
clean:
    cargo clean
    rm -rf crates/web/dist
    rm -rf frontend/node_modules/.cache

# AWS Deployment Commands

# Initialize Terraform
tf-init:
    cd terraform && terraform init

# Validate Terraform configuration
tf-validate:
    cd terraform && terraform validate

# Plan Terraform changes (does not apply)
tf-plan:
    cd terraform && terraform plan -out=platerator.tfplan

# Apply Terraform plan (creates AWS infrastructure)
tf-apply:
    #!/usr/bin/env bash
    echo "⚠️  This will create AWS resources and incur costs!"
    echo "Press Ctrl+C to cancel, or Enter to continue..."
    read
    cd terraform && terraform apply "platerator.tfplan"

# Destroy all AWS infrastructure
tf-destroy:
    #!/usr/bin/env bash
    echo "⚠️  WARNING: This will DELETE all AWS resources including S3 data!"
    echo "Press Ctrl+C to cancel, or Enter to continue..."
    read
    cd terraform && terraform destroy

# Set zoo CLI token in AWS Secrets Manager
set-zoo-token:
    #!/usr/bin/env bash
    SECRET_ARN=$(cd terraform && terraform output -raw zoo_token_secret_arn)
    ZOO_TOKEN=$(grep 'token =' ~/.config/zoo/hosts.toml | cut -d'"' -f2)
    if [ -z "$ZOO_TOKEN" ]; then
        echo "❌ Could not find zoo token in ~/.config/zoo/hosts.toml"
        exit 1
    fi
    echo "Setting zoo token in Secrets Manager..."
    aws secretsmanager put-secret-value \
        --secret-id "$SECRET_ARN" \
        --secret-string "$ZOO_TOKEN"
    echo "✅ Zoo token set successfully"

# Download latest build from GitHub Actions
download-build:
    #!/usr/bin/env bash
    echo "📥 Downloading latest build from GitHub Actions..."
    gh run download --name platerator-binary
    echo "✅ Downloaded to current directory"
    ls -lh platerator-*.tar.gz

# Deploy to Lightsail instance
deploy-lightsail:
    #!/usr/bin/env bash
    if [ ! -f platerator-*.tar.gz ]; then
        echo "❌ No build artifact found."
        echo "Run 'just download-build' or wait for GitHub Actions to finish building."
        exit 1
    fi

    LIGHTSAIL_IP=$(cd terraform && terraform output -raw lightsail_public_ip)

    echo "📤 Uploading to Lightsail ($LIGHTSAIL_IP)..."
    scp platerator-*.tar.gz root@$LIGHTSAIL_IP:/tmp/

    echo "🚀 Deploying on server..."
    ssh root@$LIGHTSAIL_IP '/usr/local/bin/deploy-platerator'

    echo ""
    echo "✅ Deployment complete!"
    echo "🎉 Visit: http://$LIGHTSAIL_IP"

# Full deployment workflow: download + deploy
deploy: download-build deploy-lightsail

# SSH into Lightsail instance
ssh:
    #!/usr/bin/env bash
    LIGHTSAIL_IP=$(cd terraform && terraform output -raw lightsail_public_ip)
    ssh root@$LIGHTSAIL_IP

# Check Lightsail instance status
status:
    #!/usr/bin/env bash
    LIGHTSAIL_IP=$(cd terraform && terraform output -raw lightsail_public_ip 2>/dev/null)
    if [ -z "$LIGHTSAIL_IP" ]; then
        echo "❌ No Lightsail instance found. Run 'just tf-apply' first."
        exit 1
    fi

    echo "Lightsail Instance Status:"
    echo "  IP: $LIGHTSAIL_IP"
    echo "  URL: http://$LIGHTSAIL_IP"
    echo ""
    echo "Service status:"
    ssh root@$LIGHTSAIL_IP 'systemctl status platerator --no-pager'

# View application logs
logs:
    #!/usr/bin/env bash
    LIGHTSAIL_IP=$(cd terraform && terraform output -raw lightsail_public_ip)
    echo "📋 Tailing Platerator logs (Ctrl+C to stop)..."
    ssh root@$LIGHTSAIL_IP 'journalctl -u platerator -f'

# Show deployment info
info:
    #!/usr/bin/env bash
    cd terraform && terraform output

# List available recipes
default:
    @just --list
