#!/bin/bash
set -e

# Update system
apt-get update
apt-get upgrade -y

# Install Rust (for future builds if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source /root/.cargo/env
echo 'source /root/.cargo/env' >> /root/.bashrc

# Install zoo CLI
curl -fsSL https://zoo.dev/install.sh | sh
export PATH="/root/.local/bin:$PATH"
echo 'export PATH="/root/.local/bin:$PATH"' >> /root/.bashrc

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

# Create systemd service
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

# Configure Caddy for reverse proxy
cat > /etc/caddy/Caddyfile <<'CADDY'
{
    auto_https off
}

:80 {
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

if [ ! -f /tmp/platerator-*.tar.gz ]; then
    echo "Error: No platerator tarball found in /tmp/"
    exit 1
fi

echo "Stopping platerator service..."
systemctl stop platerator || true

echo "Extracting tarball..."
cd /tmp
tar xzf platerator-*.tar.gz

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
