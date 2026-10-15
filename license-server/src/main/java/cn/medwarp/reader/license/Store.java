package cn.medwarp.reader.license;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.nio.file.Path;
import java.nio.file.Files;
import java.sql.*;
import java.time.Clock;
import java.util.*;

/** SQLite persistence. Transactions protect activation-count decisions. */
final class Store {
  private static final ObjectMapper JSON = new ObjectMapper();
  private final String url;
  private final Clock clock;
  Store(Path data) throws Exception {
    this(data, Clock.systemUTC());
  }
  Store(Path data, Clock clock) throws Exception {
    Class.forName("org.sqlite.JDBC"); url="jdbc:sqlite:"+data.resolve("licenses.db").toAbsolutePath();
    this.clock=Objects.requireNonNull(clock, "clock");
    Files.createDirectories(data);
    try(Connection c=conn();Statement s=c.createStatement()) {
      s.execute("PRAGMA journal_mode=WAL");
      s.execute("CREATE TABLE IF NOT EXISTS licenses (id TEXT PRIMARY KEY,subject TEXT NOT NULL,host TEXT NOT NULL,expires_at INTEGER NOT NULL,user_limit INTEGER NOT NULL,instance_limit INTEGER NOT NULL,claims_json TEXT NOT NULL,revoked_at INTEGER,created_at INTEGER NOT NULL)");
      s.execute("CREATE TABLE IF NOT EXISTS activations (license_id TEXT NOT NULL,instance_id TEXT NOT NULL,first_ip TEXT NOT NULL,last_ip TEXT NOT NULL,created_at INTEGER NOT NULL,last_seen_at INTEGER NOT NULL,PRIMARY KEY(license_id,instance_id))");
      s.execute("CREATE TABLE IF NOT EXISTS legacy_licenses (type TEXT NOT NULL,code TEXT NOT NULL,claims_json TEXT NOT NULL,instance_limit INTEGER NOT NULL,expires_at INTEGER NOT NULL,revoked_at INTEGER,created_at INTEGER NOT NULL,PRIMARY KEY(type,code))");
      s.execute("CREATE TABLE IF NOT EXISTS legacy_activations (id TEXT PRIMARY KEY,type TEXT NOT NULL,code TEXT NOT NULL,active_order INTEGER NOT NULL,first_ip TEXT NOT NULL,last_ip TEXT NOT NULL,created_at INTEGER NOT NULL,last_seen_at INTEGER NOT NULL)");
      s.execute("CREATE TABLE IF NOT EXISTS trial_requests (email TEXT PRIMARY KEY,code_hash BLOB,expires_at INTEGER NOT NULL,attempts INTEGER NOT NULL,window_started_at INTEGER NOT NULL,window_count INTEGER NOT NULL,updated_at INTEGER NOT NULL)");
      s.execute("CREATE TABLE IF NOT EXISTS trial_ip_limits (ip TEXT PRIMARY KEY,window_started_at INTEGER NOT NULL,window_count INTEGER NOT NULL)");
      s.execute("CREATE TABLE IF NOT EXISTS trial_global_limits (name TEXT PRIMARY KEY,window_started_at INTEGER NOT NULL,window_count INTEGER NOT NULL)");
      s.execute("CREATE TABLE IF NOT EXISTS trial_claims (claim_hash BLOB PRIMARY KEY,legacy_id TEXT NOT NULL,claimed_at INTEGER NOT NULL)");
      s.execute("CREATE TABLE IF NOT EXISTS audit (id INTEGER PRIMARY KEY AUTOINCREMENT,at INTEGER NOT NULL,action TEXT NOT NULL,license_id TEXT,actor TEXT NOT NULL,ip TEXT NOT NULL,detail TEXT NOT NULL)");
    }
  }
  private Connection conn()throws SQLException{Connection c=DriverManager.getConnection(url);try(Statement s=c.createStatement()){s.execute("PRAGMA busy_timeout=5000");}return c;}
  void insert(String id,String subject,String host,long expiry,int users,int instances,String claims,String actor,String ip)throws Exception {try(Connection c=conn();PreparedStatement p=c.prepareStatement("INSERT INTO licenses VALUES(?,?,?,?,?,?,?,?,?)")){p.setString(1,id);p.setString(2,subject);p.setString(3,host);p.setLong(4,expiry);p.setInt(5,users);p.setInt(6,instances);p.setString(7,claims);p.setNull(8,Types.INTEGER);p.setLong(9,now());p.executeUpdate();audit(c,"issued",id,actor,ip);}}
  void insertLegacy(String type, String code, Map<String,Object> claims, int instances, long expiry, String actor, String ip) throws Exception {
    try (Connection c = conn(); PreparedStatement p = c.prepareStatement("INSERT INTO legacy_licenses VALUES(?,?,?,?,?,?,?)")) {
      p.setString(1, type); p.setString(2, code); p.setString(3, JSON.writeValueAsString(claims));
      p.setInt(4, instances); p.setLong(5, expiry); p.setNull(6, Types.INTEGER); p.setLong(7, now());
      p.executeUpdate();
      audit(c, "legacy_issued", type + ":" + code, actor, ip);
    }
  }
  Map<String,Object> activateLegacy(Map<String,Object> candidate,String ip)throws Exception {String type=str(candidate,"type"),code=str(candidate,"code");if(Boolean.TRUE.equals(candidate.get("verified")))throw err(400,"already_used","许可证已经激活");try(Connection c=conn()){c.setAutoCommit(false);try{Map<String,Object> issued=legacy(c,type,code);candidate=new LinkedHashMap<>(issued);long expiry=num(issued.get("expiredAt"));if(expired(expiry))throw err(403,"expired","许可证已过期");int used=legacyCount(c,type,code),limit=((Number)issued.get("instances")).intValue();if(used>=limit)throw err(403,"instance_limit_reached","激活实例数已达上限");String id=UUID.randomUUID().toString();candidate.put("id",id);candidate.put("verified",true);candidate.put("verifyTime",now());try(PreparedStatement p=c.prepareStatement("INSERT INTO legacy_activations VALUES(?,?,?,?,?,?,?,?)")){p.setString(1,id);p.setString(2,type);p.setString(3,code);p.setInt(4,used+1);p.setString(5,ip);p.setString(6,ip);p.setLong(7,now());p.setLong(8,now());p.executeUpdate();}audit(c,"legacy_activated",id,"legacy-client",ip);c.commit();return candidate;}catch(Exception e){c.rollback();throw e;}}}
  Map<String,Object> legacyStatus(String id,String ip)throws Exception {try(Connection c=conn();PreparedStatement p=c.prepareStatement("SELECT l.expires_at,l.revoked_at,a.last_ip,a.last_seen_at FROM legacy_activations a JOIN legacy_licenses l ON a.type=l.type AND a.code=l.code WHERE a.id=?")){p.setString(1,id);try(ResultSet r=p.executeQuery()){Map<String,Object> out=new LinkedHashMap<>();if(!r.next()){out.put("isValid",false);out.put("errorMsg","密钥未激活");return out;}boolean valid=r.getObject("revoked_at")==null&&!expired(r.getLong("expires_at"));out.put("isValid",valid);out.put("errorMsg",valid?"":(r.getObject("revoked_at")!=null?"密钥已吊销":"密钥已过期"));long previousTime=r.getLong("last_seen_at");String previousIp=r.getString("last_ip");if(now()<previousTime+600000L&&!ip.equals(previousIp)){Map<String,Object> repeat=new LinkedHashMap<>();repeat.put("lastOnlineTime",previousTime);repeat.put("lastOnlineIp",previousIp);out.put("repeat",repeat);}try(PreparedStatement u=c.prepareStatement("UPDATE legacy_activations SET last_ip=?,last_seen_at=? WHERE id=?")){u.setString(1,ip);u.setLong(2,now());u.setString(3,id);u.executeUpdate();}audit(c,"legacy_validated",id,"legacy-client",ip);return out;}}}
  Map<String,Object> license(String id)throws Exception {try(Connection c=conn();PreparedStatement p=c.prepareStatement("SELECT * FROM licenses WHERE id=?")){p.setString(1,id);try(ResultSet r=p.executeQuery()){if(!r.next())throw err(404,"not_found","许可证不存在");Map<String,Object> out=new LinkedHashMap<>();out.put("id",id);out.put("subject",r.getString("subject"));out.put("host",r.getString("host"));out.put("expiresAt",r.getLong("expires_at"));out.put("userMaxLimit",r.getInt("user_limit"));out.put("instances",r.getInt("instance_limit"));out.put("revoked",r.getObject("revoked_at")!=null);out.put("activations",count(c,id));return out;}}}
  void revoke(String id,String actor,String ip)throws Exception{try(Connection c=conn();PreparedStatement p=c.prepareStatement("UPDATE licenses SET revoked_at=? WHERE id=? AND revoked_at IS NULL")){p.setLong(1,now());p.setString(2,id);if(p.executeUpdate()==0)throw err(404,"not_found","许可证不存在或已经吊销");audit(c,"revoked",id,actor,ip);}}
  void revokeLegacy(String type, String code, String actor, String ip) throws Exception {
    try (Connection c = conn(); PreparedStatement p = c.prepareStatement("UPDATE legacy_licenses SET revoked_at=? WHERE type=? AND code=? AND revoked_at IS NULL")) {
      p.setLong(1, now()); p.setString(2, type); p.setString(3, code);
      if (p.executeUpdate() == 0) throw err(404, "not_found", "许可证不存在或已经吊销");
      audit(c, "legacy_revoked", type + ":" + code, actor, ip);
    }
  }
  /** Reserves one short-lived code without storing the plaintext value. */
  synchronized TrialReservation reserveTrial(String email, byte[] claimHash, String ip, byte[] codeHash) throws Exception {
    try (Connection c=conn()) {
      c.setAutoCommit(false);
      try {
        long at=now();
        cleanupTrials(c, at);
        if (hasTrialClaim(c,claimHash)) {
          c.commit();
          return TrialReservation.fail("该邮箱已领取过试用许可证");
        }
        if (countTrialClaims(c) >= 100_000) {
          c.commit();
          return TrialReservation.fail("试用配额已用尽");
        }
        if (activeTrialCodeCount(c, at) >= 1_000) {
          c.commit();
          return TrialReservation.fail("试用请求繁忙，请稍后重试");
        }
        TrialRow old=trialRow(c,email);
        if (old!=null && old.expiresAt>at && old.codeHash!=null) {
          c.commit();
          return TrialReservation.fail("验证码仍在有效期，请勿重复获取");
        }
        long windowStart=old==null || old.windowStartedAt+3_600_000L<=at ? at : old.windowStartedAt;
        int windowCount=old==null || old.windowStartedAt+3_600_000L<=at ? 0 : old.windowCount;
        if (windowCount>=3) {
          c.commit();
          return TrialReservation.fail("请求过于频繁，请稍后再试");
        }
        IpLimit limit=ipLimit(c,ip);
        long ipWindow=limit==null || limit.windowStartedAt+3_600_000L<=at ? at : limit.windowStartedAt;
        int ipCount=limit==null || limit.windowStartedAt+3_600_000L<=at ? 0 : limit.windowCount;
        if (ipCount>=10) {
          c.commit();
          return TrialReservation.fail("请求过于频繁，请稍后再试");
        }
        IpLimit global=globalLimit(c,"send");
        long globalWindow=global==null || global.windowStartedAt+3_600_000L<=at ? at : global.windowStartedAt;
        int globalCount=global==null || global.windowStartedAt+3_600_000L<=at ? 0 : global.windowCount;
        if (globalCount>=200) {
          c.commit();
          return TrialReservation.fail("试用请求繁忙，请稍后重试");
        }
        try (PreparedStatement p=c.prepareStatement("INSERT INTO trial_requests(email,code_hash,expires_at,attempts,window_started_at,window_count,updated_at) VALUES(?,?,?,?,?,?,?) ON CONFLICT(email) DO UPDATE SET code_hash=excluded.code_hash,expires_at=excluded.expires_at,attempts=0,window_started_at=excluded.window_started_at,window_count=excluded.window_count,updated_at=excluded.updated_at")) {
          p.setString(1,email); p.setBytes(2,codeHash); p.setLong(3,at+900_000L); p.setInt(4,0);
          p.setLong(5,windowStart); p.setInt(6,windowCount+1); p.setLong(7,at); p.executeUpdate();
        }
        try (PreparedStatement p=c.prepareStatement("INSERT INTO trial_ip_limits(ip,window_started_at,window_count) VALUES(?,?,?) ON CONFLICT(ip) DO UPDATE SET window_started_at=excluded.window_started_at,window_count=excluded.window_count")) {
          p.setString(1,ip); p.setLong(2,ipWindow); p.setInt(3,ipCount+1); p.executeUpdate();
        }
        try (PreparedStatement p=c.prepareStatement("INSERT INTO trial_global_limits(name,window_started_at,window_count) VALUES(?,?,?) ON CONFLICT(name) DO UPDATE SET window_started_at=excluded.window_started_at,window_count=excluded.window_count")) {
          p.setString(1,"send"); p.setLong(2,globalWindow); p.setInt(3,globalCount+1); p.executeUpdate();
        }
        audit(c,"trial_code_reserved",null,"trial-client",ip);
        c.commit();
        return TrialReservation.ok(codeHash);
      } catch (Exception e) {
        c.rollback();
        throw e;
      }
    }
  }

