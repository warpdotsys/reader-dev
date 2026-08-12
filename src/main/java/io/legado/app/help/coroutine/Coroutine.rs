use crate::prelude::*;
use std::future::Future;
use std::pin::Pin;
// package io.legado.app.help.coroutine
//
// import kotlinx.coroutines.*
// import kotlin.coroutines.CoroutineContext

// fix: stubs::Block/VoidBlock 为无参单位结构体占位；此处局部定义可调用版（遮蔽 prelude glob 导入）
pub type Block<T> = Box<dyn Fn(&CoroutineScope) -> Pin<Box<dyn Future<Output = T> + '_>>>;
pub type VoidBlock = Box<dyn Fn(&CoroutineScope) -> Pin<Box<dyn Future<Output = ()> + '_>>>;
pub type CallbackBlock<V> = Box<dyn for<'a> Fn(&'a CoroutineScope, &'a V) -> Pin<Box<dyn Future<Output = ()> + 'a>>>;

// fix: 局部定义（遮蔽 stubs::with_context 同步占位）以支持 async 闭包 + .await
async fn with_context<T: Future, F: FnOnce() -> T>(_ctx: CoroutineContext, f: F) -> T::Output {
    f().await
}

/**
 * class Coroutine<T>(
 *     val scope: CoroutineScope,
 *     context: CoroutineContext = Dispatchers.IO,
 *     block: suspend CoroutineScope.() -> T
 * )
 */
pub struct Coroutine<T> {
    pub scope: CoroutineScope,
    context: CoroutineContext,
    job: Job,
    start: Option<VoidCallback>,
    success: Option<Callback<T>>,
    error: Option<Callback<Throwable>>,
    finally: Option<VoidCallback>,
    cancel: Option<VoidCallback>,
    time_millis: Option<i64>,
    error_return: Option<Result_<T>>,
}

