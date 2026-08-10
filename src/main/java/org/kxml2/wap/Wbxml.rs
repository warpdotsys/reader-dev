/* Copyright (c) 2002,2003, Stefan Haustein, Oberhausen, Rhld., Germany
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or
 * sell copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The  above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
 * IN THE SOFTWARE. */

// package org.kxml2.wap;


/** contains the WBXML constants  */


pub trait Wbxml {

    const SWITCH_PAGE: i32 = 0;
    const END: i32 = 1;
    const ENTITY: i32 = 2;
    const STR_I: i32 = 3;
    const LITERAL: i32 = 4;
    const EXT_I_0: i32 = 0x40;
    const EXT_I_1: i32 = 0x41;
    const EXT_I_2: i32 = 0x42;
    const PI: i32 = 0x43;
    const LITERAL_C: i32 = 0x44;
    const EXT_T_0: i32 = 0x80;
    const EXT_T_1: i32 = 0x81;
    const EXT_T_2: i32 = 0x82;
    const STR_T: i32 = 0x83;
    const LITERAL_A: i32 = 0x084;
    const EXT_0: i32 = 0x0c0;
    const EXT_1: i32 = 0x0c1;
    const EXT_2: i32 = 0x0c2;
    const OPAQUE: i32 = 0x0c3;
    const LITERAL_AC: i32 = 0x0c4;
}
