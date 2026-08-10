// package com.htmake.reader.lib.tts.constant;

// public enum VoiceEnum {
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VoiceEnum {
    zh_HK_HiuGaaiNeural,
    zh_HK_HiuMaanNeural,
    zh_HK_WanLungNeural,
    zh_CN_XiaoxiaoNeural,
    zh_CN_XiaoyiNeural,
    zh_CN_YunjianNeural,
    zh_CN_YunxiNeural,
    zh_CN_YunxiaNeural,
    zh_CN_YunyangNeural,
    zh_CN_liaoning_XiaobeiNeural,
    zh_TW_HsiaoChenNeural,
    zh_TW_YunJheNeural,
    zh_TW_HsiaoYuNeural,
    zh_CN_shaanxi_XiaoniNeural,
    en_US_AriaNeural,
    en_US_AnaNeural,
    en_US_ChristopherNeural,
    en_US_EricNeural,
    en_US_GuyNeural,
    en_US_JennyNeural,
    en_US_MichelleNeural,
    en_US_RogerNeural,
    en_US_SteffanNeural,
    zh_CN_XiaochenNeural,
    zh_CN_XiaohanNeural,
    zh_CN_XiaomengNeural,
    zh_CN_XiaomoNeural,
    zh_CN_XiaoqiuNeural,
    zh_CN_XiaoruiNeural,
    zh_CN_XiaoshuangNeural,
    zh_CN_XiaoxuanNeural,
    zh_CN_XiaoyanNeural,
    zh_CN_XiaoyouNeural,
    zh_CN_XiaozhenNeural,
    zh_CN_YunfengNeural,
    zh_CN_YunhaoNeural,
    zh_CN_YunyeNeural,
    zh_CN_YunzeNeural,
}

