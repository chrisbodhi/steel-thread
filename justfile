# Development: run both servers
dev: build-wasm
    #!/usr/bin/env bash
    echo "Starting API and frontend servers..."
    echo "API will run on http://localhost:3030"
    echo "Frontend will run on http://localhost:3000"
    trap 'kill 0' INT
    cargo run -p web &
    cd frontend && bun dev &
    wait

# Run just the Bun frontend dev server
dev-frontend: build-wasm
    cd frontend && bun dev

# Build WASM validation module for frontend
build-wasm:
    wasm-pack build crates/validation --target web --out-dir ../../frontend/src/wasm-validation --no-typescript

# Build frontend for production
build: build-wasm
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
    rm -rf frontend/src/wasm-validation

# Stop any running dev servers
stop:
    #!/usr/bin/env bash
    echo "Stopping any running dev servers..."
    pkill -f "cargo run -p web" || true
    pkill -f "bun.*dev" || true
    echo "Servers stopped"

# AWS Deployment Commands

# Initialize Terraform
tf-init:
    #!/usr/bin/env bash
    export AWS_REGION=us-east-1
    cd terraform && terraform init

# Validate Terraform configuration
tf-validate:
    #!/usr/bin/env bash
    export AWS_REGION=us-east-1
    cd terraform && terraform validate

# Plan Terraform changes (does not apply)
tf-plan:
    #!/usr/bin/env bash
    export AWS_REGION=us-east-1
    cd terraform && terraform plan -out=platerator.tfplan

# Apply Terraform plan (creates AWS infrastructure)
tf-apply:
    #!/usr/bin/env bash
    export AWS_REGION=us-east-1
    echo "⚠️  This will create AWS resources and incur costs!"
    echo "Press Ctrl+C to cancel, or Enter to continue..."
    read
    cd terraform && terraform apply "platerator.tfplan"

# Destroy all AWS infrastructure
tf-destroy:
    #!/usr/bin/env bash
    export AWS_REGION=us-east-1
    echo "⚠️  WARNING: This will DELETE all AWS resources including S3 data!"
    echo "Press Ctrl+C to cancel, or Enter to continue..."
    read
    cd terraform && terraform destroy

# Set zoo CLI token in AWS Secrets Manager
set-zoo-token:
    #!/usr/bin/env bash
    set -e
    export AWS_REGION=us-east-1
    SECRET_ARN=$(cd terraform && terraform output -raw zoo_token_secret_arn 2>&1)
    if [ -z "$SECRET_ARN" ] || [[ "$SECRET_ARN" == *"Warning"* ]] || [[ "$SECRET_ARN" == *"No outputs"* ]]; then
        echo "❌ No Secrets Manager ARN found. Run 'just tf-apply' first."
        exit 1
    fi
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

