output "instance_id" {
  description = "ID of the Lightsail instance"
  value       = aws_lightsail_instance.main.id
}

output "instance_name" {
  description = "Name of the Lightsail instance"
  value       = aws_lightsail_instance.main.name
}

output "public_ip" {
  description = "Public IP address of the instance"
  value       = aws_lightsail_static_ip.main.ip_address
}

output "availability_zone" {
  description = "Availability zone of the instance"
  value       = aws_lightsail_instance.main.availability_zone
}

output "ssh_command" {
  description = "SSH command to connect to the instance"
  value       = "ssh root@${aws_lightsail_static_ip.main.ip_address}"
}
