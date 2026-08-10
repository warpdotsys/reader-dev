// package io.legado.app.model.analyzeRule

// //通用的规则切分处理
pub struct RuleAnalyzer {
    queue: String, // private var queue: String = data //被处理字符串
    pos: i32, // private var pos = 0 //当前处理到的位置
    start: i32, // private var start = 0 //当前处理字段的开始
    start_x: i32, // private var startX = 0 //当前规则的开始
    rule: Vec<String>, // private var rule = ArrayList<String>()  //分割出的规则列表
    step: i32, // private var step: Int = 0 //分割字符的长度
    pub elements_type: String, // var elementsType = "" //当前分割字符串
    pub inner_type: bool, // var innerType = true //是否为内嵌{{}}
    code: bool, // 构造函数参数 code: Boolean = false, 用于选择平衡组函数
}

impl RuleAnalyzer {
    // companion object {
    //     private const val ESC = '\\'
    // }
    const ESC: char = '\\';

    // class RuleAnalyzer(data: String, code: Boolean = false) {
    pub fn new(data: String, code: bool) -> Self {
        Self {
            queue: data, // 被处理字符串
            pos: 0, // 当前处理到的位置
            start: 0, // 当前处理字段的开始
            start_x: 0, // 当前规则的开始
            rule: Vec::new(), // 分割出的规则列表
            step: 0, // 分割字符的长度
            elements_type: String::new(), // 当前分割字符串
            inner_type: true, // 是否为内嵌{{}}
            code,
        }
    }

    pub fn trim(&mut self) {
        // 修剪当前规则之前的"@"或者空白符
        // 在while里重复设置start和startX会拖慢执行速度，所以先来个判断是否存在需要修剪的字段，最后再一次性设置start和startX
        let q: Vec<char> = self.queue.chars().collect();
        if q[self.pos as usize] == '@' || q[self.pos as usize] < '!' { // queue[pos] == '@' || queue[pos] < '!'
            self.pos += 1;
            while q[self.pos as usize] == '@' || q[self.pos as usize] < '!' {
                self.pos += 1;
            }
            self.start = self.pos; // 开始点推移
            self.start_x = self.pos; // 规则起始点推移
        }
    }

    //将pos重置为0，方便复用
    pub fn re_set_pos(&mut self) {
        self.pos = 0;
        self.start_x = 0;
    }

    /**
     * 从剩余字串中拉出一个字符串，直到但不包括匹配序列
     * @param seq 查找的字符串 **区分大小写**
     * @return 是否找到相应字段。
     */
    pub fn consume_to(&mut self, seq: &str) -> bool {
        self.start = self.pos; // 将处理到的位置设置为规则起点
        // val offset = queue.indexOf(seq, pos)
        let offset = self.queue[self.pos as usize..].find(seq).map(|o| o as i32 + self.pos);
        return if let Some(offset) = offset {
            self.pos = offset;
            true
        } else {
            false
        };
    }

    /**
     * 从剩余字串中拉出一个字符串，直到但不包括匹配序列（匹配参数列表中一项即为匹配），或剩余字串用完。
     * @param seq 匹配字符串序列
     * @return 成功返回true并设置间隔，失败则直接返回fasle
     */
    pub fn consume_to_any(&mut self, seq: &[&str]) -> bool {
        // var pos = pos //声明新变量记录匹配位置，不更改类本身的位置
        let mut pos = self.pos;

        while pos != self.queue.len() as i32 {
            for s in seq {
                // if (queue.regionMatches(pos, s, 0, s.length))
                if self.queue[pos as usize..].starts_with(s) {
                    self.step = s.len() as i32; // 间隔数
                    self.pos = pos; // 匹配成功, 同步处理位置到类
                    return true; // 匹配就返回 true
                }
            }
            pos += 1; // 逐个试探
        }
        return false;
    }

    /**
     * 从剩余字串中拉出一个字符串，直到但不包括匹配序列（匹配参数列表中一项即为匹配），或剩余字串用完。
     * @param seq 匹配字符序列
     * @return 返回匹配位置
     */
    fn find_to_any(&mut self, seq: &[char]) -> i32 {
        // var pos = pos //声明新变量记录匹配位置，不更改类本身的位置
        let mut pos = self.pos;

        while pos != self.queue.len() as i32 {
            for s in seq {
                if self.queue.chars().nth(pos as usize) == Some(*s) {
                    return pos; // 匹配则返回位置
                }
            }
            pos += 1; // 逐个试探
        }

        return -1;
    }

