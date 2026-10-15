package cn.medwarp.reader.license;

import jakarta.mail.Authenticator;
import jakarta.mail.Message;
import jakarta.mail.PasswordAuthentication;
import jakarta.mail.Session;
import jakarta.mail.Transport;
import jakarta.mail.internet.InternetAddress;
import jakarta.mail.internet.MimeMessage;

import java.util.Properties;

/**
 * SMTP delivery through the maintained Angus Jakarta Mail implementation.
 * TLS certificate and hostname validation stay enabled whenever this is not
 * the explicitly requested loopback-only plaintext test mode.
 */
final class SmtpMailSender {
  private final LicenseServer.SmtpConfig config;

  SmtpMailSender(LicenseServer.SmtpConfig config) { this.config = config; }

  boolean configured() { return config != null && config.enabled(); }

  void sendTrialCode(String recipient, String code) throws Exception {
    if (!configured()) throw new IllegalStateException("试用邮件服务未配置");
    if (!LicenseServer.isSafeMailbox(recipient) || !LicenseServer.isSafeMailbox(config.from())) {
      throw new IllegalStateException("SMTP 发件人或收件人格式不合法");
    }
    Properties properties = new Properties();
    properties.setProperty("mail.smtp.host", config.host());
    properties.setProperty("mail.smtp.port", String.valueOf(config.port()));
    properties.setProperty("mail.smtp.connectiontimeout", "5000");
    properties.setProperty("mail.smtp.timeout", "10000");
    properties.setProperty("mail.smtp.writetimeout", "10000");
    properties.setProperty("mail.smtp.quitwait", "false");
    if (config.hasCredentials()) properties.setProperty("mail.smtp.auth", "true");
    if (config.startTls()) {
      properties.setProperty("mail.smtp.starttls.enable", "true");
      properties.setProperty("mail.smtp.starttls.required", "true");
    }
    if (config.implicitTls()) properties.setProperty("mail.smtp.ssl.enable", "true");
    if (!config.plaintextLoopback()) {
      properties.setProperty("mail.smtp.ssl.checkserveridentity", "true");
      properties.setProperty("mail.smtp.ssl.protocols", "TLSv1.2 TLSv1.3");
    }

    Authenticator authenticator = config.hasCredentials() ? new Authenticator() {
      @Override protected PasswordAuthentication getPasswordAuthentication() {
        return new PasswordAuthentication(config.username(), config.password());
      }
    } : null;
    Session session = Session.getInstance(properties, authenticator);
    MimeMessage message = new MimeMessage(session);
    message.setFrom(new InternetAddress(config.from(), false));
    message.setRecipients(Message.RecipientType.TO, InternetAddress.parse(recipient, false));
    message.setSubject("Reader Pro trial verification code", "UTF-8");
    message.setText("Your Reader Pro trial verification code is: " + code
        + "\n\nIt expires in 15 minutes. If you did not request it, ignore this email.\n", "UTF-8");
    Transport transport = session.getTransport("smtp");
    try {
      if (config.hasCredentials()) transport.connect(config.host(), config.port(), config.username(), config.password());
      else transport.connect();
      transport.sendMessage(message, message.getAllRecipients());
    } finally {
      transport.close();
    }
  }
}
