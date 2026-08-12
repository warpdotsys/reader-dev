use crate::prelude::*;
// fix: E0659——`Any` 由多个 glob 导入（stubs + 其他模块），显式指定 stubs 版本
use crate::stubs::Any;
// package io.legado.app.help.http
//
// import kotlinx.coroutines.CompletableDeferred
// import kotlinx.coroutines.Deferred
// import retrofit2.*
// import java.lang.reflect.ParameterizedType
// import java.lang.reflect.Type

// class CoroutinesCallAdapterFactory private constructor() : CallAdapter.Factory() {
pub struct CoroutinesCallAdapterFactory {
    // companion object {
    //     fun create(): CoroutinesCallAdapterFactory {
    //         return CoroutinesCallAdapterFactory()
    //     }
    // }
}

impl CoroutinesCallAdapterFactory {
    pub fn create() -> CoroutinesCallAdapterFactory {
        // return CoroutinesCallAdapterFactory()
        CoroutinesCallAdapterFactory {}
    }
}

impl CallAdapterFactory for CoroutinesCallAdapterFactory {
    // override fun get(
    //     returnType: Type,
    //     annotations: Array<out Annotation>,
    //     retrofit: Retrofit
    // ): CallAdapter<*, *>? {
    //     if (Deferred::class.java != getRawType(returnType)) {
    //         return None
    //     }
    //     check(returnType is ParameterizedType) { "Deferred return type must be parameterized as Deferred<Foo> or Deferred<out Foo>" }
    //     val responseType = getParameterUpperBound(0, returnType)
    //
    //     val rawDeferredType = getRawType(responseType)
    //     return if (rawDeferredType == Response::class.java) {
    //         check(responseType is ParameterizedType) { "Response must be parameterized as Response<Foo> or Response<out Foo>" }
    //         ResponseCallAdapter<Any>(
    //             getParameterUpperBound(
    //                 0,
    //                 responseType
    //             )
    //         )
    //     } else {
    //         BodyCallAdapter<Any>(responseType)
    //     }
    // }
    fn get(
        &self,
        return_type: Type,
        annotations: &[Annotation],
        retrofit: &Retrofit,
    ) -> Option<Box<dyn CallAdapter>> {
        // if (Deferred::class.java != getRawType(returnType)) {
        //     return None
        // }
        if Deferred::class != get_raw_type(&return_type) {
            return None;
        }
        // check(returnType is ParameterizedType) { "Deferred return type must be parameterized as Deferred<Foo> or Deferred<out Foo>" }
        assert!(
            is_parameterized_type(&return_type),
            "Deferred return type must be parameterized as Deferred<Foo> or Deferred<out Foo>"
        );
        // val responseType = getParameterUpperBound(0, returnType)
        let response_type = get_parameter_upper_bound(0, &return_type);

        // val rawDeferredType = getRawType(responseType)
        let raw_deferred_type = get_raw_type(&response_type);
        // return if (rawDeferredType == Response::class.java) {
        if raw_deferred_type == Response::class {
            // check(responseType is ParameterizedType) { "Response must be parameterized as Response<Foo> or Response<out Foo>" }
            assert!(
                is_parameterized_type(&response_type),
                "Response must be parameterized as Response<Foo> or Response<out Foo>"
            );
            // ResponseCallAdapter<Any>(
            //     getParameterUpperBound(
            //         0,
            //         responseType
            //     )
            // )
            Some(Box::new(ResponseCallAdapter {
                response_type: get_parameter_upper_bound(0, &response_type),
            }))
        } else {
            // BodyCallAdapter<Any>(responseType)
            Some(Box::new(BodyCallAdapter {
                response_type,
            }))
        }
    }
}

// private class BodyCallAdapter<T>(
//     private val responseType: Type
// ) : CallAdapter<T, Deferred<T>> {
pub struct BodyCallAdapter {
    response_type: Type,
}

impl CallAdapter for BodyCallAdapter {
    // override fun responseType() = responseType
    fn response_type(&self) -> Type {
        self.response_type.clone()
    }