impl<T> Coroutine<T> {
    // companion object {
    //
    //     private val DEFAULT = MainScope()
    //
    //     fun <T> async(
    //         scope: CoroutineScope = DEFAULT,
    //         context: CoroutineContext = Dispatchers.IO,
    //         block: suspend CoroutineScope.() -> T
    //     ): Coroutine<T> {
    //         return Coroutine(scope, context, block)
    //     }
    //
    // }
    // fix: `async` 为 Rust 关键字 → r#async 转义
    pub fn r#async(
        scope: CoroutineScope,
        context: CoroutineContext,
        block: Block<T>,
    ) -> Coroutine<T> {
        // return Coroutine(scope, context, block)
        Coroutine::new(scope, context, block)
    }

    pub fn new(scope: CoroutineScope, context: CoroutineContext, block: Block<T>) -> Coroutine<T> {
        // init {
        //     this.job = executeInternal(context, block)
        // }
        let mut coroutine = Coroutine {
            scope,
            context,
            job: Job::default(),
            start: None,
            success: None,
            error: None,
            finally: None,
            cancel: None,
            time_millis: None,
            error_return: None,
        };
        // fix: clone 上下文以拆分 self 借用（&mut self 方法调用 + &self.context 无法并存）
        let context = coroutine.context.clone();
        coroutine.job = coroutine.execute_internal(&context, block);
        coroutine
    }

    // val isCancelled: Boolean
    //     get() = job.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.job.is_cancelled()
    }

    // val isActive: Boolean
    //     get() = job.isActive
    pub fn is_active(&self) -> bool {
        self.job.is_active()
    }

    // val isCompleted: Boolean
    //     get() = job.isCompleted
    pub fn is_completed(&self) -> bool {
        self.job.is_completed()
    }

    // fun timeout(timeMillis: () -> Long): Coroutine<T> {
    //     this.timeMillis = timeMillis()
    //     return this@Coroutine
    // }
    pub fn timeout(mut self, time_millis: impl FnOnce() -> i64) -> Coroutine<T> {
        self.time_millis = Some(time_millis());
        self
    }

    // fun timeout(timeMillis: Long): Coroutine<T> {
    //     this.timeMillis = timeMillis
    //     return this@Coroutine
    // }
    pub fn timeout_value(mut self, time_millis: i64) -> Coroutine<T> {
        self.time_millis = Some(time_millis);
        self
    }

    // fun onErrorReturn(value: () -> T?): Coroutine<T> {
    //     this.errorReturn = Result(value())
    //     return this@Coroutine
    // }
    pub fn on_error_return(mut self, value: impl FnOnce() -> Option<T>) -> Coroutine<T> {
        self.error_return = Some(Result_ { value: value() });
        self
    }

    // fun onErrorReturn(value: T?): Coroutine<T> {
    //     this.errorReturn = Result(value)
    //     return this@Coroutine
    // }
    pub fn on_error_return_value(mut self, value: Option<T>) -> Coroutine<T> {
        self.error_return = Some(Result_ { value });
        self
    }

    // fun onStart(
    //     context: CoroutineContext? = None,
    //     block: (suspend CoroutineScope.() -> Unit)
    // ): Coroutine<T> {
    //     this.start = VoidCallback(context, block)
    //     return this@Coroutine
    // }
    pub fn on_start(mut self, context: Option<CoroutineContext>, block: VoidBlock) -> Coroutine<T> {
        self.start = Some(VoidCallback { context, block });
        self
    }

    // fun onSuccess(
    //     context: CoroutineContext? = None,
    //     block: suspend CoroutineScope.(T) -> Unit
    // ): Coroutine<T> {
    //     this.success = Callback(context, block)
    //     return this@Coroutine
    // }
    pub fn on_success(mut self, context: Option<CoroutineContext>, block: CallbackBlock<T>) -> Coroutine<T> {
        self.success = Some(Callback { context, block });
        self
    }

    // fun onError(
    //     context: CoroutineContext? = None,
    //     block: suspend CoroutineScope.(Throwable) -> Unit
    // ): Coroutine<T> {
    //     this.error = Callback(context, block)
    //     return this@Coroutine
    // }
    pub fn on_error(mut self, context: Option<CoroutineContext>, block: CallbackBlock<Throwable>) -> Coroutine<T> {
        self.error = Some(Callback { context, block });
        self
    }

    // fun onFinally(
    //     context: CoroutineContext? = None,
    //     block: suspend CoroutineScope.() -> Unit
    // ): Coroutine<T> {
    //     this.finally = VoidCallback(context, block)
    //     return this@Coroutine
    // }
    pub fn on_finally(mut self, context: Option<CoroutineContext>, block: VoidBlock) -> Coroutine<T> {
        self.finally = Some(VoidCallback { context, block });
        self
    }

    // fun onCancel(
    //     context: CoroutineContext? = None,
    //     block: suspend CoroutineScope.() -> Unit
    // ): Coroutine<T> {
    //     this.cancel = VoidCallback(context, block)
    //     return this@Coroutine
    // }
    pub fn on_cancel(mut self, context: Option<CoroutineContext>, block: VoidBlock) -> Coroutine<T> {
        self.cancel = Some(VoidCallback { context, block });
        self
    }

    //取消当前任务
    // fun cancel(cause: CancellationException? = None) {
    //     job.cancel(cause)
    //     cancel?.let {
    //         MainScope().launch {
    //             if (None == it.context) {
    //                 it.block.invoke(scope)
    //             } else {
    //                 withContext(scope.coroutineContext.plus(it.context)) {
    //                     it.block.invoke(this)
    //                 }
    //             }
    //         }
    //     }
    // }
    pub fn cancel(&self, cause: Option<CancellationException>) {
        // fix: stubs::Job::cancel 无参占位——Kotlin job.cancel(cause) 的 cause 丢弃
        self.job.cancel();
        if let Some(it) = &self.cancel {
            MainScope::new().launch(|| async move {
                if it.context.is_none() {
                    (it.block)(&self.scope).await;
                } else {
                    with_context(self.scope.coroutine_context().plus(it.context.as_ref().unwrap()), || async {
                        (it.block)(&self.scope).await;
                    }).await;
                }
            });
        }
    }

    // fun invokeOnCompletion(handler: CompletionHandler): DisposableHandle {
    //     return job.invokeOnCompletion(handler)
    // }
    pub fn invoke_on_completion(&self, handler: CompletionHandler) -> DisposableHandle {
        self.job.invoke_on_completion(handler)
    }

    // private fun executeInternal(
    //     context: CoroutineContext,
    //     block: suspend CoroutineScope.() -> T
    // ): Job {
    //     return scope.plus(Dispatchers.IO).launch {
    //         try {
    //             start?.let { dispatchVoidCallback(this, it) }
    //             val value = executeBlock(scope, context, timeMillis ?: 0L, block)
    //             if (isActive) {
    //                 success?.let { dispatchCallback(this, value, it) }
    //             }
    //         } catch (e: Throwable) {
    //             e.printStackTrace()
    //             val consume: Boolean = errorReturn?.value?.let { value ->
    //                 if (isActive) {
    //                     success?.let { dispatchCallback(this, value, it) }
    //                 }
    //                 true
    //             } ?: false
    //
    //             if (!consume && isActive) {
    //                 error?.let { dispatchCallback(this, e, it) }
    //             }
    //         } finally {
    //             if (isActive) {
    //                 finally?.let { dispatchVoidCallback(this, it) }
    //             }
    //         }
    //     }
    // }
    fn execute_internal(&mut self, context: &CoroutineContext, block: Block<T>) -> Job {
        // return scope.plus(Dispatchers.IO).launch { ... }
        let job = Job::default();
        job.launch(|| async move {
            // try {
            let result = (async {
                if let Some(start) = &self.start {
                    Self::dispatch_void_callback(&self.scope, start).await;
                }
                let value = Self::execute_block(&self.scope, context, self.time_millis.unwrap_or(0_i64), &block).await;
                if self.is_active() {
                    if let Some(success) = &self.success {
                        Self::dispatch_callback(&self.scope, &value, success).await;
                    }
                }
                Ok::<(), Throwable>(())
            }).await;
            // } catch (e: Throwable) {
            if let Err(e) = result {
                e.print_stack_trace();
                // val consume: Boolean = errorReturn?.value?.let { value ->
                //     if (isActive) {
                //         success?.let { dispatchCallback(this, value, it) }
                //     }
                //     true
                // } ?: false
                let consume: bool = match &self.error_return {
                    Some(error_return) => {
                        // fix: Kotlin errorReturn?.value?.let { value -> ... true } ?: false
                        let value = error_return.value.as_ref();
                        if let Some(value) = value {
                            if self.is_active() {
                                if let Some(success) = &self.success {
                                    Self::dispatch_callback(&self.scope, value, success).await;
                                }
                            }
                            true
                        } else {
                            false
                        }
                    }
                    None => false,
                };
                // if (!consume && isActive) {
                //     error?.let { dispatchCallback(this, e, it) }
                // }
                if !consume && self.is_active() {
                    if let Some(error) = &self.error {
                        Self::dispatch_callback(&self.scope, &e, error).await;
                    }
                }
            }
            // } finally {
            //     if (isActive) {
            //         finally?.let { dispatchVoidCallback(this, it) }
            //     }
            // }
            if self.is_active() {
                if let Some(finally) = &self.finally {
                    Self::dispatch_void_callback(&self.scope, finally).await;
                }
            }
        });
        job
    }

    // private suspend inline fun dispatchVoidCallback(scope: CoroutineScope, callback: VoidCallback) {
    //     if (None == callback.context) {
    //         callback.block.invoke(scope)
    //     } else {
    //         withContext(scope.coroutineContext.plus(callback.context)) {
    //             callback.block.invoke(this)
    //         }
    //     }
    // }
    async fn dispatch_void_callback(scope: &CoroutineScope, callback: &VoidCallback) {
        if callback.context.is_none() {
            (callback.block)(scope).await;
        } else {
            with_context(scope.coroutine_context().plus(callback.context.as_ref().unwrap()), || async {
                (callback.block)(scope).await;
            }).await;
        }
    }

    // private suspend inline fun <R> dispatchCallback(
    //     scope: CoroutineScope,
    //     value: R,
    //     callback: Callback<R>
    // ) {
    //     if (!scope.isActive) return
    //     if (None == callback.context) {
    //         callback.block.invoke(scope, value)
    //     } else {
    //         withContext(scope.coroutineContext.plus(callback.context)) {
    //             callback.block.invoke(this, value)
    //         }
    //     }
    // }
    async fn dispatch_callback<R>(
        scope: &CoroutineScope,
        value: &R,
        callback: &Callback<R>,
    ) {
        if !scope.is_active() {
            return;
        }
        if callback.context.is_none() {
            (callback.block)(scope, value).await;
        } else {
            with_context(scope.coroutine_context().plus(callback.context.as_ref().unwrap()), || async {
                (callback.block)(scope, value).await;
            }).await;
        }
    }

    // private suspend inline fun executeBlock(
    //     scope: CoroutineScope,
    //     context: CoroutineContext,
    //     timeMillis: Long,
    //     noinline block: suspend CoroutineScope.() -> T
    // ): T {
    //     return withContext(scope.coroutineContext.plus(context)) {
    //         if (timeMillis > 0L) withTimeout(timeMillis) {
    //             block()
    //         } else {
    //             block()
    //         }
    //     }
    // }
    async fn execute_block(
        scope: &CoroutineScope,
        context: &CoroutineContext,
        time_millis: i64,
        block: &Block<T>,
    ) -> T {
        with_context(scope.coroutine_context().plus(context), || async {
            if time_millis > 0_i64 {
                with_timeout(time_millis, || async { (block)(scope).await }).await
            } else {
                (block)(scope).await
            }
        }).await
    }
}

// private data class Result<out T>(val value: T?)
pub struct Result_<T> {
    value: Option<T>,
}

// private inner class VoidCallback(
//     val context: CoroutineContext?,
//     val block: suspend CoroutineScope.() -> Unit
// )
pub struct VoidCallback {
    context: Option<CoroutineContext>,
    block: VoidBlock,
}

// private inner class Callback<VALUE>(
//     val context: CoroutineContext?,
//     val block: suspend CoroutineScope.(VALUE) -> Unit
// )
pub struct Callback<VALUE> {
    context: Option<CoroutineContext>,
    block: CallbackBlock<VALUE>,
}

// fix: CompositeCoroutine 将 Rc<Coroutine<Box<dyn Any>>> 放入 HashSet，需 Eq+Hash；
// 字段含函数指针无法 derive，按指针身份比较（与 Kotlin HashSet<Coroutine<*>> 按引用去重语义一致）
impl<T> PartialEq for Coroutine<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
impl<T> Eq for Coroutine<T> {}
impl<T> std::hash::Hash for Coroutine<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state)
    }
}
