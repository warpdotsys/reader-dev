use crate::prelude::*;
/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::io;

/**
 * Like {@link Consumer} but throws {@link IOException}.
 *
 * @param <T> the type of the input to the operations.
 * @since 2.7
 */
pub trait IOConsumer {

    /**
     * Performs this operation on the given argument.
     *
     * @param t the input argument
     * @throws IOException if an I/O error occurs.
     */
    fn accept(&mut self, t: io::Error) -> Result<(), io::Error>;

    /**
     * Returns a composed {@code IoConsumer} that performs, in sequence, this operation followed by the {@code after}
     * operation. If performing either operation throws an exception, it is relayed to the caller of the composed
     * operation. If performing this operation throws an exception, the {@code after} operation will not be performed.
     *
     * @param after the operation to perform after this operation
     * @return a composed {@code Consumer} that performs in sequence this operation followed by the {@code after}
     *         operation
     * @throws NullPointerException if {@code after} is None
     */
    #[allow(dead_code)]
    fn and_then<'a>(&'a mut self, after: &'a mut dyn IOConsumer) -> Box<dyn IOConsumer + 'a>
    where
        Self: Sized,
    {
        let t: io::Error = io::Error::new(io::ErrorKind::Other, "t");
        {
            let _ = self.accept(t);
        }
        Box::new(AndThenConsumer { first: self, after })
    }
}

pub struct AndThenConsumer<'a> {
    first: &'a mut dyn IOConsumer,
    after: &'a mut dyn IOConsumer,
}

impl<'a> IOConsumer for AndThenConsumer<'a> {
    fn accept(&mut self, t: io::Error) -> Result<(), io::Error> {
        // fix: rustc 1.97 起 io::Error 不再实现 Clone，改用 kind + message 重建等价错误
        let kind = t.kind();
        let msg = t.to_string();
        self.first.accept(t)?;
        self.after.accept(io::Error::new(kind, msg))
    }
}
