package cn.medwarp.reader.license;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.sun.net.httpserver.HttpServer;
import org.junit.jupiter.api.Test;
import javax.crypto.Cipher;
import java.net.URI;
import java.net.http.*;
import java.net.ServerSocket;
import java.net.Socket;
import java.net.InetAddress;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.*;
import java.security.interfaces.RSAPublicKey;
import java.security.spec.X509EncodedKeySpec;
import java.time.Clock;
import java.time.Instant;
import java.time.ZoneId;
import java.time.ZoneOffset;
import java.util.*;
import java.util.concurrent.*;
import java.util.regex.*;
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
  @Test void trialSmtpFlowIssuesLegacyKeyOnceAndSurvivesRestart() throws Exception {
    Path dir=Files.createTempDirectory("license-trial-test");
    try (FakeSmtp smtp=new FakeSmtp()) {
      String token="012345678901234567890123";
      LicenseServer.Config config=new LicenseServer.Config("127.0.0.1",0,dir,token,"https://license.medwarp.cn",true,null,null,"",LicenseServer.SmtpConfig.localPlaintextForTest(smtp.port()));
      LicenseServer app=new LicenseServer(config); HttpServer http=app.createHttpServer();http.start();
      try {
        String base="http://127.0.0.1:"+http.getAddress().getPort(); String email="trial@example.test";
        Map<String,Object> sent=post(base+"/reader3/sendCodeToEmail",null,Map.of("email",email));
        assertTrue((Boolean)sent.get("isSuccess")); assertEquals("",sent.get("data"));
        String mail=smtp.take(); Matcher matcher=Pattern.compile("\\b(\\d{8})\\b").matcher(mail); assertTrue(matcher.find()); String code=matcher.group(1);
        Map<String,Object> repeatedSend=post(base+"/reader3/sendCodeToEmail",null,Map.of("email",email));
        assertFalse((Boolean)repeatedSend.get("isSuccess")); assertNull(repeatedSend.get("data"));
        String wrongCode=code.equals("00000000")?"00000001":"00000000";
        Map<String,Object> bad=post(base+"/reader3/supplyLicense",null,Map.of("email",email,"code",wrongCode));
        assertFalse((Boolean)bad.get("isSuccess")); assertNull(bad.get("data"));
        Map<String,Object> claimed=post(base+"/reader3/supplyLicense",null,Map.of("email",email,"code",code));
        assertTrue((Boolean)claimed.get("isSuccess"));
        String encrypted=(String)((Map<?,?>)claimed.get("data")).get("key");
        PublicKey key=KeyFactory.getInstance("RSA").generatePublic(new X509EncodedKeySpec(Files.readAllBytes(dir.resolve("legacy-rsa-public.spki"))));
        Map<String,Object> trial=decrypt(encrypted,key); assertEquals("trial",trial.get("type")); assertEquals(email,trial.get("code")); assertEquals("*",trial.get("host")); assertEquals(15,((Number)trial.get("userMaxLimit")).intValue());
        Map<String,Object> replay=post(base+"/reader3/supplyLicense",null,Map.of("email",email,"code",code)); assertFalse((Boolean)replay.get("isSuccess"));
      } finally {http.stop(0);}
      LicenseServer restarted=new LicenseServer(config); HttpServer again=restarted.createHttpServer();again.start();
      try {String base="http://127.0.0.1:"+again.getAddress().getPort();Map<String,Object> claimAgain=post(base+"/reader3/sendCodeToEmail",null,Map.of("email","trial@example.test"));assertFalse((Boolean)claimAgain.get("isSuccess"));} finally {again.stop(0);}
    }
  }
  @Test void trialStoreBoundsEmailAndIpRequests() throws Exception {
    Store store=new Store(Files.createTempDirectory("license-trial-limits"));
    byte[] hash=new byte[32];
    Arrays.fill(hash,(byte)7);
    assertTrue(store.reserveTrial("a@example.test",claim("a@example.test"),"127.0.0.1",hash).ok());
    assertFalse(store.reserveTrial("a@example.test",claim("a@example.test"),"127.0.0.1",hash).ok());
    for (int i=0;i<9;i++) {
      String email="u"+i+"@example.test";
      assertTrue(store.reserveTrial(email,claim(email),"127.0.0.1",hash).ok());
    }
    assertFalse(store.reserveTrial("overflow@example.test",claim("overflow@example.test"),"127.0.0.1",hash).ok());
  }
  @Test void concurrentTrialReservationOnlyAllowsOneLiveCode() throws Exception {
    Store store=new Store(Files.createTempDirectory("license-trial-concurrent"));
    ExecutorService pool=Executors.newFixedThreadPool(4);
    CountDownLatch start=new CountDownLatch(1);
    List<Future<Boolean>> work=new ArrayList<>();
    for (int i=0;i<4;i++) {
      final int value=i;
      work.add(pool.submit(()-> {
        start.await();
        byte[] hash=new byte[32];
        Arrays.fill(hash,(byte)value);
        return store.reserveTrial("same@example.test",claim("same@example.test"),"127.0.0.1",hash).ok();
      }));
    }
    start.countDown();
    int allowed=0;
    for (Future<Boolean> item:work) if (item.get(5,TimeUnit.SECONDS)) allowed++;
    pool.shutdownNow();
    assertEquals(1,allowed);
  }
  @Test void trialStoreEnforcesGlobalHourlyCapacity() throws Exception {
    Store store=new Store(Files.createTempDirectory("license-trial-global"));
    byte[] hash=new byte[32];
    Arrays.fill(hash,(byte)9);
    for (int i=0;i<200;i++) {
      String email="global"+i+"@example.test";
      assertTrue(store.reserveTrial(email,claim(email),"ip-"+i,hash).ok());
    }
    assertFalse(store.reserveTrial("global-overflow@example.test",claim("global-overflow@example.test"),"fresh-ip",hash).ok());
  }
  @Test void trialClockExpiresAtBoundaryAndDisablesAfterFiveWrongCodes() throws Exception {
    MutableClock clock=new MutableClock(Instant.parse("2026-10-15T00:00:00Z"));
    Store store=new Store(Files.createTempDirectory("license-trial-clock"),clock);
    String expiredEmail="expired@example.test";
    byte[] correct={1,2,3};
    assertTrue(store.reserveTrial(expiredEmail,claim(expiredEmail),"127.0.0.1",correct).ok());
    clock.advanceMillis(900_000L);
    assertFalse(store.claimTrial(expiredEmail,claim(expiredEmail),correct,trialLicense(),"127.0.0.1").ok());

    String attemptsEmail="attempts@example.test";
    byte[] candidate={4,5,6};
    byte[] wrong={7,8,9};
    assertTrue(store.reserveTrial(attemptsEmail,claim(attemptsEmail),"127.0.0.2",candidate).ok());
    for (int i=0;i<5;i++) {
      assertFalse(store.claimTrial(attemptsEmail,claim(attemptsEmail),wrong,trialLicense(),"127.0.0.2").ok());
    }
    assertFalse(store.claimTrial(attemptsEmail,claim(attemptsEmail),candidate,trialLicense(),"127.0.0.2").ok());
  }
  @Test void trialClockResetsEmailRateWindowAtOneHourBoundary() throws Exception {
    MutableClock clock=new MutableClock(Instant.parse("2026-10-15T00:00:00Z"));
    Store store=new Store(Files.createTempDirectory("license-trial-rate-clock"),clock);
    String email="window@example.test";
    byte[] code={3,2,1};
    for (int i=0;i<3;i++) {
      assertTrue(store.reserveTrial(email,claim(email),"198.51.100.1",code).ok());
      store.invalidateTrialCode(email,code);
    }
    assertFalse(store.reserveTrial(email,claim(email),"198.51.100.1",code).ok());
    clock.advanceMillis(3_600_000L);
    assertTrue(store.reserveTrial(email,claim(email),"198.51.100.1",code).ok());
  }
  @Test void readerJsonBodyRejectsOversizedDeclaredAndChunkedPayloads() throws Exception {
    LicenseServer app=new LicenseServer(new LicenseServer.Config("127.0.0.1",0,Files.createTempDirectory("license-json-limit"),"012345678901234567890123","https://license.medwarp.cn",true,null,null,""));
    HttpServer http=app.createHttpServer();
    http.start();
    try {
      String base="http://127.0.0.1:"+http.getAddress().getPort();
      String oversized="{\"email\":\""+"a".repeat(64*1024)+"@example.test\"}";
      HttpResponse<String> declared=postRaw(base+"/reader3/supplyLicense",oversized);
      assertEquals(200,declared.statusCode());
      assertLegacyBodyError(declared.body(),"请求体过大");

      String chunked=postChunked(http.getAddress().getPort(),"/reader3/supplyLicense",oversized);
      assertLegacyBodyError(chunked,"请求体过大");
    } finally {
      http.stop(0);
    }
  }
  @Test void readerJsonBodyRejectsMalformedJson() throws Exception {
    LicenseServer app=new LicenseServer(new LicenseServer.Config("127.0.0.1",0,Files.createTempDirectory("license-json-malformed"),"012345678901234567890123","https://license.medwarp.cn",true,null,null,""));
    HttpServer http=app.createHttpServer();
    http.start();
    try {
      HttpResponse<String> response=postRaw("http://127.0.0.1:"+http.getAddress().getPort()+"/reader3/supplyLicense","{not-json");
      assertEquals(200,response.statusCode());
      assertLegacyBodyError(response.body(),"请求 JSON 格式错误");
    } finally {
      http.stop(0);
    }
  }
  @Test void plaintextSmtpRejectsResolvableNamesToAvoidRebinding() {
    Map<String,String> env=new HashMap<>(); env.put("LICENSE_SMTP_HOST","localhost");env.put("LICENSE_SMTP_PORT","2525");env.put("LICENSE_SMTP_MODE","PLAINTEXT");env.put("LICENSE_SMTP_ALLOW_PLAINTEXT_LOOPBACK","true");env.put("LICENSE_SMTP_FROM","sender@example.test");
    assertThrows(IllegalStateException.class,()->LicenseServer.SmtpConfig.fromEnvironment(env));
  }
  @Test void smtpPasswordFileRemovesOnlyOneFinalLineEnding() throws Exception {
    Path passwordFile=Files.createTempFile("reader-smtp-password",".txt");
    Files.writeString(passwordFile," leading-and-trailing-space \r\n",StandardCharsets.UTF_8);
    Map<String,String> env=new HashMap<>();
    env.put("LICENSE_SMTP_HOST","smtp.example.test");
    env.put("LICENSE_SMTP_FROM","sender@example.test");
    env.put("LICENSE_SMTP_USERNAME","reader");
    env.put("LICENSE_SMTP_PASSWORD_FILE",passwordFile.toString());
    assertEquals(" leading-and-trailing-space ",LicenseServer.SmtpConfig.fromEnvironment(env).password());
    assertEquals(" value \n",LicenseServer.SmtpConfig.withoutSingleFinalLineEnding(" value \n\n"));
  }
  @Test void forgedForwardedForCannotBypassTrialIpLimit() throws Exception {
    try(ServerSocket closedSoon=new ServerSocket(0,1,InetAddress.getByName("127.0.0.1"))){int port=closedSoon.getLocalPort();closedSoon.close();LicenseServer app=new LicenseServer(new LicenseServer.Config("127.0.0.1",0,Files.createTempDirectory("license-trial-xff"),"012345678901234567890123","https://license.medwarp.cn",true,null,null,"",LicenseServer.SmtpConfig.localPlaintextForTest(port)));HttpServer http=app.createHttpServer();http.start();try{String base="http://127.0.0.1:"+http.getAddress().getPort();for(int i=0;i<10;i++){Map<String,Object> response=post(base+"/reader3/sendCodeToEmail",null,Map.of("email","xff"+i+"@example.test"),Map.of("X-Forwarded-For","198.51.100."+i));assertFalse((Boolean)response.get("isSuccess"));}Map<String,Object> overflow=post(base+"/reader3/sendCodeToEmail",null,Map.of("email","xff-overflow@example.test"),Map.of("X-Forwarded-For","203.0.113.1"));assertFalse((Boolean)overflow.get("isSuccess"));assertEquals("请求过于频繁，请稍后再试",overflow.get("errorMsg"));}finally{http.stop(0);}}
  }
  @Test void twoSlowSmtpDeliveriesDoNotStarveHealthEndpoint() throws Exception {
    try(SlowSmtp smtp=new SlowSmtp(2)){LicenseServer app=new LicenseServer(new LicenseServer.Config("127.0.0.1",0,Files.createTempDirectory("license-trial-slow"),"012345678901234567890123","https://license.medwarp.cn",true,null,null,"",LicenseServer.SmtpConfig.localPlaintextForTest(smtp.port())));HttpServer http=app.createHttpServer();http.start();try{String base="http://127.0.0.1:"+http.getAddress().getPort();HttpClient client=HttpClient.newHttpClient();CompletableFuture<HttpResponse<String>> one=client.sendAsync(request(base+"/reader3/sendCodeToEmail",Map.of("email","slow-one@example.test")),HttpResponse.BodyHandlers.ofString());CompletableFuture<HttpResponse<String>> two=client.sendAsync(request(base+"/reader3/sendCodeToEmail",Map.of("email","slow-two@example.test")),HttpResponse.BodyHandlers.ofString());assertTrue(smtp.awaitAccepted());long started=System.nanoTime();Map<String,Object> health=get(base+"/healthz");assertEquals("ok",health.get("status"));assertTrue(TimeUnit.NANOSECONDS.toMillis(System.nanoTime()-started)<1000,"health must retain an HTTP worker while SMTP is slow");smtp.close();one.get(5,TimeUnit.SECONDS);two.get(5,TimeUnit.SECONDS);}finally{http.stop(0);}}
  }
  @Test void trialDeliveryFailureReturnsReturnDataAndDoesNotLeaveUsableCode() throws Exception {
    try (ServerSocket closedSoon=new ServerSocket(0, 1, InetAddress.getByName("127.0.0.1"))) {
      int unavailablePort=closedSoon.getLocalPort(); closedSoon.close();
      LicenseServer app=new LicenseServer(new LicenseServer.Config("127.0.0.1",0,Files.createTempDirectory("license-trial-mail-failure"),"012345678901234567890123","https://license.medwarp.cn",true,null,null,"",LicenseServer.SmtpConfig.localPlaintextForTest(unavailablePort)));
      HttpServer http=app.createHttpServer();http.start();
      try {String base="http://127.0.0.1:"+http.getAddress().getPort();Map<String,Object> failed=post(base+"/reader3/sendCodeToEmail",null,Map.of("email","failure@example.test"));assertFalse((Boolean)failed.get("isSuccess"));assertNull(failed.get("data"));Map<String,Object> supply=post(base+"/reader3/supplyLicense",null,Map.of("email","failure@example.test","code","12345678"));assertFalse((Boolean)supply.get("isSuccess"));assertNull(supply.get("data"));} finally {http.stop(0);}
    }
  }
  static Map<String,Object> post(String url,String token,Map<String,Object> body)throws Exception {return post(url,token,body,Map.of());}
  static Map<String,Object> post(String url,String token,Map<String,Object> body,Map<String,String> headers)throws Exception {HttpRequest.Builder b=HttpRequest.newBuilder(URI.create(url)).header("Content-Type","application/json").POST(HttpRequest.BodyPublishers.ofString(JSON.writeValueAsString(body)));if(token!=null)b.header("Authorization","Bearer "+token);headers.forEach(b::header);HttpResponse<String> r=HttpClient.newHttpClient().send(b.build(),HttpResponse.BodyHandlers.ofString());assertTrue(r.statusCode()==200||r.statusCode()==201);return JSON.readValue(r.body(),new TypeReference<Map<String,Object>>(){});}
  static HttpResponse<String> postRaw(String url,String body)throws Exception {
    HttpRequest request=HttpRequest.newBuilder(URI.create(url))
        .header("Content-Type","application/json")
        .POST(HttpRequest.BodyPublishers.ofString(body,StandardCharsets.UTF_8))
        .build();
    return HttpClient.newHttpClient().send(request,HttpResponse.BodyHandlers.ofString());
  }
  static String postChunked(int port,String path,String body)throws Exception {
    byte[] bytes=body.getBytes(StandardCharsets.UTF_8);
    try (Socket socket=new Socket(InetAddress.getByName("127.0.0.1"),port)) {
      OutputStream out=socket.getOutputStream();
      out.write(("POST "+path+" HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n").getBytes(StandardCharsets.US_ASCII));
      out.write(Integer.toHexString(bytes.length).getBytes(StandardCharsets.US_ASCII));
      out.write("\r\n".getBytes(StandardCharsets.US_ASCII));
      out.write(bytes);
      out.write("\r\n0\r\n\r\n".getBytes(StandardCharsets.US_ASCII));
      out.flush();
      return new String(socket.getInputStream().readAllBytes(),StandardCharsets.UTF_8);
    }
  }
  static void assertLegacyBodyError(String response,String expectedError)throws Exception {
    int bodyStart=response.indexOf("\r\n\r\n");
    String json=bodyStart>=0 ? response.substring(bodyStart+4) : response;
    Map<String,Object> payload=JSON.readValue(json,new TypeReference<Map<String,Object>>(){});
    assertFalse((Boolean)payload.get("isSuccess"));
    assertEquals(expectedError,payload.get("errorMsg"));
    assertNull(payload.get("data"));
  }
  static HttpRequest request(String url,Map<String,Object> body)throws Exception{return HttpRequest.newBuilder(URI.create(url)).header("Content-Type","application/json").POST(HttpRequest.BodyPublishers.ofString(JSON.writeValueAsString(body))).build();}
  static byte[] claim(String email){return ("claim\0"+email).getBytes(StandardCharsets.UTF_8);}
  static Map<String,Object> trialLicense(){return Map.of("id",UUID.randomUUID().toString());}
  static Map<String,Object> get(String url)throws Exception {HttpResponse<String> r=HttpClient.newHttpClient().send(HttpRequest.newBuilder(URI.create(url)).GET().build(),HttpResponse.BodyHandlers.ofString());assertEquals(200,r.statusCode());return JSON.readValue(r.body(),new TypeReference<Map<String,Object>>(){});}
  static Map<String,Object> decrypt(String content,PublicKey key)throws Exception {byte[] data=Base64.getDecoder().decode(content);int block=((RSAPublicKey)key).getModulus().bitLength()/8;Cipher c=Cipher.getInstance("RSA");c.init(Cipher.DECRYPT_MODE,key);java.io.ByteArrayOutputStream out=new java.io.ByteArrayOutputStream();for(int i=0;i<data.length;i+=block)out.write(c.doFinal(data,i,Math.min(block,data.length-i)));return JSON.readValue(new String(out.toByteArray(),StandardCharsets.UTF_8),new TypeReference<Map<String,Object>>(){});}
  static final class FakeSmtp implements AutoCloseable {
    private final ServerSocket server; private final ExecutorService executor=Executors.newSingleThreadExecutor(); private final BlockingQueue<String> messages=new ArrayBlockingQueue<>(2);
    FakeSmtp() throws IOException {server=new ServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));executor.submit(()->{try(Socket socket=server.accept();BufferedReader in=new BufferedReader(new InputStreamReader(socket.getInputStream(),StandardCharsets.US_ASCII));BufferedWriter out=new BufferedWriter(new OutputStreamWriter(socket.getOutputStream(),StandardCharsets.US_ASCII))){reply(out,"220 test");String line;while((line=in.readLine())!=null){if(line.startsWith("EHLO")){reply(out,"250-test");reply(out,"250 SIZE 100000");}else if(line.startsWith("MAIL FROM")||line.startsWith("RCPT TO")){reply(out,"250 OK");}else if(line.equals("DATA")){reply(out,"354 send");StringBuilder body=new StringBuilder();while((line=in.readLine())!=null&&!line.equals("."))body.append(line).append('\n');messages.add(body.toString());reply(out,"250 queued");}else if(line.equals("QUIT")){reply(out,"221 bye");break;}else reply(out,"250 OK");}}catch(IOException ignored){}});}
    int port(){return server.getLocalPort();} String take()throws Exception{String value=messages.poll(5,TimeUnit.SECONDS);assertNotNull(value,"fake SMTP did not receive a message");return value;} private static void reply(BufferedWriter out,String value)throws IOException{out.write(value+"\r\n");out.flush();} public void close()throws Exception{server.close();executor.shutdownNow();}
  }
  static final class SlowSmtp implements AutoCloseable {
    private final ServerSocket server; private final ExecutorService executor=Executors.newSingleThreadExecutor(); private final CountDownLatch accepted; private final List<Socket> sockets=Collections.synchronizedList(new ArrayList<>());
    SlowSmtp(int clients)throws IOException{server=new ServerSocket(0,1,InetAddress.getByName("127.0.0.1"));accepted=new CountDownLatch(clients);executor.submit(()->{for(int i=0;i<clients;i++){try{Socket socket=server.accept();sockets.add(socket);accepted.countDown();}catch(IOException ignored){break;}}});}
    int port(){return server.getLocalPort();} boolean awaitAccepted()throws InterruptedException{return accepted.await(5,TimeUnit.SECONDS);} public void close()throws Exception{server.close();synchronized(sockets){for(Socket socket:sockets)socket.close();}executor.shutdownNow();}
  }
  static final class MutableClock extends Clock {
    private long millis;
    MutableClock(Instant initial) { millis=initial.toEpochMilli(); }
    void advanceMillis(long delta) { millis+=delta; }
    @Override public ZoneId getZone() { return ZoneOffset.UTC; }
    @Override public Clock withZone(ZoneId zone) { return this; }
    @Override public Instant instant() { return Instant.ofEpochMilli(millis); }
  }
}
