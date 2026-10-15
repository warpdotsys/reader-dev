package me.ag2s.epublib.util;

import org.junit.Test;

import java.nio.charset.StandardCharsets;

import static org.junit.Assert.assertTrue;

public class ResourceUtilEncodingTest {

    @Test
    public void chapterResourceUsesDeclaredUtf8Encoding() throws Exception {
        String html = new String(ResourceUtil.createChapterResource(
                "第一章 开始", "这是第一章的正文。", "<html>{title}|{content}</html>",
                "Text/chapter_0.html").getData(), StandardCharsets.UTF_8);

        assertTrue(html.contains("第一章"));
        assertTrue(html.contains("开始"));
        assertTrue(html.contains("这是第一章的正文。"));
    }

    @Test
    public void publicResourceUsesDeclaredUtf8Encoding() throws Exception {
        String html = new String(ResourceUtil.createPublicResource(
                "中文书名", "作者", "简介", "分类", "一万字",
                "<html>{name}|{author}|{intro}|{kind}|{wordCount}</html>",
                "Text/intro.html").getData(), StandardCharsets.UTF_8);

        assertTrue(html.contains("中文书名|作者|"));
        assertTrue(html.contains("简介"));
        assertTrue(html.contains("|分类|一万字"));
    }
}