    /**
     * 拉出一个非内嵌代码平衡组，存在转义文本
     */
    pub fn chomp_code_balanced(&mut self, open: char, close: char) -> bool {
        // var pos = pos //声明临时变量记录匹配位置，匹配成功后才同步到类的pos
        let mut pos = self.pos;

        let mut depth: i32 = 0; // 嵌套深度
        let mut other_depth: i32 = 0; // 其他对称符合嵌套深度

        let mut in_single_quote = false; // 单引号
        let mut in_double_quote = false; // 双引号

        loop {
            // do {
            if pos == self.queue.len() as i32 {
                break; // if (pos == queue.length) break
            }
            let c = self.queue.chars().nth(pos as usize).unwrap(); // val c = queue[pos++]
            pos += 1;
            if c != Self::ESC {
                //非转义字符
                if c == '\'' && !in_double_quote {
                    in_single_quote = !in_single_quote; // 匹配具有语法功能的单引号
                } else if c == '"' && !in_single_quote {
                    in_double_quote = !in_double_quote; // 匹配具有语法功能的双引号
                }

                if in_single_quote || in_double_quote {
                    continue; // 语法单元未匹配结束，直接进入下个循环
                }

                if c == '[' {
                    depth += 1; // 开始嵌套一层
                } else if c == ']' {
                    depth -= 1; // 闭合一层嵌套
                } else if depth == 0 {
                    //处于默认嵌套中的非默认字符不需要平衡，仅depth为0时默认嵌套全部闭合，此字符才进行嵌套
                    if c == open {
                        other_depth += 1;
                    } else if c == close {
                        other_depth -= 1;
                    }
                }
            } else {
                pos += 1;
            }
            if depth <= 0 && other_depth <= 0 {
                break; // } while (depth > 0 || otherDepth > 0) //拉出一个平衡字串
            }
        }

        return if depth > 0 || other_depth > 0 {
            false
        } else {
            self.pos = pos; // 同步位置
            true
        };
    }

    /**
     * 拉出一个规则平衡组，经过仔细测试xpath和jsoup中，引号内转义字符无效。
     */
    pub fn chomp_rule_balanced(&mut self, open: char, close: char) -> bool {
        // var pos = pos //声明临时变量记录匹配位置，匹配成功后才同步到类的pos
        let mut pos = self.pos;
        let mut depth: i32 = 0; // 嵌套深度
        let mut in_single_quote = false; // 单引号
        let mut in_double_quote = false; // 双引号

        loop {
            // do {
            if pos == self.queue.len() as i32 {
                break; // if (pos == queue.length) break
            }
            let c = self.queue.chars().nth(pos as usize).unwrap(); // val c = queue[pos++]
            pos += 1;
            if c == '\'' && !in_double_quote {
                in_single_quote = !in_single_quote; // 匹配具有语法功能的单引号
            } else if c == '"' && !in_single_quote {
                in_double_quote = !in_double_quote; // 匹配具有语法功能的双引号
            }

            if in_single_quote || in_double_quote {
                continue; // 语法单元未匹配结束，直接进入下个循环
            } else if c == '\\' {
                //不在引号中的转义字符才将下个字符转义
                pos += 1;
                continue;
            }

            if c == open {
                depth += 1; // 开始嵌套一层
            } else if c == close {
                depth -= 1; // 闭合一层嵌套
            }
            if depth <= 0 {
                break; // } while (depth > 0) //拉出一个平衡字串
            }
        }

        return if depth > 0 {
            false
        } else {
            self.pos = pos; // 同步位置
            true
        };
    }