  synchronized void invalidateTrialCode(String email, byte[] codeHash) throws Exception {
    try (Connection c=conn(); PreparedStatement p=c.prepareStatement("UPDATE trial_requests SET code_hash=NULL,expires_at=0,updated_at=? WHERE email=? AND code_hash=?")) {
      p.setLong(1,now()); p.setString(2,email); p.setBytes(3,codeHash); p.executeUpdate();
    }
  }
  /** Checks, consumes and persists a trial claim in one SQLite transaction. */
  synchronized TrialClaim claimTrial(String email, byte[] claimHash, byte[] candidateHash, Map<String,Object> license, String ip) throws Exception {
    try (Connection c=conn()) {
      c.setAutoCommit(false);
      try {
        long at=now();
        TrialRow row=trialRow(c,email);
        if (hasTrialClaim(c,claimHash) || row==null || row.codeHash==null || row.expiresAt<=at) {
          c.commit();
          return TrialClaim.fail("验证码错误或已过期");
        }
        if (!java.security.MessageDigest.isEqual(row.codeHash,candidateHash)) {
          int attempts=row.attempts+1;
          try (PreparedStatement p=c.prepareStatement("UPDATE trial_requests SET attempts=?,code_hash=CASE WHEN ?>=5 THEN NULL ELSE code_hash END,expires_at=CASE WHEN ?>=5 THEN 0 ELSE expires_at END,updated_at=? WHERE email=?")) {
            p.setInt(1,attempts); p.setInt(2,attempts); p.setInt(3,attempts); p.setLong(4,at); p.setString(5,email); p.executeUpdate();
          }
          c.commit();
          return TrialClaim.fail("验证码错误或已过期");
        }
        if (countTrialClaims(c) >= 100_000) {
          c.commit();
          return TrialClaim.fail("试用配额已用尽");
        }
        String id=str(license,"id");
        try (PreparedStatement p=c.prepareStatement("INSERT INTO legacy_licenses VALUES(?,?,?,?,?,?,?)")) {
          p.setString(1,"trial"); p.setString(2,email); p.setString(3,JSON.writeValueAsString(license)); p.setInt(4,1);
          p.setLong(5,0L); p.setNull(6,Types.INTEGER); p.setLong(7,at); p.executeUpdate();
        }
        try (PreparedStatement p=c.prepareStatement("INSERT INTO trial_claims(claim_hash,legacy_id,claimed_at) VALUES(?,?,?)")) {
          p.setBytes(1,claimHash); p.setString(2,id); p.setLong(3,at); p.executeUpdate();
        }
        try (PreparedStatement p=c.prepareStatement("UPDATE trial_requests SET code_hash=NULL,expires_at=0,updated_at=? WHERE email=?")) {
          p.setLong(1,at); p.setString(2,email); p.executeUpdate();
        }
        audit(c,"trial_claimed",id,"trial-client",ip);
        c.commit();
        return TrialClaim.success();
      } catch (Exception e) {
        c.rollback();
        throw e;
      }
    }
  }

