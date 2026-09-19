package cn.medwarp.reader.license;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.sun.net.httpserver.HttpServer;
import org.junit.jupiter.api.Test;
import javax.crypto.Cipher;
import java.net.URI;
import java.net.http.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.security.*;
import java.security.interfaces.RSAPublicKey;
import java.security.spec.X509EncodedKeySpec;
import java.util.*;
import static org.junit.jupiter.api.Assertions.*;

class LicenseServerTest {
  static final ObjectMapper JSON=new ObjectMapper();
  @Test void generatedLegacyDerKeysSurviveRestart() throws Exception {
    java.nio.file.Path dir=Files.createTempDirectory("license-key-restart");
    LicenseServer.Config config=new LicenseServer.Config("127.0.0.1",0,dir,"012345678901234567890123","https://license.medwarp.cn",true,null,null,"");
    new LicenseServer(config);
    byte[] publicKey=Files.readAllBytes(dir.resolve("legacy-rsa-public.spki"));
    byte[] privateKey=Files.readAllBytes(dir.resolve("legacy-rsa-private.pkcs8"));
    new LicenseServer(config);
    assertArrayEquals(publicKey,Files.readAllBytes(dir.resolve("legacy-rsa-public.spki")));
    assertArrayEquals(privateKey,Files.readAllBytes(dir.resolve("legacy-rsa-private.pkcs8")));
  }
  @Test void legacyIssueActivateThenValidate() throws Exception {
    String token="012345678901234567890123";
    LicenseServer app=new LicenseServer(new LicenseServer.Config("127.0.0.1",0,Files.createTempDirectory("license-test"),token,"https://license.medwarp.cn",true,null,null,"https://reader.example"));
    HttpServer http=app.createHttpServer();http.start();
    try {
      String base="http://127.0.0.1:"+http.getAddress().getPort();
      HttpResponse<String> preflight=HttpClient.newHttpClient().send(HttpRequest.newBuilder(URI.create(base+"/reader3/activateLicense")).method("OPTIONS",HttpRequest.BodyPublishers.noBody()).header("Origin","https://reader.example").build(),HttpResponse.BodyHandlers.ofString());
      assertEquals(204,preflight.statusCode());assertEquals("https://reader.example",preflight.headers().firstValue("Access-Control-Allow-Origin").orElse(""));
      Map<String,Object> issued=post(base+"/v1/legacy/licenses",token,Map.of("host","reader.example","type","pro","code","test-code","instances",1));
      assertTrue(issued.containsKey("content"));
      Map<String,Object> activation=post(base+"/reader3/activateLicense",null,Map.of("content",issued.get("content")));
      assertTrue((Boolean)activation.get("isSuccess"));
      Map<String,Object> duplicate=post(base+"/reader3/activateLicense",null,Map.of("content",issued.get("content")));
      assertFalse((Boolean)duplicate.get("isSuccess"));
      assertEquals("密钥已超过最大使用次数", duplicate.get("errorMsg"));
      assertNull(duplicate.get("data"));
      Map<String,Object> missingContent=post(base+"/reader3/activateLicense",null,Map.of());
      assertFalse((Boolean)missingContent.get("isSuccess"));
      assertEquals("请输入密钥", missingContent.get("errorMsg"));
      assertNull(missingContent.get("data"));
      Map<String,Object> trial=post(base+"/reader3/sendCodeToEmail",null,Map.of("email","nobody@example.com"));
      assertFalse((Boolean)trial.get("isSuccess"));
      assertNull(trial.get("data"));
      Map<String,Object> doc=get(base+"/.well-known/reader-license/legacy-public-key");
      PublicKey key=KeyFactory.getInstance("RSA").generatePublic(new X509EncodedKeySpec(Base64.getDecoder().decode((String)doc.get("publicKey"))));
      Map<String,Object> licence=decrypt((String)((Map<?,?>)activation.get("data")).get("result"),key);
      assertTrue((Boolean)licence.get("verified"));
      String id=(String)licence.get("id");
      Map<String,Object> checked=get(base+"/reader3/isLicenseValid?id="+id);
      assertTrue((Boolean)decrypt((String)((Map<?,?>)checked.get("data")).get("result"),key).get("isValid"));
      Map<String,Object> absent=get(base+"/reader3/isLicenseValid?id=missing");
      assertFalse((Boolean)decrypt((String)((Map<?,?>)absent.get("data")).get("result"),key).get("isValid"));
      post(base+"/v1/legacy/licenses/pro/test-code/revoke",token,Map.of());
      Map<String,Object> revoked=get(base+"/reader3/isLicenseValid?id="+id);
      assertFalse((Boolean)decrypt((String)((Map<?,?>)revoked.get("data")).get("result"),key).get("isValid"));
    } finally {http.stop(0);}
  }
  static Map<String,Object> post(String url,String token,Map<String,Object> body)throws Exception {HttpRequest.Builder b=HttpRequest.newBuilder(URI.create(url)).header("Content-Type","application/json").POST(HttpRequest.BodyPublishers.ofString(JSON.writeValueAsString(body)));if(token!=null)b.header("Authorization","Bearer "+token);HttpResponse<String> r=HttpClient.newHttpClient().send(b.build(),HttpResponse.BodyHandlers.ofString());assertTrue(r.statusCode()==200||r.statusCode()==201);return JSON.readValue(r.body(),new TypeReference<Map<String,Object>>(){});}
  static Map<String,Object> get(String url)throws Exception {HttpResponse<String> r=HttpClient.newHttpClient().send(HttpRequest.newBuilder(URI.create(url)).GET().build(),HttpResponse.BodyHandlers.ofString());assertEquals(200,r.statusCode());return JSON.readValue(r.body(),new TypeReference<Map<String,Object>>(){});}
  static Map<String,Object> decrypt(String content,PublicKey key)throws Exception {byte[] data=Base64.getDecoder().decode(content);int block=((RSAPublicKey)key).getModulus().bitLength()/8;Cipher c=Cipher.getInstance("RSA");c.init(Cipher.DECRYPT_MODE,key);java.io.ByteArrayOutputStream out=new java.io.ByteArrayOutputStream();for(int i=0;i<data.length;i+=block)out.write(c.doFinal(data,i,Math.min(block,data.length-i)));return JSON.readValue(new String(out.toByteArray(),StandardCharsets.UTF_8),new TypeReference<Map<String,Object>>(){});}
}
