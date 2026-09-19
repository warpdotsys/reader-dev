package cn.medwarp.reader.license;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpServer;

import java.io.IOException;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyFactory;
import java.security.KeyPair;
import java.security.KeyPairGenerator;
import java.security.PrivateKey;
import java.security.PublicKey;
import java.security.Signature;
import java.security.interfaces.RSAPrivateKey;
import java.security.spec.PKCS8EncodedKeySpec;
import java.security.spec.X509EncodedKeySpec;
import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.Statement;
import java.time.Instant;
import java.util.Base64;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;

/** Standalone service with a modern v1 API and a bytecode-verified reader3 migration API. */
public final class LicenseServer {
    private static final ObjectMapper JSON = new ObjectMapper();
    private final Config config;
    private final Store store;
    private final KeyPair keys;
    private final PrivateKey legacyRsaKey;
    private final PublicKey legacyRsaPublicKey;

    LicenseServer(Config config) throws Exception {
        this.config = config;
        this.store = new Store(config.dataDir());
        this.keys = loadOrCreateKeys(config.dataDir());
        KeyPair legacyPair = loadLegacyRsa(config);
        this.legacyRsaKey = legacyPair == null ? null : legacyPair.getPrivate();
        this.legacyRsaPublicKey = legacyPair == null ? null : legacyPair.getPublic();
    }

    public static void main(String[] args) throws Exception {
        Config config = Config.fromEnvironment();
        LicenseServer application = new LicenseServer(config);
        HttpServer server = application.createHttpServer();
        server.start();
        System.out.printf("Reader license v1 listening on %s:%d, issuer=%s%n", config.bind(), config.port(), config.issuer());
    }

    HttpServer createHttpServer() throws IOException {
        HttpServer server = HttpServer.create(new InetSocketAddress(config.bind(), config.port()), 32);
        server.createContext("/", this::handle);
        server.setExecutor(java.util.concurrent.Executors.newCachedThreadPool());
        return server;
    }

    private void handle(HttpExchange exchange) throws IOException {
        String path = exchange.getRequestURI().getPath();
        try {
            applyCors(exchange);
            if ("OPTIONS".equals(exchange.getRequestMethod())) { exchange.sendResponseHeaders(204, -1); exchange.close(); return; }
            if ("GET".equals(exchange.getRequestMethod()) && "/healthz".equals(path)) {
                send(exchange, 200, Map.of("status", "ok", "service", "reader-license", "version", "v1")); return;
            }
            if ("GET".equals(exchange.getRequestMethod()) && "/.well-known/reader-license/v1/public-key".equals(path)) {
                send(exchange, 200, Map.of("algorithm", "RSASSA-PKCS1-v1_5-SHA-256", "format", "X.509-SPKI-base64", "publicKey", Base64.getEncoder().encodeToString(keys.getPublic().getEncoded()), "issuer", config.issuer())); return;
            }
            if ("GET".equals(exchange.getRequestMethod()) && ("/.well-known/reader-license/legacy-rsa-public-key".equals(path) || "/.well-known/reader-license/legacy-public-key".equals(path))) {
                if (legacyRsaPublicKey == null) throw new ApiError(404, "not_enabled", "旧协议迁移未启用");
                send(exchange, 200, Map.of("algorithm", "RSA-2048", "format", "X.509-SPKI-base64", "publicKey", Base64.getEncoder().encodeToString(legacyRsaPublicKey.getEncoded()))); return;
            }
            if ("POST".equals(exchange.getRequestMethod()) && "/v1/legacy/licenses".equals(path)) { requireAdmin(exchange); issueLegacy(exchange); return; }
            if ("POST".equals(exchange.getRequestMethod()) && path.matches("/v1/legacy/licenses/[^/]+/[^/]+/revoke")) { requireAdmin(exchange); revokeLegacy(exchange, path); return; }
            if (path.startsWith("/v1/licenses")) { handleLicenses(exchange, path); return; }
            if ("POST".equals(exchange.getRequestMethod()) && "/reader3/activateLicense".equals(path)) { legacyActivate(exchange); return; }
            if (("GET".equals(exchange.getRequestMethod()) || "POST".equals(exchange.getRequestMethod())) && "/reader3/isLicenseValid".equals(path)) { legacyValidate(exchange); return; }
            if ("POST".equals(exchange.getRequestMethod()) && ("/reader3/sendCodeToEmail".equals(path) || "/reader3/supplyLicense".equals(path))) { legacyNotImplemented(exchange); return; }
            if ("POST".equals(exchange.getRequestMethod()) && "/v1/activations".equals(path)) { activate(exchange); return; }
            if ("POST".equals(exchange.getRequestMethod()) && "/v1/validations".equals(path)) { validate(exchange); return; }
            if ("GET".equals(exchange.getRequestMethod()) && "/v1/audit".equals(path)) { requireAdmin(exchange); send(exchange, 200, Map.of("items", store.audit())); return; }
            send(exchange, 404, Map.of("error", "not_found"));
        } catch (ApiError e) { send(exchange, e.status, Map.of("error", e.code, "message", e.getMessage())); }
        catch (Exception e) { e.printStackTrace(System.err); send(exchange, 500, Map.of("error", "internal_error")); }
    }

