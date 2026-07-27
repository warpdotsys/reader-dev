package com.htmake.reader.lib.tts.constant;

public enum VoiceEnum {

    zh_HK_HiuGaaiNeural("zh-HK-HiuGaaiNeural", "\u5973", "zh-HK"),
    zh_HK_HiuMaanNeural("zh-HK-HiuMaanNeural", "\u5973", "zh-HK"),
    zh_HK_WanLungNeural("zh-HK-WanLungNeural", "\u7537", "zh-HK"),
    zh_CN_XiaoxiaoNeural("zh-CN-XiaoxiaoNeural", "\u5973", "zh-CN"),
    zh_CN_XiaoyiNeural("zh-CN-XiaoyiNeural", "\u5973", "zh-CN"),
    zh_CN_YunjianNeural("zh-CN-YunjianNeural", "\u7537", "zh-CN"),
    zh_CN_YunxiNeural("zh-CN-YunxiNeural", "\u7537", "zh-CN"),
    zh_CN_YunxiaNeural("zh-CN-YunxiaNeural", "\u7537", "zh-CN"),
    zh_CN_YunyangNeural("zh-CN-YunyangNeural", "\u7537", "zh-CN"),
    zh_CN_liaoning_XiaobeiNeural("zh-CN-liaoning-XiaobeiNeural", "\u5973", "zh-CN-liaoning"),
    zh_TW_HsiaoChenNeural("zh-TW-HsiaoChenNeural", "\u5973", "zh-TW"),
    zh_TW_YunJheNeural("zh-TW-YunJheNeural", "\u7537", "zh-TW"),
    zh_TW_HsiaoYuNeural("zh-TW-HsiaoYuNeural", "\u5973", "zh-TW"),
    zh_CN_shaanxi_XiaoniNeural("zh-CN-shaanxi-XiaoniNeural", "\u5973", "zh-CN-shaanxi"),
    en_US_AriaNeural("en-US-AriaNeural", "\u5973", "en-US"),
    en_US_AnaNeural("en-US-AnaNeural", "\u5973", "en-US"),
    en_US_ChristopherNeural("en-US-ChristopherNeural", "\u7537", "en-US"),
    en_US_EricNeural("en-US-EricNeural", "\u7537", "en-US"),
    en_US_GuyNeural("en-US-GuyNeural", "\u7537", "en-US"),
    en_US_JennyNeural("en-US-JennyNeural", "\u5973", "en-US"),
    en_US_MichelleNeural("en-US-MichelleNeural", "\u5973", "en-US"),
    en_US_RogerNeural("en-US-RogerNeural", "\u7537", "en-US"),
    en_US_SteffanNeural("en-US-SteffanNeural", "\u7537", "en-US"),
    zh_CN_XiaochenNeural("zh-CN-XiaochenNeural", "\u5973", "zh-CN"),
    zh_CN_XiaohanNeural("zh-CN-XiaohanNeural", "\u5973", "zh-CN"),
    zh_CN_XiaomengNeural("zh-CN-XiaomengNeural", "\u5973", "zh-CN"),
    zh_CN_XiaomoNeural("zh-CN-XiaomoNeural", "\u5973", "zh-CN"),
    zh_CN_XiaoqiuNeural("zh-CN-XiaoqiuNeural", "\u5973", "zh-CN"),
    zh_CN_XiaoruiNeural("zh-CN-XiaoruiNeural", "\u5973", "zh-CN"),
    zh_CN_XiaoshuangNeural("zh-CN-XiaoshuangNeural", "\u5973", "zh-CN"),
    zh_CN_XiaoxuanNeural("zh-CN-XiaoxuanNeural", "\u5973", "zh-CN"),
    zh_CN_XiaoyanNeural("zh-CN-XiaoyanNeural", "\u5973", "zh-CN"),
    zh_CN_XiaoyouNeural("zh-CN-XiaoyouNeural", "\u5973", "zh-CN"),
    zh_CN_XiaozhenNeural("zh-CN-XiaozhenNeural", "\u5973", "zh-CN"),
    zh_CN_YunfengNeural("zh-CN-YunfengNeural", "\u7537", "zh-CN"),
    zh_CN_YunhaoNeural("zh-CN-YunhaoNeural", "\u7537", "zh-CN"),
    zh_CN_YunyeNeural("zh-CN-YunyeNeural", "\u7537", "zh-CN"),
    zh_CN_YunzeNeural("zh-CN-YunzeNeural", "\u7537", "zh-CN");

    private final String shortName;
    private final String gender;
    private final String locale;

    VoiceEnum(String shortName, String gender, String locale) {
        this.shortName = shortName;
        this.gender = gender;
        this.locale = locale;
    }

    public String getShortName() {
        return shortName;
    }

    public String getGender() {
        return gender;
    }

    public String getLocale() {
        return locale;
    }

    public static VoiceEnum fromSortName(String shortName) {
        for (VoiceEnum voice : values()) {
            if (voice.getShortName().equals(shortName)) {
                return voice;
            }
        }
        return null;
    }
}