impl VoiceEnum {
    // public String getShortName() {
    //     return shortName;
    // }
    pub fn get_short_name(&self) -> &'static str {
        match self {
            VoiceEnum::zh_HK_HiuGaaiNeural => "zh-HK-HiuGaaiNeural",
            VoiceEnum::zh_HK_HiuMaanNeural => "zh-HK-HiuMaanNeural",
            VoiceEnum::zh_HK_WanLungNeural => "zh-HK-WanLungNeural",
            VoiceEnum::zh_CN_XiaoxiaoNeural => "zh-CN-XiaoxiaoNeural",
            VoiceEnum::zh_CN_XiaoyiNeural => "zh-CN-XiaoyiNeural",
            VoiceEnum::zh_CN_YunjianNeural => "zh-CN-YunjianNeural",
            VoiceEnum::zh_CN_YunxiNeural => "zh-CN-YunxiNeural",
            VoiceEnum::zh_CN_YunxiaNeural => "zh-CN-YunxiaNeural",
            VoiceEnum::zh_CN_YunyangNeural => "zh-CN-YunyangNeural",
            VoiceEnum::zh_CN_liaoning_XiaobeiNeural => "zh-CN-liaoning-XiaobeiNeural",
            VoiceEnum::zh_TW_HsiaoChenNeural => "zh-TW-HsiaoChenNeural",
            VoiceEnum::zh_TW_YunJheNeural => "zh-TW-YunJheNeural",
            VoiceEnum::zh_TW_HsiaoYuNeural => "zh-TW-HsiaoYuNeural",
            VoiceEnum::zh_CN_shaanxi_XiaoniNeural => "zh-CN-shaanxi-XiaoniNeural",
            VoiceEnum::en_US_AriaNeural => "en-US-AriaNeural",
            VoiceEnum::en_US_AnaNeural => "en-US-AnaNeural",
            VoiceEnum::en_US_ChristopherNeural => "en-US-ChristopherNeural",
            VoiceEnum::en_US_EricNeural => "en-US-EricNeural",
            VoiceEnum::en_US_GuyNeural => "en-US-GuyNeural",
            VoiceEnum::en_US_JennyNeural => "en-US-JennyNeural",
            VoiceEnum::en_US_MichelleNeural => "en-US-MichelleNeural",
            VoiceEnum::en_US_RogerNeural => "en-US-RogerNeural",
            VoiceEnum::en_US_SteffanNeural => "en-US-SteffanNeural",
            VoiceEnum::zh_CN_XiaochenNeural => "zh-CN-XiaochenNeural",
            VoiceEnum::zh_CN_XiaohanNeural => "zh-CN-XiaohanNeural",
            VoiceEnum::zh_CN_XiaomengNeural => "zh-CN-XiaomengNeural",
            VoiceEnum::zh_CN_XiaomoNeural => "zh-CN-XiaomoNeural",
            VoiceEnum::zh_CN_XiaoqiuNeural => "zh-CN-XiaoqiuNeural",
            VoiceEnum::zh_CN_XiaoruiNeural => "zh-CN-XiaoruiNeural",
            VoiceEnum::zh_CN_XiaoshuangNeural => "zh-CN-XiaoshuangNeural",
            VoiceEnum::zh_CN_XiaoxuanNeural => "zh-CN-XiaoxuanNeural",
            VoiceEnum::zh_CN_XiaoyanNeural => "zh-CN-XiaoyanNeural",
            VoiceEnum::zh_CN_XiaoyouNeural => "zh-CN-XiaoyouNeural",
            VoiceEnum::zh_CN_XiaozhenNeural => "zh-CN-XiaozhenNeural",
            VoiceEnum::zh_CN_YunfengNeural => "zh-CN-YunfengNeural",
            VoiceEnum::zh_CN_YunhaoNeural => "zh-CN-YunhaoNeural",
            VoiceEnum::zh_CN_YunyeNeural => "zh-CN-YunyeNeural",
            VoiceEnum::zh_CN_YunzeNeural => "zh-CN-YunzeNeural",
        }
    }

    // public String getGender() {
    //     return gender;
    // }
    pub fn get_gender(&self) -> &'static str {
        match self {
            VoiceEnum::zh_HK_HiuGaaiNeural => "女",
            VoiceEnum::zh_HK_HiuMaanNeural => "女",
            VoiceEnum::zh_HK_WanLungNeural => "男",
            VoiceEnum::zh_CN_XiaoxiaoNeural => "女",
            VoiceEnum::zh_CN_XiaoyiNeural => "女",
            VoiceEnum::zh_CN_YunjianNeural => "男",
            VoiceEnum::zh_CN_YunxiNeural => "男",
            VoiceEnum::zh_CN_YunxiaNeural => "男",
            VoiceEnum::zh_CN_YunyangNeural => "男",
            VoiceEnum::zh_CN_liaoning_XiaobeiNeural => "女",
            VoiceEnum::zh_TW_HsiaoChenNeural => "女",
            VoiceEnum::zh_TW_YunJheNeural => "男",
            VoiceEnum::zh_TW_HsiaoYuNeural => "女",
            VoiceEnum::zh_CN_shaanxi_XiaoniNeural => "女",
            VoiceEnum::en_US_AriaNeural => "女",
            VoiceEnum::en_US_AnaNeural => "女",
            VoiceEnum::en_US_ChristopherNeural => "男",
            VoiceEnum::en_US_EricNeural => "男",
            VoiceEnum::en_US_GuyNeural => "男",
            VoiceEnum::en_US_JennyNeural => "女",
            VoiceEnum::en_US_MichelleNeural => "女",
            VoiceEnum::en_US_RogerNeural => "男",
            VoiceEnum::en_US_SteffanNeural => "男",
            VoiceEnum::zh_CN_XiaochenNeural => "女",
            VoiceEnum::zh_CN_XiaohanNeural => "女",
            VoiceEnum::zh_CN_XiaomengNeural => "女",
            VoiceEnum::zh_CN_XiaomoNeural => "女",
            VoiceEnum::zh_CN_XiaoqiuNeural => "女",
            VoiceEnum::zh_CN_XiaoruiNeural => "女",
            VoiceEnum::zh_CN_XiaoshuangNeural => "女",
            VoiceEnum::zh_CN_XiaoxuanNeural => "女",
            VoiceEnum::zh_CN_XiaoyanNeural => "女",
            VoiceEnum::zh_CN_XiaoyouNeural => "女",
            VoiceEnum::zh_CN_XiaozhenNeural => "女",
            VoiceEnum::zh_CN_YunfengNeural => "男",
            VoiceEnum::zh_CN_YunhaoNeural => "男",
            VoiceEnum::zh_CN_YunyeNeural => "男",
            VoiceEnum::zh_CN_YunzeNeural => "男",
        }
    }

    // public String getLocale() {
    //     return locale;
    // }
    pub fn get_locale(&self) -> &'static str {
        match self {
            VoiceEnum::zh_HK_HiuGaaiNeural => "zh-HK",
            VoiceEnum::zh_HK_HiuMaanNeural => "zh-HK",
            VoiceEnum::zh_HK_WanLungNeural => "zh-HK",
            VoiceEnum::zh_CN_XiaoxiaoNeural => "zh-CN",
            VoiceEnum::zh_CN_XiaoyiNeural => "zh-CN",
            VoiceEnum::zh_CN_YunjianNeural => "zh-CN",
            VoiceEnum::zh_CN_YunxiNeural => "zh-CN",
            VoiceEnum::zh_CN_YunxiaNeural => "zh-CN",
            VoiceEnum::zh_CN_YunyangNeural => "zh-CN",
            VoiceEnum::zh_CN_liaoning_XiaobeiNeural => "zh-CN-liaoning",
            VoiceEnum::zh_TW_HsiaoChenNeural => "zh-TW",
            VoiceEnum::zh_TW_YunJheNeural => "zh-TW",
            VoiceEnum::zh_TW_HsiaoYuNeural => "zh-TW",
            VoiceEnum::zh_CN_shaanxi_XiaoniNeural => "zh-CN-shaanxi",
            VoiceEnum::en_US_AriaNeural => "en-US",
            VoiceEnum::en_US_AnaNeural => "en-US",
            VoiceEnum::en_US_ChristopherNeural => "en-US",
            VoiceEnum::en_US_EricNeural => "en-US",
            VoiceEnum::en_US_GuyNeural => "en-US",
            VoiceEnum::en_US_JennyNeural => "en-US",
            VoiceEnum::en_US_MichelleNeural => "en-US",
            VoiceEnum::en_US_RogerNeural => "en-US",
            VoiceEnum::en_US_SteffanNeural => "en-US",
            VoiceEnum::zh_CN_XiaochenNeural => "zh-CN",
            VoiceEnum::zh_CN_XiaohanNeural => "zh-CN",
            VoiceEnum::zh_CN_XiaomengNeural => "zh-CN",
            VoiceEnum::zh_CN_XiaomoNeural => "zh-CN",
            VoiceEnum::zh_CN_XiaoqiuNeural => "zh-CN",
            VoiceEnum::zh_CN_XiaoruiNeural => "zh-CN",
            VoiceEnum::zh_CN_XiaoshuangNeural => "zh-CN",
            VoiceEnum::zh_CN_XiaoxuanNeural => "zh-CN",
            VoiceEnum::zh_CN_XiaoyanNeural => "zh-CN",
            VoiceEnum::zh_CN_XiaoyouNeural => "zh-CN",
            VoiceEnum::zh_CN_XiaozhenNeural => "zh-CN",
            VoiceEnum::zh_CN_YunfengNeural => "zh-CN",
            VoiceEnum::zh_CN_YunhaoNeural => "zh-CN",
            VoiceEnum::zh_CN_YunyeNeural => "zh-CN",
            VoiceEnum::zh_CN_YunzeNeural => "zh-CN",
        }
    }

    // public static VoiceEnum fromSortName(String shortName) {
    //     for (VoiceEnum voice : values()) {
    //         if (voice.getShortName().equals(shortName)) {
    //             return voice;
    //         }
    //     }
    //     return null;
    // }
    pub fn from_sort_name(short_name: &str) -> Option<VoiceEnum> {
        const ALL: &[VoiceEnum] = &[
            VoiceEnum::zh_HK_HiuGaaiNeural,
            VoiceEnum::zh_HK_HiuMaanNeural,
            VoiceEnum::zh_HK_WanLungNeural,
            VoiceEnum::zh_CN_XiaoxiaoNeural,
            VoiceEnum::zh_CN_XiaoyiNeural,
            VoiceEnum::zh_CN_YunjianNeural,
            VoiceEnum::zh_CN_YunxiNeural,
            VoiceEnum::zh_CN_YunxiaNeural,
            VoiceEnum::zh_CN_YunyangNeural,
            VoiceEnum::zh_CN_liaoning_XiaobeiNeural,
            VoiceEnum::zh_TW_HsiaoChenNeural,
            VoiceEnum::zh_TW_YunJheNeural,
            VoiceEnum::zh_TW_HsiaoYuNeural,
            VoiceEnum::zh_CN_shaanxi_XiaoniNeural,
            VoiceEnum::en_US_AriaNeural,
            VoiceEnum::en_US_AnaNeural,
            VoiceEnum::en_US_ChristopherNeural,
            VoiceEnum::en_US_EricNeural,
            VoiceEnum::en_US_GuyNeural,
            VoiceEnum::en_US_JennyNeural,
            VoiceEnum::en_US_MichelleNeural,
            VoiceEnum::en_US_RogerNeural,
            VoiceEnum::en_US_SteffanNeural,
            VoiceEnum::zh_CN_XiaochenNeural,
            VoiceEnum::zh_CN_XiaohanNeural,
            VoiceEnum::zh_CN_XiaomengNeural,
            VoiceEnum::zh_CN_XiaomoNeural,
            VoiceEnum::zh_CN_XiaoqiuNeural,
            VoiceEnum::zh_CN_XiaoruiNeural,
            VoiceEnum::zh_CN_XiaoshuangNeural,
            VoiceEnum::zh_CN_XiaoxuanNeural,
            VoiceEnum::zh_CN_XiaoyanNeural,
            VoiceEnum::zh_CN_XiaoyouNeural,
            VoiceEnum::zh_CN_XiaozhenNeural,
            VoiceEnum::zh_CN_YunfengNeural,
            VoiceEnum::zh_CN_YunhaoNeural,
            VoiceEnum::zh_CN_YunyeNeural,
            VoiceEnum::zh_CN_YunzeNeural,
        ];
        for voice in ALL {
            if voice.get_short_name() == short_name {
                return Some(*voice);
            }
        }
        return None;
    }
}
