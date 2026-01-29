#!/bin/bash
set -e

# Update system
apt-get update
apt-get upgrade -y

# Install AWS CLI
echo "Installing AWS CLI..."
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
apt-get install -y unzip
unzip -q awscliv2.zip
./aws/install
rm -rf aws awscliv2.zip

# Install zoo CLI from GitHub releases to system-wide location
echo "Installing zoo CLI..."
LATEST_RELEASE=$(curl -s https://api.github.com/repos/KittyCAD/cli/releases/latest | grep -o '"tag_name": *"[^"]*"' | sed 's/"tag_name": *"//;s/"//')
curl -L -o /usr/local/bin/zoo "https://github.com/KittyCAD/cli/releases/download/$${LATEST_RELEASE}/zoo-x86_64-unknown-linux-musl"
chmod +x /usr/local/bin/zoo

# Install Caddy for automatic HTTPS
apt-get install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | tee /etc/apt/sources.list.d/caddy-stable.list
apt-get update
apt-get install -y caddy

# Create application directory
mkdir -p /opt/platerator/kcl
mkdir -p /opt/platerator/dist
chown -R root:root /opt/platerator

# Create systemd service that loads zoo token from EnvironmentFile
cat > /etc/systemd/system/platerator.service <<'SYSTEMD'
[Unit]
Description=Platerator Web Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/platerator
Environment="PORT=8080"
Environment="RUST_LOG=info"
Environment="KCL_SRC_DIR=/opt/platerator/kcl"
Environment="AWS_REGION=${aws_region}"
Environment="S3_BUCKET_NAME=${s3_bucket_name}"
Environment="DYNAMODB_TABLE=${dynamodb_table}"
EnvironmentFile=-/opt/platerator/.env
ExecStart=/opt/platerator/web
Restart=always
RestartSec=5s
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SYSTEMD

# Enable service (don't start yet - no binary)
systemctl daemon-reload
systemctl enable platerator

# Configure Caddy for reverse proxy with automatic HTTPS
cat > /etc/caddy/Caddyfile <<'CADDY'
platerator.newschematic.org {
    reverse_proxy localhost:8080
}
CADDY

# Start Caddy
systemctl enable caddy
systemctl restart caddy

# Create deployment helper script
cat > /usr/local/bin/deploy-platerator <<'DEPLOY'
#!/bin/bash
set -e

# Count tarballs
TARBALL_COUNT=$(ls -1 /tmp/platerator-*.tar.gz 2>/dev/null | wc -l)

if [ "$TARBALL_COUNT" -eq 0 ]; then
    echo "Error: No platerator tarball found in /tmp/"
    exit 1
elif [ "$TARBALL_COUNT" -gt 1 ]; then
    echo "Error: Multiple platerator tarballs found in /tmp/"
    echo "Please remove old tarballs before deploying:"
    ls -lh /tmp/platerator-*.tar.gz
    exit 1
fi

# Get the single tarball
TARBALL=$(ls /tmp/platerator-*.tar.gz)
echo "Found tarball: $TARBALL"

echo "Stopping platerator service..."
systemctl stop platerator || true

echo "Extracting tarball..."
cd /tmp
tar xzf "$TARBALL"

echo "Installing files..."
cp release/web /opt/platerator/
cp -r release/dist/* /opt/platerator/dist/
cp release/kcl/* /opt/platerator/kcl/

echo "Setting permissions..."
chmod +x /opt/platerator/web

echo "Starting platerator service..."
systemctl start platerator

echo "Checking status..."
sleep 2
systemctl status platerator --no-pager

echo "Cleaning up..."
rm -rf /tmp/release /tmp/platerator-*.tar.gz

echo "✅ Deployment complete!"
DEPLOY

chmod +x /usr/local/bin/deploy-platerator

echo "Lightsail instance setup complete!"
echo "Ready for deployment via: scp + ssh deploy-platerator"