    private void handleLicenses(HttpExchange x, String path) throws Exception {
        requireAdmin(x);
        String[] parts = path.split("/");
        if ("POST".equals(x.getRequestMethod()) && "/v1/licenses".equals(path)) { issue(x); return; }
        if (parts.length == 4 && "GET".equals(x.getRequestMethod())) { send(x, 200, store.license(parts[3])); return; }
        if (parts.length == 5 && "revoke".equals(parts[4]) && "POST".equals(x.getRequestMethod())) { store.revoke(parts[3], actor(x), clientIp(x)); send(x, 200, Map.of("id", parts[3], "revoked", true)); return; }
        throw new ApiError(404, "not_found", "未知管理路径");
    }

    private void issue(HttpExchange x) throws Exception {
        Map<String, Object> request = body(x);
        String subject = required(request, "subject");
        String host = required(request, "host");
        long expiresAt = number(request, "expiresAt", 0);
        int users = (int) number(request, "userMaxLimit", 1);
        int instances = (int) number(request, "instances", 1);
        if (users < 1 || instances < 1 || host.length() > 255) throw new ApiError(400, "invalid_request", "许可参数不合法");
        String id = UUID.randomUUID().toString();
        Map<String, Object> claims = new LinkedHashMap<>();
        claims.put("version", 1); claims.put("id", id); claims.put("issuer", config.issuer()); claims.put("subject", subject);
        claims.put("host", host); claims.put("expiresAt", expiresAt); claims.put("userMaxLimit", users); claims.put("instances", instances);
        claims.put("features", request.getOrDefault("features", List.of())); claims.put("issuedAt", Instant.now().toEpochMilli());
        String token = sign(claims);
        store.insert(id, subject, host, expiresAt, users, instances, JSON.writeValueAsString(claims), actor(x), clientIp(x));
        send(x, 201, Map.of("license", claims, "token", token));
    }

