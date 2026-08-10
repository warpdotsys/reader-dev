// package io.legado.app.model.analyzeRule
// import com.jayway.jsonpath.JsonPath
// import com.jayway.jsonpath.ReadContext
// import java.util.*

// @Suppress("RegExpRedundantEscape")
struct AnalyzeByJSonPath {
    ctx: ReadContext,
}

impl AnalyzeByJSonPath {
    // companion object
    fn parse(json: &Any) -> ReadContext {
        return match json {
            Any::ReadContext(ctx) => ctx.clone(),
            Any::String(s) => JsonPath::parse(s), //JsonPath.parse<String>(json)
            _ => JsonPath::parse(json), //JsonPath.parse<Any>(json)
        }
    }

    fn new(json: &Any) -> AnalyzeByJSonPath {
        // private var ctx: ReadContext = parse(json)
        AnalyzeByJSonPath {
            ctx: Self::parse(json),
        }
    }

    /**
     * 改进解析方法
     * 解决阅读"&&"、"||"与jsonPath支持的"&&"、"||"之间的冲突
     * 解决{$.rule}形式规则可能匹配错误的问题，旧规则用正则解析内容含'}'的json文本时，用规则中的字段去匹配这种内容会匹配错误.现改用平衡嵌套方法解决这个问题
     * */
    fn get_string(&self, rule: &str) -> Option<String> {
        if rule.is_empty() { return None; }
        let mut result: String;
        let rule_analyzes = RuleAnalyzer::new(rule, true); //设置平衡组为代码平衡
        let rules = rule_analyzes.split_rule("&&", "||");

        if rules.len() == 1 {
            rule_analyzes.re_set_pos(); //将pos重置为0，复用解析器

            result = rule_analyzes.inner_rule("{$.") { |it| self.get_string(it) }.unwrap_or_default(); //替换所有{$.rule...}

            if result.is_empty() { //st为空，表明无成功替换的内嵌规则
                try {
                    let ob = self.ctx.read::<Any>(rule);
                    result = if ob is List<*> {
                        ob.join_to_string("\n")
                    } else {
                        ob.to_string()
                    };
                } catch (e: Exception) {
                    e.print_stack_trace();
                }
            }

            return Some(result);
        } else {
            let text_list = array_list_of::<String>();
            for rl in rules {
                let temp = self.get_string(rl);
                if !temp.is_null_or_empty() {
                    text_list.add(temp.unwrap());
                    if rule_analyzes.elements_type == "||" {
                        break;
                    }
                }
            }
            return Some(text_list.join_to_string("\n"));
        }
    }

    fn get_string_list(&self, rule: &str) -> List<String> {
        let result = ArrayList::<String>::new();
        if rule.is_empty() { return result; }
        let rule_analyzes = RuleAnalyzer::new(rule, true); //设置平衡组为代码平衡
        let rules = rule_analyzes.split_rule("&&", "||", "%%");

        if rules.len() == 1 {
            rule_analyzes.re_set_pos(); //将pos重置为0，复用解析器

            let st = rule_analyzes.inner_rule("{$.") { |it| self.get_string(it) }.unwrap_or_default(); //替换所有{$.rule...}

            if st.is_empty() { //st为空，表明无成功替换的内嵌规则
                let obj = (|| -> Option<Any> {
                    try {
                        self.ctx.read::<Any>(rule) //kotlin的Any型返回值不包含null ，删除赘余 ?: return result
                    } catch (e: Exception) {
                        e.print_stack_trace();
                        None
                    }
                })();

                if obj.is_some() {
                    let obj = obj.unwrap();
                    if obj is List<*> {
                        for o in obj { result.add(o.to_string()); }
                    } else {
                        result.add(obj.to_string());
                    }
                }
            } else {
                result.add(st);
            }
            return result;
        } else {
            let mut results = ArrayList::<List<String>>::new();
            for rl in rules {
                let temp = self.get_string_list(rl);
                if !temp.is_empty() {
                    results.add(temp.clone());
                    if !temp.is_empty() && rule_analyzes.elements_type == "||" {
                        break;
                    }
                }
            }
            if results.len() > 0 {
                if "%%" == rule_analyzes.elements_type {
                    for i in results[0].indices() {
                        for temp in &results {
                            if i < temp.len() {
                                result.add(temp[i].clone());
                            }
                        }
                    }
                } else {
                    for temp in &results {
                        for item in temp {
                            result.add(item.clone());
                        }
                    }
                }
            }
            return result;
        }
    }

    fn get_object(&self, rule: &str) -> Any {
        return self.ctx.read(rule);
    }

    fn get_list(&self, rule: &str) -> Option<ArrayList<Any>> {
        let result = ArrayList::<Any>::new();
        if rule.is_empty() { return Some(result); }
        let rule_analyzes = RuleAnalyzer::new(rule, true); //设置平衡组为代码平衡
        let rules = rule_analyzes.split_rule("&&", "||", "%%");
        if rules.len() == 1 {
            try {
                return Some(self.ctx.read::<ArrayList<Any>>(rules[0]));
            } catch (e: Exception) {
                e.print_stack_trace();
            }
        } else {
            let mut results = ArrayList::<ArrayList<Any>>::new();
            for rl in rules {
                let temp = self.get_list(rl);
                if temp.is_some() && !temp.as_ref().unwrap().is_empty() {
                    results.add(temp.unwrap());
                    if !temp.as_ref().unwrap().is_empty() && rule_analyzes.elements_type == "||" {
                        break;
                    }
                }
            }
            if results.len() > 0 {
                if "%%" == rule_analyzes.elements_type {
                    for i in 0..results[0].len() {
                        for temp in &results {
                            if i < temp.len() {
                                result.add(temp[i].clone());
                            }
                        }
                    }
                } else {
                    for temp in &results {
                        result.add_all(temp.clone());
                    }
                }
            }
        }
        return Some(result);
    }
}
