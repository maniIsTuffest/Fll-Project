"""Email utility functions for sending notifications."""

import logging
import smtplib
import os
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
from typing import Optional

logger = logging.getLogger(__name__)

# Email configuration - can be set via environment variables
SMTP_SERVER = os.getenv("SMTP_SERVER", "smtp.gmail.com")
SMTP_PORT = int(os.getenv("SMTP_PORT", "587"))
SMTP_USERNAME = os.getenv("SMTP_USERNAME", "")
SMTP_PASSWORD = os.getenv("SMTP_PASSWORD", "")
FROM_EMAIL = os.getenv("FROM_EMAIL", "noreply@artifactgallery.com")


def send_email(to_email: str, subject: str, body: str, html_body: Optional[str] = None) -> bool:
    """
    Send an email notification.
    
    Args:
        to_email: Recipient email address
        subject: Email subject line
        body: Plain text email body
        html_body: Optional HTML version of email body
        
    Returns:
        True if email sent successfully, False otherwise
    """
    if not SMTP_USERNAME or not SMTP_PASSWORD:
        logger.warning("SMTP credentials not configured. Email not sent.")
        logger.info(f"Would have sent email to {to_email}: {subject}")
        logger.info(f"Body: {body}")
        return False
    
    try:
        msg = MIMEMultipart("alternative")
        msg["Subject"] = subject
        msg["From"] = FROM_EMAIL
        msg["To"] = to_email
        
        # Attach plain text version
        msg.attach(MIMEText(body, "plain"))
        
        # Attach HTML version if provided
        if html_body:
            msg.attach(MIMEText(html_body, "html"))
        
        # Connect and send
        with smtplib.SMTP(SMTP_SERVER, SMTP_PORT) as server:
            server.starttls()
            server.login(SMTP_USERNAME, SMTP_PASSWORD)
            server.sendmail(FROM_EMAIL, to_email, msg.as_string())
        
        logger.info(f"Email sent successfully to {to_email}")
        return True
        
    except Exception as e:
        logger.error(f"Failed to send email to {to_email}: {str(e)}")
        return False


def send_verification_notification(
    to_email: str,
    artifact_name: str,
    status: str,
    reason: str,
    verified_by: str
) -> bool:
    """
    Send a notification about artifact verification status change.
    
    Args:
        to_email: Uploader's email address
        artifact_name: Name of the artifact
        status: New verification status ('verified' or 'rejected')
        reason: Reason provided by the reviewer
        verified_by: Username of the person who verified/rejected
        
    Returns:
        True if email sent successfully, False otherwise
    """
    if status.lower() == "verified":
        status_text = "APPROVED ✅"
        status_color = "#28a745"
    else:
        status_text = "REJECTED ❌"
        status_color = "#dc3545"
    
    subject = f"Artifact '{artifact_name}' has been {status_text}"
    
    body = f"""
Hello,

Your artifact submission has been reviewed.

Artifact: {artifact_name}
Status: {status_text}
Reviewed by: {verified_by}

Reviewer's Comments:
{reason}

Thank you for your contribution to the Artifact Gallery.

Best regards,
The ArtiQuest Team
"""

    html_body = f"""
<!DOCTYPE html>
<html>
<head>
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; text-align: center; border-radius: 8px 8px 0 0; }}
        .content {{ background: #f9f9f9; padding: 20px; border: 1px solid #ddd; }}
        .status {{ display: inline-block; padding: 8px 16px; border-radius: 4px; color: white; font-weight: bold; background-color: {status_color}; }}
        .reason-box {{ background: white; border-left: 4px solid {status_color}; padding: 15px; margin: 15px 0; }}
        .footer {{ text-align: center; padding: 20px; color: #666; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🏺 Artifact Gallery</h1>
            <p>Verification Notification</p>
        </div>
        <div class="content">
            <h2>Hello,</h2>
            <p>Your artifact submission has been reviewed.</p>
            
            <p><strong>Artifact:</strong> {artifact_name}</p>
            <p><strong>Status:</strong> <span class="status">{status_text}</span></p>
            <p><strong>Reviewed by:</strong> {verified_by}</p>
            
            <div class="reason-box">
                <strong>Reviewer's Comments:</strong>
                <p>{reason}</p>
            </div>
            
            <p>Thank you for your contribution to the Artifact Gallery.</p>
        </div>
        <div class="footer">
            <p>Best regards,<br>The ArtiQuest Team</p>
        </div>
    </div>
</body>
</html>
"""
    
    return send_email(to_email, subject, body, html_body)

