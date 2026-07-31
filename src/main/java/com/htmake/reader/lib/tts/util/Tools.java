package com.htmake.reader.lib.tts.util;

import okhttp3.OkHttpClient;
import okhttp3.Request;
import okhttp3.Response;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.text.SimpleDateFormat;
import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;
import java.util.Date;
import java.util.Locale;
import java.util.UUID;
import java.util.regex.Pattern;

public class Tools {

    public static final Pattern NO_VOICE_PATTERN = Pattern.compile("[\\s\\p{C}\\p{P}\\p{Z}\\p{S}]");

    public static final String SDF = "EEE MMM dd yyyy HH:mm:ss 'GMT'Z";

    public static final DateTimeFormatter DTF = DateTimeFormatter.ofPattern("yyyyMMdd_HHmmss");

    public static Logger log = LoggerFactory.getLogger(Tools.class);

    private static OkHttpClient client = new OkHttpClient();

    public Tools() {
    }

    public static String httpGet(String url) {
        try {
            Request request = new Request.Builder().url(url).build();
            Response response = client.newCall(request).execute();
            log.info("response.toString():{}", response.toString());
            log.info("response.isSuccessful():{}", response.isSuccessful());
            if (response.isSuccessful()) {
                String body = response.body().string();
                return body;
            }
            throw new RuntimeException(String.format("request\uff1a%s fail, message:%s", url, response.code()));
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
    }

    public static boolean isNoVoice(CharSequence text) {
        return NO_VOICE_PATTERN.matcher(text).replaceAll("").isEmpty();
    }

    public static void sleep(int seconds) {
        try {
            Thread.sleep(seconds * 1000);
        } catch (InterruptedException e) {
            // ignored
        }
    }

    public static String date() {
        return new SimpleDateFormat(SDF).format(new Date());
    }

    public static String localDateTime() {
        return LocalDateTime.now().format(DTF);
    }

    public static String localeToEmoji(Locale locale) {
        String country = locale.getCountry();
        if ("TW".equals(country) && "CN".equals(Locale.getDefault().getCountry())) {
            return "";
        }
        int firstCodePoint = Character.codePointAt(country, 0) - 65 + 127462;
        int secondCodePoint = Character.codePointAt(country, 1) - 65 + 127462;
        return new String(Character.toChars(firstCodePoint)) + new String(Character.toChars(secondCodePoint));
    }

    public static String getRandomId() {
        return UUID.randomUUID().toString().replace("-", "");
    }
}
