resource "aws_cloudwatch_log_group" "app_runner" {
  name              = var.log_group_name
  retention_in_days = var.retention_days

  tags = {
    Name        = var.log_group_name
    Environment = var.environment
  }
}

resource "aws_cloudwatch_dashboard" "main" {
  dashboard_name = "${var.project_name}-dashboard"

  dashboard_body = jsonencode({
    widgets = [
      {
        type = "metric"
        properties = {
          metrics = [
            ["AWS/AppRunner", "RequestCount", { stat = "Sum", label = "Total Requests" }],
            [".", "2xxStatusCount", { stat = "Sum", label = "2xx Responses" }],
            [".", "4xxStatusCount", { stat = "Sum", label = "4xx Errors" }],
            [".", "5xxStatusCount", { stat = "Sum", label = "5xx Errors" }]
          ]
          period = 300
          stat   = "Average"
          region = var.region
          title  = "Request Metrics"
        }
      },
      {
        type = "metric"
        properties = {
          metrics = [
            ["AWS/AppRunner", "ActiveInstances", { stat = "Average", label = "Active Instances" }]
          ]
          period = 300
          stat   = "Average"
          region = var.region
          title  = "Active Instances"
        }
      },
      {
        type = "metric"
        properties = {
          metrics = [
            ["AWS/DynamoDB", "ConsumedReadCapacityUnits", { stat = "Sum", label = "Read Capacity" }],
            [".", "ConsumedWriteCapacityUnits", { stat = "Sum", label = "Write Capacity" }]
          ]
          period = 300
          stat   = "Sum"
          region = var.region
          title  = "DynamoDB Usage"
        }
      },
      {
        type = "metric"
        properties = {
          metrics = [
            ["AWS/S3", "NumberOfObjects", { stat = "Average", label = "Object Count" }],
            [".", "BucketSizeBytes", { stat = "Average", label = "Bucket Size (Bytes)" }]
          ]
          period = 86400
          stat   = "Average"
          region = var.region
          title  = "S3 Storage"
        }
      }
    ]
  })
}