    private void issueLegacy(HttpExchange x) throws Exception {
        if (legacyRsaKey == null) throw new ApiError(409, "legacy_not_enabled", "请启用 LICENSE_LEGACY_RSA_ENABLED");
        Map<String,Object> request = body(x);
        String host = required(request, "host");
        String type = required(request, "type");
        String code = request.containsKey("code") ? required(request, "code") : UUID.randomUUID().toString();
        long expiresAt = number(request, "expiredAt", 0);
        long simpleWebExpiresAt = number(request, "simpleWebExpiredAt", 0);
        int users = (int) number(request, "userMaxLimit", 15);
        int instances = (int) number(request, "instances", 1);
        if (users < 1 || instances < 1) throw new ApiError(400, "invalid_request", "许可参数不合法");
        Map<String,Object> license = new LinkedHashMap<>();
        license.put("host", host); license.put("userMaxLimit", users); license.put("expiredAt", expiresAt);
        license.put("openApi", Boolean.TRUE.equals(request.get("openApi"))); license.put("simpleWebExpiredAt", simpleWebExpiresAt);
        license.put("instances", instances); license.put("type", type); license.put("id", UUID.randomUUID().toString()); license.put("code", code);
        license.put("verified", false); license.put("verifyTime", null);
        store.insertLegacy(type, code, license, instances, expiresAt, actor(x), clientIp(x));
        send(x, 201, Map.of("license", license, "content", legacyEncrypt(JSON.writeValueAsString(license), legacyRsaKey), "publicKey", Base64.getEncoder().encodeToString(legacyRsaPublicKey.getEncoded())));
    }
    private void revokeLegacy(HttpExchange x, String path) throws Exception {
        String[] items = path.split("/");
        store.revokeLegacy(items[4], items[5], actor(x), clientIp(x));
        send(x, 200, Map.of("type", items[4], "code", items[5], "revoked", true));
    }

    private void activate(HttpExchange x) throws Exception {
        Map<String, Object> request = body(x); Map<String, Object> claims = verify(required(request, "token"));
        String instance = required(request, "instanceId"); checkActive(claims);
        boolean newlyCreated = store.activate((String) claims.get("id"), instance, clientIp(x));
        send(x, 200, Map.of("valid", true, "activated", newlyCreated, "licenseId", claims.get("id"), "expiresAt", claims.get("expiresAt")));
    }

    private void validate(HttpExchange x) throws Exception {
        Map<String, Object> request = body(x); Map<String, Object> claims = verify(required(request, "token"));
        String instance = required(request, "instanceId"); checkActive(claims);
        if (!store.isActivated((String) claims.get("id"), instance)) throw new ApiError(403, "not_activated", "实例尚未激活");
        store.touch((String) claims.get("id"), instance, clientIp(x));
        send(x, 200, Map.of("valid", true, "licenseId", claims.get("id"), "expiresAt", claims.get("expiresAt"), "serverTime", Instant.now().toEpochMilli()));
    }

