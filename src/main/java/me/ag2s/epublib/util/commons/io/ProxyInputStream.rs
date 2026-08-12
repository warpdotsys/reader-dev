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

use crate::me::ag2s::epublib::util::IOUtil;

/**
 * A Proxy stream which acts as expected, that is it passes the method
 * calls on to the proxied stream and doesn't change which methods are
 * being called.
 * <p>
 * It is an alternative base class to FilterInputStream
 * to increase reusability, because FilterInputStream changes the
 * methods being called, such as read(byte[]) to read(byte[], int, int).
 * </p>
 * <p>
 * See the protected methods for ways in which a subclass can easily decorate
 * a stream with custom pre-, post- or error processing functionality.
 * </p>
 */
pub struct ProxyInputStream {
    in_stream: Box<dyn InputStream>,
}

impl ProxyInputStream {

    /**
     * Constructs a new ProxyInputStream.
     *
     * @param proxy the InputStream to delegate to
     */
    pub fn new(proxy: Box<dyn InputStream>) -> Self {
        ProxyInputStream { in_stream: proxy }
        // the proxy is stored in a protected superclass variable named 'in'
    }

    /**
     * Invokes the delegate's <code>read()</code> method.
     *
     * @return the byte read or -1 if the end of stream
     * @throws IOException if an I/O error occurs
     */
    pub fn read(&mut self) -> Result<i32, io::Error> {
        match self.read_byte_inner() {
            Ok(b) => Ok(b),
            Err(e) => {
                self.handle_io_exception(e)?;
                Ok(IOUtil::EOF)
            }
        }
    }

    fn read_byte_inner(&mut self) -> Result<i32, io::Error> {
        self.before_read(1);
        let b = self.in_stream.read_byte();
        self.after_read(if b != IOUtil::EOF { 1 } else { IOUtil::EOF });
        Ok(b)
    }

    /**
     * Invokes the delegate's <code>read(byte[])</code> method.
     *
     * @param bts the buffer to read the bytes into
     * @return the number of bytes read or EOF if the end of stream
     * @throws IOException if an I/O error occurs
     */
    pub fn read_bytes(&mut self, bts: &mut [u8]) -> Result<i32, io::Error> {
        match self.read_bytes_inner(bts) {
            Ok(n) => Ok(n),
            Err(e) => {
                self.handle_io_exception(e)?;
                Ok(IOUtil::EOF)
            }
        }
    }

    fn read_bytes_inner(&mut self, bts: &mut [u8]) -> Result<i32, io::Error> {
        self.before_read(IOUtil::length_byte(bts) as i32);
        let n = self.in_stream.read(bts);
        self.after_read(n);
        Ok(n)
    }

    /**
     * Invokes the delegate's <code>read(byte[], int, int)</code> method.
     *
     * @param bts the buffer to read the bytes into
     * @param off The start offset
     * @param len The number of bytes to read
     * @return the number of bytes read or -1 if the end of stream
     * @throws IOException if an I/O error occurs
     */
    pub fn read_off(&mut self, bts: &mut [u8], off: usize, len: usize) -> Result<i32, io::Error> {
        match self.read_off_inner(bts, off, len) {
            Ok(n) => Ok(n),
            Err(e) => {
                self.handle_io_exception(e)?;
                Ok(IOUtil::EOF)
            }
        }
    }

    fn read_off_inner(&mut self, bts: &mut [u8], off: usize, len: usize) -> Result<i32, io::Error> {
        self.before_read(len as i32);
        let n = self.in_stream.read_off(bts, off, len);
        self.after_read(n);
        Ok(n)
    }

    /**
     * Invokes the delegate's <code>skip(long)</code> method.
     *
     * @param ln the number of bytes to skip
     * @return the actual number of bytes skipped
     * @throws IOException if an I/O error occurs
     */
    pub fn skip(&mut self, ln: i64) -> Result<i64, io::Error> {
        match self.in_stream.skip(ln) {
            Ok(n) => Ok(n),
            Err(e) => {
                self.handle_io_exception(e)?;
                Ok(0)
            }
        }
    }

