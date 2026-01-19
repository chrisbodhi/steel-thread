# Platerator - AWS Deployment Quickstart (Docker-Free!)

## The Workflow

```
Your Machine          GitHub Actions         AWS Lightsail
────────────          ──────────────         ─────────────
git push         →    Build Linux binary
                      Build frontend
                      Package tarball    →   Download artifact
                                             Upload to server
                                             Restart systemd
```

**No Docker required!** Builds happen in GitHub Actions, deployment from your machine using your local AWS credentials.

## Prerequisites

- AWS Account with AWS CLI configured (`aws configure`)
- GitHub CLI (`gh auth login`)
- Terraform installed
- `just` command runner
- zoo CLI token in `~/.config/zoo/hosts.toml`

## First-Time Deployment

### Step 1: Create AWS Infrastructure

```bash
# Initialize Terraform
just tf-init

# Plan infrastructure (review what will be created)
just tf-plan

# Create infrastructure (13 resources, ~$3.50-5/month)
just tf-apply

# Store your zoo token in AWS Secrets Manager
just set-zoo-token

# Deploy the token to the Lightsail instance
just deploy-zoo-token
```

**Cost**: $3.50/month for Lightsail + ~$1-2 for S3/DynamoDB = **~$5/month total**

### Step 2: Wait for Instance Setup

The Lightsail instance needs ~5 minutes to install AWS CLI, zoo CLI, and Caddy.

Check progress:
```bash
LIGHTSAIL_IP=$(cd terraform && terraform output -raw lightsail_public_ip)
ssh ubuntu@$LIGHTSAIL_IP 'tail -f /var/log/cloud-init-output.log'
```

Wait for: "Lightsail instance setup complete!"

### Step 3: Deploy Application

```bash
# Push code to GitHub (triggers build)
git add .
git commit -m "Initial deployment"
git push origin master

# Wait for GitHub Actions (~5 min)
# Check: https://github.com/yourusername/steel-thread/actions

# Download build and deploy
just deploy
```

This will:
1. Download latest build from GitHub Actions
2. SCP tarball to Lightsail
3. Extract and install on server
4. Restart systemd service
5. Display URL

### Step 4: Visit Your App

```bash
# Get URL
just info

# Visit: http://YOUR_LIGHTSAIL_IP
```

---

## Subsequent Deployments

After making code changes:

```bash
# Test locally
just dev

# Push to GitHub (triggers build)
git push origin master

# Wait for build, then deploy
just deploy
```

---

## Daily Commands

```bash
# Deploy latest build
just deploy

# Check service status
just status

# View logs
just logs

# SSH to server
just ssh

# Show all info (IP, URLs, etc.)
just info
```

---

## Monitoring

```bash
# Check if app is running
just status

# View logs in real-time
just logs

# SSH and debug
just ssh
systemctl status platerator
journalctl -u platerator -n 100
```

---

## Teardown

```bash
# Destroy all AWS resources
just tf-destroy
```

**Warning**: This deletes everything including S3 files!

---

## Troubleshooting

### "No build artifact found"
Wait for GitHub Actions to finish or run `just download-build` to check.

### "SSH connection refused"
Instance is still setting up. Wait 5 minutes after `terraform apply`.

### "Zoo token not found"
Ensure `~/.config/zoo/hosts.toml` exists:
```toml
["https://api.zoo.dev/"]
token = "dev-xxx-your-token"
user = "your@email.com"
```

### "Service won't start"
SSH in and check logs:
```bash
just ssh
journalctl -u platerator -n 50
```

### Build failed on GitHub Actions
Check Actions tab for error details.

---

## Cost Breakdown

| Service | Monthly Cost |
|---------|--------------|
| Lightsail nano_3_0 | **$3.50** |
| S3 (50GB + requests) | ~$1-2 |
| DynamoDB (on-demand) | ~$0.50 |
| Secrets Manager | $0.40 |
| CloudWatch Logs | ~$0.50 |
| **Total** | **~$6/month** |

With AWS free tier (first year): **~$3-4/month**

---

## What Gets Created

**AWS Resources (13 total)**:
- Lightsail instance ($3.50/mo) with static IP
- S3 bucket with lifecycle policies
- DynamoDB cache table
- Secrets Manager for zoo token
- CloudWatch logs + dashboard

**On the Instance**:
- AWS CLI (for accessing Secrets Manager)
- zoo CLI (system-wide in /usr/local/bin)
- Caddy web server (automatic HTTP)
- systemd service (auto-restart)

**SSH Access**:
- Your SSH public key is automatically configured via Terraform
- Connect as `ubuntu` user (not root)

---

## Next Steps

- [ ] Custom domain? Point DNS to Lightsail IP, update Caddy config for HTTPS
- [ ] Implement S3/DynamoDB caching (see DEPLOYMENT.md)
- [ ] Set up monitoring alerts (optional)
- [ ] Configure auto-scaling (upgrade to larger Lightsail bundle)

See `DEPLOYMENT.md` for detailed documentation and AWS integration code examples.