    /**
     * 不用正则,不到最后不切片也不用中间变量存储,只在序列中标记当前查找字段的开头结尾,到返回时才切片,高效快速准确切割规则
     * 解决jsonPath自带的"&&"和"||"与阅读的规则冲突,以及规则正则或字符串中包含"&&"、"||"、"%%"、"@"导致的冲突
     */
    // tailrec fun splitRule(vararg split: String): ArrayList<String> { //首段匹配,elementsType为空
    pub fn split_rule(&mut self, split: &[&str]) -> Vec<String> {
        if split.len() == 1 {
            self.elements_type = split[0].to_string(); // 设置分割字串
            return if !self.consume_to(&self.elements_type) {
                self.rule.push(self.queue[self.start_x as usize..].to_string()); // rule += queue.substring(startX)
                self.rule.clone()
            } else {
                self.step = self.elements_type.len() as i32; // 设置分隔符长度
                self.split_rule_next()
            }; // 递归匹配
        } else if !self.consume_to_any(split) {
            //未找到分隔符
            self.rule.push(self.queue[self.start_x as usize..].to_string()); // rule += queue.substring(startX)
            return self.rule.clone();
        }

        let end = self.pos; // 记录分隔位置
        self.pos = self.start; // 重回开始，启动另一种查找

        loop {
            let st = self.find_to_any(&['[', '(']); // 查找筛选器位置

            if st == -1 {
                self.rule = vec![self.queue[self.start_x as usize..end as usize].to_string()]; // rule = arrayListOf(queue.substring(startX, end)) //压入分隔的首段规则到数组
                self.elements_type = self.queue[end as usize..(end + self.step) as usize].to_string(); // 设置组合类型
                self.pos = end + self.step; // 跳过分隔符

                while self.consume_to(&self.elements_type) {
                    //循环切分规则压入数组
                    self.rule.push(self.queue[self.start as usize..self.pos as usize].to_string()); // rule += queue.substring(start, pos)
                    self.pos += self.step; // 跳过分隔符
                }

                self.rule.push(self.queue[self.pos as usize..].to_string()); // 将剩余字段压入数组末尾

                return self.rule.clone();
            }

            if st > end {
                //先匹配到st1pos，表明分隔字串不在选择器中，将选择器前分隔字串分隔的字段依次压入数组
                self.rule = vec![self.queue[self.start_x as usize..end as usize].to_string()]; // rule = arrayListOf(queue.substring(startX, end)) //压入分隔的首段规则到数组
                self.elements_type = self.queue[end as usize..(end + self.step) as usize].to_string(); // 设置组合类型
                self.pos = end + self.step; // 跳过分隔符

                while self.consume_to(&self.elements_type) && self.pos < st {
                    //循环切分规则压入数组
                    self.rule.push(self.queue[self.start as usize..self.pos as usize].to_string()); // rule += queue.substring(start, pos)
                    self.pos += self.step; // 跳过分隔符
                }

                return if self.pos > st {
                    self.start_x = self.start;
                    self.split_rule(split) // 首段已匹配,但当前段匹配未完成,调用二段匹配
                } else {
                    //执行到此，证明后面再无分隔字符
                    self.rule.push(self.queue[self.pos as usize..].to_string()); // 将剩余字段压入数组末尾
                    self.rule.clone()
                };
            }

            self.pos = st; // 位置推移到筛选器处
            let next = if self.queue.chars().nth(self.pos as usize) == Some('[') {
                ']'
            } else {
                ')'
            }; // 平衡组末尾字符

            if !self.chomp_balanced(self.queue.chars().nth(self.pos as usize).unwrap(), next) {
                // throw Error(queue.substring(0, start) + "后未平衡") //拉出一个筛选器,不平衡则报错
                panic!("{}后未平衡", &self.queue[..self.start as usize]);
            }

            if end <= self.pos {
                break; // } while (end > pos)
            }
        }

        self.start = self.pos; // 设置开始查找筛选器位置的起始位置

        return self.split_rule(split); // 递归调用首段匹配
    }

    // @JvmName("splitRuleNext")
    // private tailrec fun splitRule(): ArrayList<String> { //二段匹配被调用,elementsType非空(已在首段赋值),直接按elementsType查找,比首段采用的方式更快
    fn split_rule_next(&mut self) -> Vec<String> {
        let end = self.pos; // 记录分隔位置
        self.pos = self.start; // 重回开始，启动另一种查找

        loop {
            let st = self.find_to_any(&['[', '(']); // 查找筛选器位置

            if st == -1 {
                self.rule.push(self.queue[self.start_x as usize..end as usize].to_string()); // rule += arrayOf(queue.substring(startX, end)) //压入分隔的首段规则到数组
                self.pos = end + self.step; // 跳过分隔符

                while self.consume_to(&self.elements_type) {
                    //循环切分规则压入数组
                    self.rule.push(self.queue[self.start as usize..self.pos as usize].to_string()); // rule += queue.substring(start, pos)
                    self.pos += self.step; // 跳过分隔符
                }

                self.rule.push(self.queue[self.pos as usize..].to_string()); // 将剩余字段压入数组末尾

                return self.rule.clone();
            }

            if st > end {
                //先匹配到st1pos，表明分隔字串不在选择器中，将选择器前分隔字串分隔的字段依次压入数组
                self.rule.push(self.queue[self.start_x as usize..end as usize].to_string()); // rule += arrayListOf(queue.substring(startX, end)) //压入分隔的首段规则到数组
                self.pos = end + self.step; // 跳过分隔符

                while self.consume_to(&self.elements_type) && self.pos < st {
                    //循环切分规则压入数组
                    self.rule.push(self.queue[self.start as usize..self.pos as usize].to_string()); // rule += queue.substring(start, pos)
                    self.pos += self.step; // 跳过分隔符
                }

                return if self.pos > st {
                    self.start_x = self.start;
                    self.split_rule_next() // 首段已匹配,但当前段匹配未完成,调用二段匹配
                } else {
                    //执行到此，证明后面再无分隔字符
                    self.rule.push(self.queue[self.pos as usize..].to_string()); // 将剩余字段压入数组末尾
                    self.rule.clone()
                };
            }

            self.pos = st; // 位置推移到筛选器处
            let next = if self.queue.chars().nth(self.pos as usize) == Some('[') {
                ']'
            } else {
                ')'
            }; // 平衡组末尾字符

            if !self.chomp_balanced(self.queue.chars().nth(self.pos as usize).unwrap(), next) {
                // throw Error(queue.substring(0, start) + "后未平衡") //拉出一个筛选器,不平衡则报错
                panic!("{}后未平衡", &self.queue[..self.start as usize]);
            }

            if end <= self.pos {
                break; // } while (end > pos)
            }
        }

        self.start = self.pos; // 设置开始查找筛选器位置的起始位置

        // return if (!consumeTo(elementsType)) { rule += queue.substring(startX); rule } else splitRule() // 递归匹配
        return if !self.consume_to(&self.elements_type) {
            self.rule.push(self.queue[self.start_x as usize..].to_string()); // rule += queue.substring(startX)
            self.rule.clone()
        } else {
            self.split_rule_next()
        };
    }

