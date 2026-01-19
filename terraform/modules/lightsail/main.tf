resource "aws_lightsail_key_pair" "main" {
  name       = "${var.instance_name}-key"
  public_key = var.ssh_public_key
}

resource "aws_lightsail_instance" "main" {
  name              = var.instance_name
  availability_zone = var.availability_zone
  blueprint_id      = var.blueprint_id
  bundle_id         = var.bundle_id
  key_pair_name     = aws_lightsail_key_pair.main.name

  user_data = templatefile("${path.module}/user_data.sh", {
    s3_bucket_name = var.s3_bucket_name
    dynamodb_table = var.dynamodb_table
    aws_region     = var.aws_region
  })

  tags = {
    Name        = var.instance_name
    Environment = var.environment
  }
}

resource "aws_lightsail_instance_public_ports" "main" {
  instance_name = aws_lightsail_instance.main.name

  port_info {
    protocol  = "tcp"
    from_port = 80
    to_port   = 80
  }

  port_info {
    protocol  = "tcp"
    from_port = 443
    to_port   = 443
  }

  port_info {
    protocol  = "tcp"
    from_port = 22
    to_port   = 22
  }
}

resource "aws_lightsail_static_ip" "main" {
  name = "${var.instance_name}-ip"
}

resource "aws_lightsail_static_ip_attachment" "main" {
  static_ip_name = aws_lightsail_static_ip.main.name
  instance_name  = aws_lightsail_instance.main.name
}
