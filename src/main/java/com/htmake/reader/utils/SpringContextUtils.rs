// package com.htmake.reader.utils;

// import org.springframework.beans.BeansException;
// import org.springframework.context.ApplicationContext;
// import org.springframework.context.ApplicationContextAware;
// import org.springframework.stereotype.Component;

// @Component
// public class SpringContextUtils implements ApplicationContextAware {
pub struct SpringContextUtils;

impl SpringContextUtils {
    /**
     * 上下文对象实例
     */
    // private static ApplicationContext applicationContext;
    static APPLICATION_CONTEXT: std::sync::OnceLock<Option<ApplicationContext>> = std::sync::OnceLock::new();

    // @Override
    // public void setApplicationContext(ApplicationContext context) throws BeansException {
    //     applicationContext = context;
    // }
    pub fn set_application_context(context: ApplicationContext) {
        *APPLICATION_CONTEXT.get_or_init(|| None) = Some(context);
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
    //     if (applicationContext != null) {
    //         return getApplicationContext().getBean(name);
    //     }
    //     return null;
    // }
    pub fn get_bean_by_name(name: &str) -> Option<Object> {
        if APPLICATION_CONTEXT.get_or_init(|| None).is_some() {
            return Self::get_application_context().unwrap().get_bean(name);
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
    //     if (applicationContext != null) {
    //         return getApplicationContext().getBean(clazz);
    //     }
    //     return null;
    // }
    pub fn get_bean_by_class<T>(clazz: Class<T>) -> Option<T> {
        if APPLICATION_CONTEXT.get_or_init(|| None).is_some() {
            return Self::get_application_context().unwrap().get_bean(clazz);
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
    //     if (applicationContext != null) {
    //         return getApplicationContext().getBean(name, clazz);
    //     }
    //     return null;
    // }
    pub fn get_bean_by_name_and_class<T>(name: &str, clazz: Class<T>) -> Option<T> {
        if APPLICATION_CONTEXT.get_or_init(|| None).is_some() {
            return Self::get_application_context().unwrap().get_bean(name, clazz);
        }
        return None;
    }
}
