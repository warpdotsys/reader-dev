use crate::prelude::*;
// package com.htmake.reader.utils;

// import org.springframework.beans.BeansException;
// import org.springframework.context.ApplicationContext;
// import org.springframework.context.ApplicationContextAware;
// import org.springframework.stereotype.Component;

// @Component
// public class SpringContextUtils implements ApplicationContextAware {
pub struct SpringContextUtils;

// fix: Rust 不允许 impl 块内的关联 static，移到模块级（impl 内引用不变）
/**
 * 上下文对象实例
 */
// private static ApplicationContext applicationContext;
pub static APPLICATION_CONTEXT: std::sync::OnceLock<Option<ApplicationContext>> = std::sync::OnceLock::new();

impl SpringContextUtils {
    // @Override
    // public void setApplicationContext(ApplicationContext context) throws BeansException {
    //     applicationContext = context;
    // }
    pub fn set_application_context(context: ApplicationContext) {
        // fix: OnceLock 仅支持首次写入，Java 的重复赋值语义降级为首次生效
        let _ = APPLICATION_CONTEXT.set(Some(context));
    }

    /**
     * 获取applicationContext
     *
     * @return
     */
    // public static ApplicationContext getApplicationContext() {
    //     return applicationContext;
    // }
    pub fn get_application_context() -> Option<ApplicationContext> {
        return APPLICATION_CONTEXT.get_or_init(|| None).clone();
    }

    /**
     * 通过name获取 Bean.
     *
     * @param name
     * @return
     */
    // public static Object getBean(String name) {
    //     if (applicationContext != None) {
    //         return getApplicationContext().getBean(name);
    //     }
    //     return None;
    // }
    pub fn get_bean_by_name(name: &str) -> Option<Object> {
        if APPLICATION_CONTEXT.get_or_init(|| None).is_some() {
            return Self::get_application_context().unwrap().get_bean_by_name(name);
        }
        return None;
    }

    /**
     * 通过class获取Bean.
     *
     * @param clazz
     * @param <T>
     * @return
     */
    // public static <T> T getBean(Class<T> clazz) {
    //     if (applicationContext != None) {
    //         return getApplicationContext().getBean(clazz);
    //     }
    //     return None;
    // }
    pub fn get_bean_by_class<T>(clazz: Class<T>) -> Option<T> {
        if APPLICATION_CONTEXT.get_or_init(|| None).is_some() {
            return Self::get_application_context().unwrap().get_bean_by_class(clazz);
        }
        return None;
    }

    /**
     * 通过name,以及Clazz返回指定的Bean
     *
     * @param name
     * @param clazz
     * @param <T>
     * @return
     */
    // public static <T> T getBean(String name, Class<T> clazz) {
    //     if (applicationContext != None) {
    //         return getApplicationContext().getBean(name, clazz);
    //     }
    //     return None;
    // }
    pub fn get_bean_by_name_and_class<T>(name: &str, clazz: Class<T>) -> Option<T> {
        if APPLICATION_CONTEXT.get_or_init(|| None).is_some() {
            return Self::get_application_context().unwrap().get_bean_by_name_and_class(name, clazz);
        }
        return None;
    }
}
