pub struct DeepinkBookSource;

impl DeepinkBookSource {
    pub fn generate(name: &str, url: &str, md5: &str) {
        let text = format!(
            "{{\n  \"name\": \"{name} by [yuedu.best]\",\n  \"url\": \"{url}\",\n  \"version\": 100,\n  \"search\": {{\n    \"url\": \"http://api.yuedu.best/yuedu/searchBook@post->{{\\\"key\\\":\\\"${{key}}\\\", \\\"bookSourceCode\\\":\\\"{md5}\\\"}}\",\n    \"charset\": \"utf-8\",\n    \"list\": \"$.[*]\",\n    \"name\": \"$.name\",\n    \"author\": \"$.author\",\n    \"cover\": \"$.coverUrl\",\n    \"summary\": \"$.intro\",\n    \"detail\": \"http://api.yuedu.best/yuedu/getBookInfo@post->{{\\\"searchBook\\\":${{$}}, \\\"bookSourceCode\\\":\\\"{md5}\\\"}}\"\n  }},\n  \"detail\": {{\n    \"name\": \"$.name\",\n    \"author\": \"$.author\",\n    \"cover\": \"$.coverUrl\",\n    \"summary\": \"$.intro\",\n    \"status\": \"\",\n    \"update\": \"$.latestChapterTime\",\n    \"lastChapter\": \"$.latestChapterTitle\",\n    \"catalog\": \"http://api.yuedu.best/yuedu/getChapterList@post->{{\\\"book\\\":${{$}}, \\\"bookSourceCode\\\":\\\"{md5}\\\"}}\"\n  }},\n  \"catalog\": {{\n    \"list\": \"$.[*]\",\n    \"name\": \"$.title\",\n    \"chapter\": \"http://api.yuedu.best/yuedu/getContent@post->{{\\\"bookChapter\\\":${{$}}, \\\"bookSourceCode\\\":\\\"{md5}\\\"}}\"\n  }},\n  \"chapter\": {{\n    \"content\": \"$.text\"\n  }}\n}}"
        );

        let file = File::new(format!("repo/{}.json", url.replace("https://", "").replace("http://", "")));
        println!("file path: " + file.absoluteFile);
        file.createNewFile();
        file.writeText(text);
//        println("file path: " + file.absoluteFile);
    }
}