    // override fun adapt(call: Call<T>): Deferred<T> {
    //     val deferred = CompletableDeferred<T>()
    //
    //     deferred.invokeOnCompletion {
    //         if (deferred.isCancelled) {
    //             call.cancel()
    //         }
    //     }
    //
    //     call.enqueue(object : Callback<T> {
    //         override fun onFailure(call: Call<T>, t: Throwable) {
    //             deferred.completeExceptionally(t)
    //         }
    //
    //         override fun onResponse(call: Call<T>, response: Response<T>) {
    //             if (response.isSuccessful) {
    //                 deferred.complete(response.body()!!)
    //             } else {
    //                 deferred.completeExceptionally(HttpException(response))
    //             }
    //         }
    //     })
    //
    //     return deferred
    // }
    fn adapt(&self, call: Call<Any>) -> Deferred {
        // val deferred = CompletableDeferred<T>()
        let deferred = CompletableDeferred::<Body>::new();

        // deferred.invokeOnCompletion {
        //     if (deferred.isCancelled) {
        //         call.cancel()
        //     }
        // }
        deferred.invoke_on_completion(|| {
            if deferred.is_cancelled() {
                call.cancel();
            }
        });

        // call.enqueue(object : Callback<T> { ... })
        // fix: stub 模型下回调不实际触发；克隆句柄避免 move 闭包与返回 deferred 冲突
        let callback_deferred = deferred.clone();
        call.enqueue(move |result: Result<Response, Throwable>| {
            match result {
                Err(t) => {
                    // deferred.completeExceptionally(t)
                    callback_deferred.complete_exceptionally(t);
                }
                Ok(response) => {
                    // if (response.isSuccessful) {
                    //     deferred.complete(response.body()!!)
                    // } else {
                    //     deferred.completeExceptionally(HttpException(response))
                    // }
                    if response.is_successful() {
                        // fix: stub Body 非 Option，Kotlin `body()!!` 直接透传
                        callback_deferred.complete(response.body());
                    } else {
                        callback_deferred.complete_exceptionally(HttpException::new(response));
                    }
                }
            }
        });

        // return deferred
        // fix: E0308——Kotlin 返回 Deferred<T>，stub 模型下 CompletableDeferred<T> 转换为占位 Deferred
        deferred.into()
    }
}

// private class ResponseCallAdapter<T>(
//     private val responseType: Type
// ) : CallAdapter<T, Deferred<Response<T>>> {
pub struct ResponseCallAdapter {
    response_type: Type,
}

impl CallAdapter for ResponseCallAdapter {
    // override fun responseType() = responseType
    fn response_type(&self) -> Type {
        self.response_type.clone()
    }

    // override fun adapt(call: Call<T>): Deferred<Response<T>> {
    //     val deferred = CompletableDeferred<Response<T>>()
    //
    //     deferred.invokeOnCompletion {
    //         if (deferred.isCancelled) {
    //             call.cancel()
    //         }
    //     }
    //
    //     call.enqueue(object : Callback<T> {
    //         override fun onFailure(call: Call<T>, t: Throwable) {
    //             deferred.completeExceptionally(t)
    //         }
    //
    //         override fun onResponse(call: Call<T>, response: Response<T>) {
    //             deferred.complete(response)
    //         }
    //     })
    //
    //     return deferred
    // }
    fn adapt(&self, call: Call<Any>) -> Deferred {
        // val deferred = CompletableDeferred<Response<T>>()
        let deferred = CompletableDeferred::<Response>::new();

        // deferred.invokeOnCompletion {
        //     if (deferred.isCancelled) {
        //         call.cancel()
        //     }
        // }
        deferred.invoke_on_completion(|| {
            if deferred.is_cancelled() {
                call.cancel();
            }
        });

        // call.enqueue(object : Callback<T> {
        //     override fun onFailure(call: Call<T>, t: Throwable) {
        //         deferred.completeExceptionally(t)
        //     }
        //
        //     override fun onResponse(call: Call<T>, response: Response<T>) {
        //         deferred.complete(response)
        //     }
        // })
        // fix: stub 模型下回调不实际触发；克隆句柄避免 move 闭包与返回 deferred 冲突
        let callback_deferred = deferred.clone();
        call.enqueue(move |result: Result<Response, Throwable>| {
            match result {
                Err(t) => {
                    callback_deferred.complete_exceptionally(t);
                }
                Ok(response) => {
                    callback_deferred.complete(response);
                }
            }
        });

        // return deferred
        // fix: E0308——Kotlin 返回 Deferred<Response<T>>，stub 模型下 CompletableDeferred<T> 转换为占位 Deferred
        deferred.into()
    }
}