    // val chompBalanced = if (code) ::chompCodeBalanced else ::chompRuleBalanced
    // 设置平衡组函数，json或JavaScript时设置成chompCodeBalanced，否则为chompRuleBalanced
    fn chomp_balanced(&mut self, open: char, close: char) -> bool {
        if self.code {
            self.chomp_code_balanced(open, close)
        } else {
            self.chomp_rule_balanced(open, close)
        }
    }

    /**
     * 替换内嵌规则
     * @param inner 起始标志,如{$.
     * @param startStep 不属于规则部分的前置字符长度，如{$.中{不属于规则的组成部分，故startStep为1
     * @param endStep 不属于规则部分的后置字符长度
     * @param fr 查找到内嵌规则时，用于解析的函数
     *
     * */
    // (Kotlin 重载1: innerRule(inner, startStep = 1, endStep = 1, fr))
    pub fn inner_rule(
        &mut self,
        inner: String,
        start_step: i32,
        end_step: i32,
        fr: impl Fn(String) -> Option<String>,
    ) -> String {
        let mut st = String::new();

        while self.consume_to(&inner) {
            //拉取成功返回true，ruleAnalyzes里的字符序列索引变量pos后移相应位置，否则返回false,且isEmpty为true
            let pos_pre = self.pos; // 记录consumeTo匹配位置
            if self.chomp_code_balanced('{', '}') {
                // val frv = fr(queue.substring(posPre + startStep, pos - endStep))
                let frv = fr(self.queue[(pos_pre + start_step) as usize..(self.pos - end_step) as usize].to_string());
                if !frv.is_none_or(|s| s.is_empty()) {
                    // if (!frv.isNullOrEmpty())
                    st.push_str(&(self.queue[self.start_x as usize..pos_pre as usize].to_string() + &frv.unwrap())); // 压入内嵌规则前的内容，及内嵌规则解析得到的字符串
                    self.start_x = self.pos; // 记录下次规则起点
                    continue; // 获取内容成功，继续选择下个内嵌规则
                }
            }
            self.pos += inner.len() as i32; // 拉出字段不平衡，inner只是个普通字串，跳到此inner后继续匹配
        }

        // return if (startX == 0) "" else st.apply { append(queue.substring(startX)) }.toString()
        return if self.start_x == 0 {
            String::new()
        } else {
            st + &self.queue[self.start_x as usize..]
        };
    }

    /**
     * 替换内嵌规则
     * @param fr 查找到内嵌规则时，用于解析的函数
     *
     * */
    // (Kotlin 重载2: innerRule(startStr, endStr, fr), 与重载1同名不同参)
    pub fn inner_rule(
        &mut self,
        start_str: String,
        end_str: String,
        fr: impl Fn(String) -> Option<String>,
    ) -> String {
        let mut st = String::new();
        while self.consume_to(&start_str) {
            //拉取成功返回true，ruleAnalyzes里的字符序列索引变量pos后移相应位置，否则返回false,且isEmpty为true
            self.pos += start_str.len() as i32; // 跳过开始字符串
            let pos_pre = self.pos; // 记录consumeTo匹配位置
            if self.consume_to(&end_str) {
                // val frv = fr(queue.substring(posPre, pos))
                let frv = fr(self.queue[pos_pre as usize..self.pos as usize].to_string());
                // Kotlin: st.append(queue.substring(startX, posPre - startStr.length) + frv) //"str" + null == "strnull"
                st.push_str(&(self.queue[self.start_x as usize..(pos_pre - start_str.len() as i32) as usize].to_string() + &frv.unwrap_or_default()));
                self.pos += end_str.len() as i32; // 跳过结束字符串
                self.start_x = self.pos; // 记录下次规则起点
            }
        }

        // return if (startX == 0) queue else st.apply { append(queue.substring(startX)) }.toString()
        return if self.start_x == 0 {
            self.queue.clone()
        } else {
            st + &self.queue[self.start_x as usize..]
        };
    }
}