  static final class TrialReservation {
    private final boolean ok;
    private final String message;
    private final byte[] hash;

    private TrialReservation(boolean ok,String message,byte[] hash) { this.ok=ok; this.message=message; this.hash=hash; }
    static TrialReservation ok(byte[] h) { return new TrialReservation(true,"",h); }
    static TrialReservation fail(String m) { return new TrialReservation(false,m,null); }
    boolean ok() { return ok; }
    String message() { return message; }
    byte[] hash() { return hash; }
  }

  static final class TrialClaim {
    private final boolean ok;
    private final String message;

    private TrialClaim(boolean ok,String message) { this.ok=ok; this.message=message; }
    static TrialClaim success() { return new TrialClaim(true,""); }
    static TrialClaim fail(String m) { return new TrialClaim(false,m); }
    boolean ok() { return ok; }
    String message() { return message; }
  }
  boolean revoked(String id)throws Exception{try(Connection c=conn()){return revoked(c,id);}}
  boolean isActivated(String id,String instance)throws Exception{try(Connection c=conn()){return active(c,id,instance);}}
  boolean activate(String id,String instance,String ip)throws Exception{try(Connection c=conn()){c.setAutoCommit(false);try{if(revoked(c,id))throw err(403,"revoked","许可证已吊销");if(active(c,id,instance)){touch(c,id,instance,ip);c.commit();return false;}if(count(c,id)>=limit(c,id))throw err(403,"instance_limit_reached","激活实例数已达上限");try(PreparedStatement p=c.prepareStatement("INSERT INTO activations VALUES(?,?,?,?,?,?)")){p.setString(1,id);p.setString(2,instance);p.setString(3,ip);p.setString(4,ip);p.setLong(5,now());p.setLong(6,now());p.executeUpdate();}audit(c,"activated",id,"client",ip);c.commit();return true;}catch(Exception e){c.rollback();throw e;}}}
  void touch(String id,String instance,String ip)throws Exception{try(Connection c=conn()){touch(c,id,instance,ip);audit(c,"validated",id,"client",ip);}}
  List<Map<String,Object>> audit()throws Exception{try(Connection c=conn();Statement s=c.createStatement();ResultSet r=s.executeQuery("SELECT * FROM audit ORDER BY id DESC LIMIT 200")){List<Map<String,Object>> list=new ArrayList<>();while(r.next()){Map<String,Object> x=new LinkedHashMap<>();x.put("at",r.getLong("at"));x.put("action",r.getString("action"));x.put("licenseId",r.getString("license_id"));x.put("actor",r.getString("actor"));x.put("ip",r.getString("ip"));list.add(x);}return list;}}
  private Map<String,Object> legacy(Connection c,String type,String code)throws Exception{try(PreparedStatement p=c.prepareStatement("SELECT claims_json,revoked_at FROM legacy_licenses WHERE type=? AND code=?")){p.setString(1,type);p.setString(2,code);try(ResultSet r=p.executeQuery()){if(!r.next())throw err(403,"unknown_license","许可证不存在");if(r.getObject(2)!=null)throw err(403,"revoked","许可证已吊销");return JSON.readValue(r.getString(1),new TypeReference<Map<String,Object>>(){});}}}
  private boolean revoked(Connection c,String id)throws Exception{try(PreparedStatement p=c.prepareStatement("SELECT revoked_at FROM licenses WHERE id=?")){p.setString(1,id);try(ResultSet r=p.executeQuery()){if(!r.next())throw err(403,"unknown_license","许可证不存在");return r.getObject(1)!=null;}}}
  private int legacyCount(Connection c,String type,String code)throws SQLException{try(PreparedStatement p=c.prepareStatement("SELECT count(*) FROM legacy_activations WHERE type=? AND code=?")){p.setString(1,type);p.setString(2,code);try(ResultSet r=p.executeQuery()){r.next();return r.getInt(1);}}}
  private int count(Connection c,String id)throws SQLException{try(PreparedStatement p=c.prepareStatement("SELECT count(*) FROM activations WHERE license_id=?")){p.setString(1,id);try(ResultSet r=p.executeQuery()){r.next();return r.getInt(1);}}}
  private int limit(Connection c,String id)throws SQLException{try(PreparedStatement p=c.prepareStatement("SELECT instance_limit FROM licenses WHERE id=?")){p.setString(1,id);try(ResultSet r=p.executeQuery()){if(!r.next())throw new SQLException("unknown license");return r.getInt(1);}}}
  private boolean active(Connection c,String id,String instance)throws SQLException{try(PreparedStatement p=c.prepareStatement("SELECT 1 FROM activations WHERE license_id=? AND instance_id=?")){p.setString(1,id);p.setString(2,instance);try(ResultSet r=p.executeQuery()){return r.next();}}}
  private TrialRow trialRow(Connection c,String email)throws SQLException {
    try (PreparedStatement p=c.prepareStatement("SELECT code_hash,expires_at,attempts,window_started_at,window_count FROM trial_requests WHERE email=?")) {
      p.setString(1,email);
      try (ResultSet r=p.executeQuery()) {
        if (!r.next()) return null;
        return new TrialRow(r.getBytes(1),r.getLong(2),r.getInt(3),r.getLong(4),r.getInt(5));
      }
    }
  }
  private IpLimit ipLimit(Connection c,String ip)throws SQLException {
    try (PreparedStatement p=c.prepareStatement("SELECT window_started_at,window_count FROM trial_ip_limits WHERE ip=?")) {
      p.setString(1,ip);
      try (ResultSet r=p.executeQuery()) { return r.next() ? new IpLimit(r.getLong(1),r.getInt(2)) : null; }
    }
  }
  private boolean hasTrialClaim(Connection c,byte[] claimHash)throws SQLException {
    try (PreparedStatement p=c.prepareStatement("SELECT 1 FROM trial_claims WHERE claim_hash=?")) {
      p.setBytes(1,claimHash);
      try (ResultSet r=p.executeQuery()) { return r.next(); }
    }
  }
  private int countTrialClaims(Connection c)throws SQLException {
    try (Statement s=c.createStatement(); ResultSet r=s.executeQuery("SELECT count(*) FROM trial_claims")) { r.next(); return r.getInt(1); }
  }
  private int activeTrialCodeCount(Connection c,long at)throws SQLException {
    try (PreparedStatement p=c.prepareStatement("SELECT count(*) FROM trial_requests WHERE code_hash IS NOT NULL AND expires_at>?")) {
      p.setLong(1,at);
      try (ResultSet r=p.executeQuery()) { r.next(); return r.getInt(1); }
    }
  }
  private IpLimit globalLimit(Connection c,String name)throws SQLException {
    try (PreparedStatement p=c.prepareStatement("SELECT window_started_at,window_count FROM trial_global_limits WHERE name=?")) {
      p.setString(1,name);
      try (ResultSet r=p.executeQuery()) { return r.next() ? new IpLimit(r.getLong(1),r.getInt(2)) : null; }
    }
  }
  private void cleanupTrials(Connection c,long at)throws SQLException {
    try (PreparedStatement p=c.prepareStatement("DELETE FROM trial_requests WHERE updated_at<?")) {
      p.setLong(1,at-7L*24*3_600_000L); p.executeUpdate();
    }
    try (PreparedStatement p=c.prepareStatement("DELETE FROM trial_ip_limits WHERE window_started_at<?")) {
      p.setLong(1,at-24L*3_600_000L); p.executeUpdate();
    }
  }
  private static final class TrialRow {
    final byte[] codeHash;
    final long expiresAt,windowStartedAt;
    final int attempts,windowCount;
    TrialRow(byte[] h,long e,int a,long w,int c) { codeHash=h; expiresAt=e; attempts=a; windowStartedAt=w; windowCount=c; }
  }
  private static final class IpLimit {
    final long windowStartedAt;
    final int windowCount;
    IpLimit(long w,int c) { windowStartedAt=w; windowCount=c; }
  }
  private void touch(Connection c,String id,String instance,String ip)throws SQLException{try(PreparedStatement p=c.prepareStatement("UPDATE activations SET last_ip=?,last_seen_at=? WHERE license_id=? AND instance_id=?")){p.setString(1,ip);p.setLong(2,now());p.setString(3,id);p.setString(4,instance);p.executeUpdate();}}
  private void audit(Connection c,String action,String license,String actor,String ip)throws SQLException{try(PreparedStatement p=c.prepareStatement("INSERT INTO audit(at,action,license_id,actor,ip,detail) VALUES(?,?,?,?,?,?)")){p.setLong(1,now());p.setString(2,action);p.setString(3,license);p.setString(4,actor);p.setString(5,ip);p.setString(6,"{}");p.executeUpdate();}}
  private static LicenseServer.ApiError err(int s,String c,String m){return new LicenseServer.ApiError(s,c,m);}
  private static String str(Map<String,Object> m,String n)throws LicenseServer.ApiError{Object v=m.get(n);if(!(v instanceof String)||((String)v).trim().isEmpty())throw err(400,"malformed_license","缺少许可证字段 "+n);return(String)v;}
  private static long num(Object o){return o instanceof Number?((Number)o).longValue():0;}
  private boolean expired(long x){return x>0&&x<now();}
  private long now(){return clock.millis();}
}