    /* Contract reconstructed from bytecode: ReturnData plus RSA(private)-encrypted `result`.
       It cannot validate old upstream licenses without the retired private key / matching client public key. */
    private void legacyActivate(HttpExchange x) throws Exception {
        try {
            if (legacyRsaKey == null) { legacySend(x, false, "旧协议签名密钥未配置", Map.of()); return; }
            Map<String,Object> request = body(x);
            Object rawContent = request.get("content");
            if (!(rawContent instanceof String) || ((String) rawContent).isBlank()) {
                legacySend(x, false, "请输入密钥", Map.of());
                return;
            }
            String content = (String) rawContent;
            Map<String,Object> activated = store.activateLegacy(legacyDecrypt(content), clientIp(x));
            send(x, 200, Map.of("isSuccess", true, "errorMsg", "", "data", Map.of("result", legacyEncrypt(JSON.writeValueAsString(activated), legacyRsaKey))));
        } catch (Exception e) { legacySend(x, false, legacyMessage(e), Map.of()); }
    }
    private void legacyValidate(HttpExchange x) throws Exception {
        try {
            if (legacyRsaKey == null) { legacySend(x, false, "旧协议签名密钥未配置", Map.of()); return; }
            String id;
            if ("POST".equals(x.getRequestMethod())) id = required(body(x), "id");
            else { String query=x.getRequestURI().getRawQuery(); id=query==null?"":java.net.URLDecoder.decode(query.replaceFirst("^id=", ""), StandardCharsets.UTF_8); }
            legacySend(x, true, "", store.legacyStatus(id, clientIp(x)));
        } catch (Exception e) { legacySend(x, false, legacyMessage(e), Map.of()); }
    }
    private void legacyNotImplemented(HttpExchange x) throws IOException {
        Map<String,Object> payload = new LinkedHashMap<>();
        payload.put("isSuccess", false);
        payload.put("errorMsg", "此部署尚未配置试用邮件服务");
        payload.put("data", null);
        send(x, 200, payload);
    }
    private static String legacyMessage(Exception e) {
        if (!(e instanceof ApiError)) return "密钥错误";
        ApiError api = (ApiError) e;
        if ("instance_limit_reached".equals(api.code)) return "密钥已超过最大使用次数";
        if ("already_used".equals(api.code)) return "许可证已被使用";
        if ("unknown_license".equals(api.code) || "malformed_license".equals(api.code)) return "密钥错误";
        return api.getMessage();
    }
    private void legacySend(HttpExchange x, boolean success, String error, Map<String,Object> result) throws Exception {
        Map<String,Object> payload = new LinkedHashMap<>();
        payload.put("isSuccess", success);
        payload.put("errorMsg", error);
        if (success && legacyRsaKey != null) {
            payload.put("data", Map.of("result", legacyEncrypt(JSON.writeValueAsString(result), legacyRsaKey)));
        } else {
            payload.put("data", null);
        }
        send(x, 200, payload);
    }
    private static String legacyEncrypt(String input, PrivateKey key) throws Exception {
        int bytes=((RSAPrivateKey)key).getModulus().bitLength()/8, block=bytes-11, offset=0;
        javax.crypto.Cipher cipher=javax.crypto.Cipher.getInstance("RSA"); cipher.init(javax.crypto.Cipher.ENCRYPT_MODE,key);
        java.io.ByteArrayOutputStream out=new java.io.ByteArrayOutputStream(); byte[] raw=input.getBytes(StandardCharsets.UTF_8);
        while(offset<raw.length){int size=Math.min(block,raw.length-offset);out.write(cipher.doFinal(raw,offset,size));offset+=size;}
        return Base64.getEncoder().encodeToString(out.toByteArray());
    }
    private Map<String,Object> legacyDecrypt(String content) throws Exception {
        byte[] encrypted = Base64.getDecoder().decode(content);
        int block = ((java.security.interfaces.RSAPublicKey) legacyRsaPublicKey).getModulus().bitLength() / 8;
        javax.crypto.Cipher cipher = javax.crypto.Cipher.getInstance("RSA"); cipher.init(javax.crypto.Cipher.DECRYPT_MODE, legacyRsaPublicKey);
        java.io.ByteArrayOutputStream out = new java.io.ByteArrayOutputStream();
        for (int offset = 0; offset < encrypted.length; offset += block) out.write(cipher.doFinal(encrypted, offset, Math.min(block, encrypted.length - offset)));
        return JSON.readValue(out.toByteArray(), new TypeReference<Map<String,Object>>() {});
    }

    private void checkActive(Map<String, Object> c) throws Exception {
        if (!config.issuer().equals(c.get("issuer"))) throw new ApiError(401, "wrong_issuer", "签发者不匹配");
        long expiry = ((Number) c.get("expiresAt")).longValue();
        if (expiry > 0 && expiry < Instant.now().toEpochMilli()) throw new ApiError(403, "expired", "许可证已过期");
        if (store.revoked((String) c.get("id"))) throw new ApiError(403, "revoked", "许可证已吊销");
    }

    private String sign(Map<String, Object> claims) throws Exception {
        String payload = b64(JSON.writeValueAsBytes(claims));
        Signature signature = Signature.getInstance("SHA256withRSA"); signature.initSign(keys.getPrivate()); signature.update(payload.getBytes(StandardCharsets.US_ASCII));
        return "rlic1." + payload + "." + b64(signature.sign());
    }

    private Map<String, Object> verify(String token) throws Exception {
        String[] fields = token.split("\\.");
        if (fields.length != 3 || !"rlic1".equals(fields[0])) throw new ApiError(401, "malformed_token", "token 格式错误");
        Signature signature = Signature.getInstance("SHA256withRSA"); signature.initVerify(keys.getPublic()); signature.update(fields[1].getBytes(StandardCharsets.US_ASCII));
        if (!signature.verify(unb64(fields[2]))) throw new ApiError(401, "invalid_signature", "token 签名无效");
        return JSON.readValue(unb64(fields[1]), new TypeReference<>() {});
    }