    /**
     * Invokes the delegate's <code>available()</code> method.
     *
     * @return the number of available bytes
     * @throws IOException if an I/O error occurs
     */
    pub fn available(&mut self) -> Result<i32, io::Error> {
        match self.in_stream.available() {
            Ok(n) => Ok(n),
            Err(e) => {
                self.handle_io_exception(e)?;
                Ok(0)
            }
        }
    }

    /**
     * Invokes the delegate's <code>close()</code> method.
     *
     * @throws IOException if an I/O error occurs
     */
    pub fn close(&mut self) -> Result<(), io::Error> {
        match self.in_stream.close() {
            Ok(_) => Ok(()),
            Err(e) => self.handle_io_exception(e),
        }
    }

    /**
     * Invokes the delegate's <code>mark(int)</code> method.
     *
     * @param readlimit read ahead limit
     */
    pub fn mark(&mut self, readlimit: i32) {
        self.in_stream.mark(readlimit);
    }

    /**
     * Invokes the delegate's <code>reset()</code> method.
     *
     * @throws IOException if an I/O error occurs
     */
    pub fn reset(&mut self) -> Result<(), io::Error> {
        match self.in_stream.reset() {
            Ok(_) => Ok(()),
            Err(e) => self.handle_io_exception(e),
        }
    }

    /**
     * Invokes the delegate's <code>markSupported()</code> method.
     *
     * @return true if mark is supported, otherwise false
     */
    pub fn mark_supported(&self) -> bool {
        self.in_stream.mark_supported()
    }

    /**
     * Invoked by the read methods before the call is proxied. The number
     * of bytes that the caller wanted to read (1 for the {@link #read()}
     * method, buffer length for {@link #read(byte[])}, etc.) is given as
     * an argument.
     * <p>
     * Subclasses can override this method to add common pre-processing
     * functionality without having to override all the read methods.
     * The default implementation does nothing.
     * <p>
     * Note this method is <em>not</em> called from {@link #skip(long)} or
     * {@link #reset()}. You need to explicitly override those methods if
     * you want to add pre-processing steps also to them.
     *
     * @param n number of bytes that the caller asked to be read
     * @since 2.0
     */
    #[allow(dead_code)]
    fn before_read(&mut self, n: i32) {
        // no-op
    }

    /**
     * Invoked by the read methods after the proxied call has returned
     * successfully. The number of bytes returned to the caller (or -1 if
     * the end of stream was reached) is given as an argument.
     * <p>
     * Subclasses can override this method to add common post-processing
     * functionality without having to override all the read methods.
     * The default implementation does nothing.
     * <p>
     * Note this method is <em>not</em> called from {@link #skip(long)} or
     * {@link #reset()}. You need to explicitly override those methods if
     * you want to add post-processing steps also to them.
     *
     * @param n number of bytes read, or -1 if the end of stream was reached
     * @since 2.0
     */
    #[allow(dead_code)]
    fn after_read(&mut self, n: i32) {
        // no-op
    }

    /**
     * Handle any IOExceptions thrown.
     * <p>
     * This method provides a point to implement custom exception
     * handling. The default behavior is to re-throw the exception.
     *
     * @param e The IOException thrown
     * @throws IOException if an I/O error occurs
     * @since 2.0
     */
    fn handle_io_exception(&mut self, e: io::Error) -> Result<(), io::Error> {
        Err(e)
    }
}

pub trait InputStream {
    fn read_byte(&mut self) -> i32;
    fn read(&mut self, bts: &mut [u8]) -> i32;
    fn read_off(&mut self, bts: &mut [u8], off: usize, len: usize) -> i32;
    fn skip(&mut self, ln: i64) -> Result<i64, io::Error>;
    fn available(&mut self) -> Result<i32, io::Error>;
    fn close(&mut self) -> Result<(), io::Error>;
    fn mark(&mut self, readlimit: i32);
    fn reset(&mut self) -> Result<(), io::Error>;
    fn mark_supported(&self) -> bool;
}