# Deploy zoo token to Lightsail instance
deploy-zoo-token:
    #!/usr/bin/env bash
    set -e
    export AWS_REGION=us-east-1

    # Get Lightsail IP
    LIGHTSAIL_IP=$(cd terraform && terraform output -raw lightsail_public_ip 2>&1)
    if [ -z "$LIGHTSAIL_IP" ] || [[ "$LIGHTSAIL_IP" == *"Warning"* ]] || [[ "$LIGHTSAIL_IP" == *"No outputs"* ]]; then
        echo "❌ No Lightsail instance found. Run 'just tf-apply' first."
        exit 1
    fi

    # Get token from Secrets Manager
    ZOO_TOKEN=$(aws secretsmanager get-secret-value \
        --secret-id platerator/zoo-token \
        --query SecretString \
        --output text)

    if [ -z "$ZOO_TOKEN" ]; then
        echo "❌ Could not retrieve zoo token from Secrets Manager."
        echo "Run 'just set-zoo-token' first."
        exit 1
    fi

    # Deploy to instance as .env file for systemd EnvironmentFile
    echo "Deploying zoo token to instance..."
    echo "ZOO_API_TOKEN=$ZOO_TOKEN" | ssh ubuntu@$LIGHTSAIL_IP 'sudo bash -c "cat > /opt/platerator/.env && chmod 600 /opt/platerator/.env"'

    # Restart service to pick up the token
    ssh ubuntu@$LIGHTSAIL_IP 'sudo systemctl restart platerator'

    echo "✅ Zoo token deployed and service restarted"

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
    set -e  # Exit on any error
    export AWS_REGION=us-east-1

    if [ ! -f platerator-*.tar.gz ]; then
        echo "❌ No build artifact found."
        echo "Run 'just download-build' or wait for GitHub Actions to finish building."
        exit 1
    fi

    echo "📋 Checking Terraform infrastructure..."
    LIGHTSAIL_IP=$(cd terraform && terraform output -raw lightsail_public_ip 2>&1)

    if [ -z "$LIGHTSAIL_IP" ] || [[ "$LIGHTSAIL_IP" == *"Warning"* ]] || [[ "$LIGHTSAIL_IP" == *"No outputs"* ]]; then
        echo "❌ No Lightsail instance found in Terraform state."
        echo ""
        echo "It looks like the infrastructure hasn't been created yet."
        echo "Please run the following commands first:"
        echo "  1. just tf-plan"
        echo "  2. just tf-apply"
        echo "  3. just set-zoo-token"
        echo ""
        echo "After the infrastructure is ready (~5 minutes), you can deploy with:"
        echo "  just deploy"
        exit 1
    fi

    echo "✓ Found Lightsail instance: $LIGHTSAIL_IP"
    echo ""
    echo "📤 Uploading to Lightsail ($LIGHTSAIL_IP)..."
    scp platerator-*.tar.gz ubuntu@$LIGHTSAIL_IP:/tmp/

    echo "🚀 Deploying on server..."
    ssh ubuntu@$LIGHTSAIL_IP 'sudo /usr/local/bin/deploy-platerator'

    echo ""
    echo "✅ Deployment complete!"
    echo "🎉 Visit: http://$LIGHTSAIL_IP"

# Full deployment workflow: download + deploy
deploy: download-build deploy-lightsail

# SSH into Lightsail instance
ssh:
    #!/usr/bin/env bash
    set -e
    export AWS_REGION=us-east-1
    LIGHTSAIL_IP=$(cd terraform && terraform output -raw lightsail_public_ip 2>&1)
    if [ -z "$LIGHTSAIL_IP" ] || [[ "$LIGHTSAIL_IP" == *"Warning"* ]] || [[ "$LIGHTSAIL_IP" == *"No outputs"* ]]; then
        echo "❌ No Lightsail instance found. Run 'just tf-apply' first."
        exit 1
    fi
    ssh ubuntu@$LIGHTSAIL_IP

# Check Lightsail instance status
status:
    #!/usr/bin/env bash
    set -e
    export AWS_REGION=us-east-1
    LIGHTSAIL_IP=$(cd terraform && terraform output -raw lightsail_public_ip 2>&1)
    if [ -z "$LIGHTSAIL_IP" ] || [[ "$LIGHTSAIL_IP" == *"Warning"* ]] || [[ "$LIGHTSAIL_IP" == *"No outputs"* ]]; then
        echo "❌ No Lightsail instance found. Run 'just tf-apply' first."
        exit 1
    fi

    echo "Lightsail Instance Status:"
    echo "  IP: $LIGHTSAIL_IP"
    echo "  URL: http://$LIGHTSAIL_IP"
    echo ""
    echo "Service status:"
    ssh ubuntu@$LIGHTSAIL_IP 'systemctl status platerator --no-pager'

# View application logs
logs:
    #!/usr/bin/env bash
    set -e
    export AWS_REGION=us-east-1
    LIGHTSAIL_IP=$(cd terraform && terraform output -raw lightsail_public_ip 2>&1)
    if [ -z "$LIGHTSAIL_IP" ] || [[ "$LIGHTSAIL_IP" == *"Warning"* ]] || [[ "$LIGHTSAIL_IP" == *"No outputs"* ]]; then
        echo "❌ No Lightsail instance found. Run 'just tf-apply' first."
        exit 1
    fi
    echo "📋 Tailing Platerator logs (Ctrl+C to stop)..."
    ssh ubuntu@$LIGHTSAIL_IP 'journalctl -u platerator -f'

# Show deployment info
info:
    #!/usr/bin/env bash
    export AWS_REGION=us-east-1
    cd terraform && terraform output

# List available recipes
default:
    @just --list