    private void requireAdmin(HttpExchange x) throws ApiError {
        String got = x.getRequestHeaders().getFirst("Authorization"); String expected = "Bearer " + config.adminToken();
        if (got == null || !java.security.MessageDigest.isEqual(got.getBytes(StandardCharsets.UTF_8), expected.getBytes(StandardCharsets.UTF_8))) throw new ApiError(401, "unauthorized", "需要管理凭据");
    }
    private static Map<String,Object> body(HttpExchange x) throws IOException { return JSON.readValue(x.getRequestBody(), new TypeReference<>() {}); }
    private static String required(Map<String,Object> map, String name) throws ApiError { Object value = map.get(name); if (!(value instanceof String) || ((String) value).trim().isEmpty()) throw new ApiError(400,"invalid_request","缺少 "+name); return (String) value; }
    private static long number(Map<String,Object> map, String name, long fallback) throws ApiError { Object value=map.get(name); if(value==null)return fallback; if(value instanceof Number)return ((Number)value).longValue(); throw new ApiError(400,"invalid_request",name+" 必须是数字"); }
    private static String b64(byte[] raw) { return Base64.getUrlEncoder().withoutPadding().encodeToString(raw); }
    private static byte[] unb64(String value) { return Base64.getUrlDecoder().decode(value); }
    private static String clientIp(HttpExchange x) { String forwarded=x.getRequestHeaders().getFirst("X-Forwarded-For"); return forwarded==null ? x.getRemoteAddress().getAddress().getHostAddress() : forwarded.split(",",2)[0].trim(); }
    private static String actor(HttpExchange x) { return "admin@" + clientIp(x); }
    private void applyCors(HttpExchange x) {
        String origin = x.getRequestHeaders().getFirst("Origin");
        if (origin == null || !config.corsAllows(origin)) return;
        x.getResponseHeaders().set("Access-Control-Allow-Origin", origin);
        x.getResponseHeaders().set("Access-Control-Allow-Credentials", "true");
        x.getResponseHeaders().set("Access-Control-Allow-Headers", "Content-Type, Authorization");
        x.getResponseHeaders().set("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
        x.getResponseHeaders().set("Vary", "Origin");
    }
    private static void send(HttpExchange x, int status, Object payload) throws IOException { byte[] bytes=JSON.writeValueAsBytes(payload); x.getResponseHeaders().set("Content-Type","application/json; charset=utf-8"); x.getResponseHeaders().set("Cache-Control","no-store"); x.sendResponseHeaders(status,bytes.length); x.getResponseBody().write(bytes); x.close(); }

    static KeyPair loadOrCreateKeys(Path dir) throws Exception { Files.createDirectories(dir); Path privateFile=dir.resolve("signing-key.pkcs8"); Path publicFile=dir.resolve("signing-key.pub"); KeyFactory factory=KeyFactory.getInstance("RSA"); if(Files.exists(privateFile)&&Files.exists(publicFile)) return new KeyPair(factory.generatePublic(new X509EncodedKeySpec(Files.readAllBytes(publicFile))),factory.generatePrivate(new PKCS8EncodedKeySpec(Files.readAllBytes(privateFile)))); KeyPairGenerator gen=KeyPairGenerator.getInstance("RSA"); gen.initialize(2048); KeyPair pair=gen.generateKeyPair(); Files.write(privateFile,pair.getPrivate().getEncoded()); Files.write(publicFile,pair.getPublic().getEncoded()); return pair; }
    static KeyPair loadLegacyRsa(Config config) throws Exception { if (!config.legacyRsaEnabled()) return null; Path privateFile=config.legacyRsaFile()==null ? config.dataDir().resolve("legacy-rsa-private.pkcs8") : Path.of(config.legacyRsaFile()); Path publicFile=config.legacyRsaPublicFile()==null ? config.dataDir().resolve("legacy-rsa-public.spki") : Path.of(config.legacyRsaPublicFile()); KeyFactory f=KeyFactory.getInstance("RSA"); if(!Files.exists(privateFile)){KeyPairGenerator g=KeyPairGenerator.getInstance("RSA");g.initialize(2048);KeyPair pair=g.generateKeyPair();Files.write(privateFile,pair.getPrivate().getEncoded());Files.write(publicFile,pair.getPublic().getEncoded());return pair;} if(!Files.exists(publicFile)) throw new IllegalStateException("外部 RSA 私钥必须同时配置对应公钥文件"); return new KeyPair(f.generatePublic(new X509EncodedKeySpec(readPemOrDer(publicFile))),f.generatePrivate(new PKCS8EncodedKeySpec(readPemOrDer(privateFile)))); }
    static byte[] readPemOrDer(Path file) throws IOException {
        byte[] raw = Files.readAllBytes(file);
        int prefixLength = Math.min(raw.length, 64);
        String prefix = new String(raw, 0, prefixLength, StandardCharsets.US_ASCII);
        if (!prefix.startsWith("-----BEGIN")) return raw;
        String text = new String(raw, StandardCharsets.US_ASCII);
        return Base64.getMimeDecoder().decode(text.replaceAll("-----[^-]+-----", "").replaceAll("\\s", ""));
    }

    static final class Config {
        private final String bind, adminToken, issuer, legacyRsaFile, legacyRsaPublicFile, corsAllowedOrigins; private final int port; private final Path dataDir; private final boolean legacyRsaEnabled;
        Config(String bind,int port,Path dataDir,String adminToken,String issuer,boolean legacyRsaEnabled,String legacyRsaFile,String legacyRsaPublicFile,String corsAllowedOrigins){this.bind=bind;this.port=port;this.dataDir=dataDir;this.adminToken=adminToken;this.issuer=issuer;this.legacyRsaEnabled=legacyRsaEnabled;this.legacyRsaFile=legacyRsaFile;this.legacyRsaPublicFile=legacyRsaPublicFile;this.corsAllowedOrigins=corsAllowedOrigins;}
        String bind(){return bind;} int port(){return port;} Path dataDir(){return dataDir;} String adminToken(){return adminToken;} String issuer(){return issuer;} boolean legacyRsaEnabled(){return legacyRsaEnabled;} String legacyRsaFile(){return legacyRsaFile;} String legacyRsaPublicFile(){return legacyRsaPublicFile;}
        boolean corsAllows(String origin){for(String allowed:corsAllowedOrigins.split(",")){if(allowed.trim().equals(origin))return true;}return false;}
        static Config fromEnvironment() { Map<String,String> env=System.getenv(); String token=env.getOrDefault("LICENSE_ADMIN_TOKEN", ""); if(token.length()<24) throw new IllegalStateException("LICENSE_ADMIN_TOKEN 必须至少 24 字符"); return new Config(env.getOrDefault("LICENSE_BIND","127.0.0.1"),Integer.parseInt(env.getOrDefault("LICENSE_PORT","8080")),Path.of(env.getOrDefault("LICENSE_DATA_DIR","data")),token,env.getOrDefault("LICENSE_ISSUER","https://license.medwarp.cn"),Boolean.parseBoolean(env.getOrDefault("LICENSE_LEGACY_RSA_ENABLED","false")),env.get("LICENSE_LEGACY_RSA_PRIVATE_KEY_FILE"),env.get("LICENSE_LEGACY_RSA_PUBLIC_KEY_FILE"),env.getOrDefault("LICENSE_CORS_ALLOWED_ORIGINS","")); }
    }
    static final class ApiError extends Exception { final int status; final String code; ApiError(int status,String code,String message){super(message);this.status=status;this.code=code;} }
}
