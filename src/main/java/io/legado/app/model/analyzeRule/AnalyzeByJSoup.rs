use crate::prelude::*;
use crate::stubs::{Collector, Element, Evaluator};
// package io.legado.app.model.analyzeRule

// import org.jsoup.Jsoup
// import org.jsoup.nodes.Element
// import org.jsoup.select.Collector
// import org.jsoup.select.Elements
// import org.jsoup.select.Evaluator
// import org.seimicrawler.xpath.JXNode
// import java.util.*

// 注：Element / Elements / Jsoup / JXNode / Collector / Evaluator 为 jsoup、xpath 外部依赖占位类型，
// Any 为占位枚举（对应 Kotlin 的 Any，用于 parse() 入参及 indexes 列表中的 Int / Triple 元素）。
// 本文件为纯转录，不参与编译。

// Any 占位枚举: parse() 入参使用 Element / JXNode 变体,
// ElementsSingle.indexes 使用 Int(对应 Int) 与 Triple(对应 Triple<Int?, Int?, Int>) 变体。
pub enum Any {
    Element(Element),
    JXNode(JXNode),
    Int(i32),
    Triple(Option<i32>, Option<i32>, i32),
}

impl std::fmt::Display for Any {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Any::Element(e) => write!(f, "{}", e),
            Any::JXNode(n) => write!(f, "{}", n),
            Any::Int(v) => write!(f, "{}", v),
            Any::Triple(a, b, c) => write!(f, "({:?}, {:?}, {})", a, b, c),
        }
    }
}

// String.startsWith(prefix, ignoreCase = true)
pub fn starts_with_ignore_case(s: &str, prefix: &str) -> bool {
    s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix)
}

// String.isBlank()
pub fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

// 模拟 LinkedHashSet 去重且保持插入顺序 (mutableSetOf<Int>)
pub fn add_index(index_set: &mut Vec<i32>, it: i32) {
    if !index_set.contains(&it) {
        index_set.push(it);
    }
}

/**
 * Created by GKF on 2018/1/25.
 * 书源规则解析
 */
pub struct AnalyzeByJSoup {
    // private var element: Element = parse(doc)
    element: Element,
}

impl AnalyzeByJSoup {
    // companion object {
    /**
     * "class", "id", "tag", "text", "children"
     */
    const VALID_KEYS: [&'static str; 5] = ["class", "id", "tag", "text", "children"];

    // fun parse(doc: Any): Element {
    //     return when (doc) {
    //         is Element -> doc
    //         is JXNode -> if (doc.isElement) doc.asElement() else Jsoup.parse(doc.toString())
    //         else -> Jsoup.parse(doc.toString())
    //     }
    // }
    pub fn parse(doc: Any) -> Element {
        return match doc {
            Any::Element(doc) => doc,
            Any::JXNode(doc) => if doc.is_element() {
                doc.as_element()
            } else {
                Jsoup::parse(doc.to_string()).body()
            },
            _ => Jsoup::parse(doc.to_string()).body(),
        }
    }
    // }

    // 构造函数: element = parse(doc)
    pub fn new(doc: Any) -> AnalyzeByJSoup {
        AnalyzeByJSoup {
            element: Self::parse(doc),
        }
    }

    /**
     * 获取列表
     */
    // internal fun getElements(rule: String) = getElements(element, rule)
    pub fn get_elements(&self, rule: &str) -> Elements {
        Self::get_elements_impl(Some(&self.element), rule)
    }

    /**
     * 合并内容列表,得到内容
     */
    // internal fun getString(ruleStr: String) =
    //     if (ruleStr.isEmpty()) None
    //     else getStringList(ruleStr).takeIf { it.isNotEmpty() }?.joinToString("\n")
    pub fn get_string(&self, rule_str: &str) -> Option<String> {
        if rule_str.is_empty() {
            None
        } else {
            let list = self.get_string_list(rule_str);
            if !list.is_empty() {
                Some(list.join("\n"))
            } else {
                None
            }
        }
    }

    /**
     * 获取一个字符串
     */
    // internal fun getString0(ruleStr: String) =
    //     getStringList(ruleStr).let { if (it.isEmpty()) "" else it[0] }
    pub fn get_string0(&self, rule_str: &str) -> String {
        let list = self.get_string_list(rule_str);
        if list.is_empty() {
            String::new()
        } else {
            list[0].clone()
        }
    }

    /**
     * 获取所有内容列表
     */
    // internal fun getStringList(ruleStr: String): List<String> {
    pub fn get_string_list(&self, rule_str: &str) -> Vec<String> {
        let mut text_s: Vec<String> = Vec::new(); // ArrayList<String>()

        if rule_str.is_empty() {
            return text_s;
        }

        //拆分规则
        let source_rule = SourceRule::new(rule_str);

        if source_rule.elements_rule.is_empty() {
            text_s.push(self.element.data().unwrap_or_default());
        } else {
            let mut rule_analyzes = RuleAnalyzer::new(source_rule.elements_rule.clone(), false);
            let rule_str_s = rule_analyzes.split_rule(&["&&", "||", "%%"]);

            let mut results: Vec<Vec<String>> = Vec::new(); // ArrayList<List<String>>()
            for rule_str_x in rule_str_s {
                let temp: Option<Vec<String>> = if source_rule.is_css {
                    let last_index = rule_str_x.rfind('@').unwrap();
                    Some(self.get_result_last(
                        self.element.select(&rule_str_x[0..last_index]),
                        &rule_str_x[last_index + 1..],
                    ))
                } else {
                    self.get_result_list(&rule_str_x)
                };

                if temp.is_some() && !temp.as_ref().unwrap().is_empty() {
                    results.push(temp.unwrap());
                    if rule_analyzes.elements_type == "||" {
                        break;
                    }
                }
            }
            if results.len() > 0 {
                if "%%" == rule_analyzes.elements_type {
                    for i in 0..results[0].len() {
                        for temp in &results {
                            if i < temp.len() {
                                text_s.push(temp[i].clone());
                            }
                        }
                    }
                } else {
                    for temp in results {
                        text_s.extend(temp);
                    }
                }
            }
        }
        return text_s;
    }

    /**
     * 获取Elements
     */
    // private fun getElements(temp: Element?, rule: String): Elements {
    // 原 Kotlin 中与 getElements(rule) 为同名重载，此处以 _impl 后缀区分
    fn get_elements_impl(temp: Option<&Element>, rule: &str) -> Elements {
        if temp.is_none() || rule.is_empty() {
            return Elements::new(); // Elements()
        }
        let temp = temp.unwrap();

        let mut elements = Elements::new(); // Elements()

        let source_rule = SourceRule::new(rule);
        let mut rule_analyzes = RuleAnalyzer::new(source_rule.elements_rule.clone(), false);
        let rule_str_s = rule_analyzes.split_rule(&["&&", "||", "%%"]);

        let mut elements_list: Vec<Elements> = Vec::new(); // ArrayList<Elements>()
        if source_rule.is_css {
            for rule_str in rule_str_s {
                let temp_s = temp.select(&rule_str);
                elements_list.push(temp_s.clone());
                if temp_s.size() > 0 && rule_analyzes.elements_type == "||" {
                    break;
                }
            }
        } else {
            for rule_str in rule_str_s {
                let mut rs_rule = RuleAnalyzer::new(rule_str.clone(), false);

                rs_rule.trim(); // 修剪当前规则之前的"@"或者空白符

                let rs = rs_rule.split_rule(&["@"]);

                let el = if rs.len() > 1 {
                    let mut el = Elements::new();
                    el.add(temp.clone());
                    for rl in rs {
                        let mut es = Elements::new();
                        for et in el.iter() {
                            es.add_all(Self::get_elements_impl(Some(et), &rl));
                        }
                        el.clear();
                        el.add_all(es);
                    }
                    el
                } else {
                    ElementsSingle::new().get_elements_single(temp, &rule_str)
                };

                elements_list.push(el.clone());
                if el.size() > 0 && rule_analyzes.elements_type == "||" {
                    break;
                }
            }
        }
        if elements_list.len() > 0 {
            if "%%" == rule_analyzes.elements_type {
                for i in 0..elements_list[0].size() {
                    for es in &elements_list {
                        if i < es.size() {
                            elements.add(es.get(i));
                        }
                    }
                }
            } else {
                for es in elements_list {
                    elements.add_all(es);
                }
            }
        }
        return elements;
    }

    /**
     * 获取内容列表
     */
    // private fun getResultList(ruleStr: String): List<String>? {
    fn get_result_list(&self, rule_str: &str) -> Option<Vec<String>> {
        if rule_str.is_empty() {
            return None;
        }

        let mut elements = Elements::new(); // Elements()

        elements.add(self.element.clone());

        let mut rule = RuleAnalyzer::new(rule_str.to_string(), false); //创建解析
        rule.trim(); //修建前置赘余符号

        let rules = rule.split_rule(&["@"]); // 切割成列表

        let last = rules.len() - 1;
        for i in 0..last {
            let mut es = Elements::new();
            for elt in elements.iter() {
                es.add_all(ElementsSingle::new().get_elements_single(elt, &rules[i]));
            }
            elements.clear();
            elements = es;
        }
        return if elements.is_empty() {
            None
        } else {
            Some(self.get_result_last(elements, &rules[last]))
        };
    }

    /**
     * 根据最后一个规则获取内容
     */
    // private fun getResultLast(elements: Elements, lastRule: String): List<String> {
    fn get_result_last(&self, elements: Elements, last_rule: &str) -> Vec<String> {
        let mut text_s: Vec<String> = Vec::new(); // ArrayList<String>()
        match last_rule {
            "text" => {
                for element in elements.iter() {
                    let text = element.text();
                    if !text.is_empty() {
                        text_s.push(text);
                    }
                }
            }
            "textNodes" => {
                for element in elements.iter() {
                    let mut tn: Vec<String> = Vec::new(); // arrayListOf<String>()
                    let content_es = element.text_nodes();
                    for item in content_es.iter() {
                        let text = item.text.clone().trim_matches(|c| c <= ' ').to_string();
                        if !text.is_empty() {
                            tn.push(text);
                        }
                    }
                    if !tn.is_empty() {
                        text_s.push(tn.join("\n"));
                    }
                }
            }
            "ownText" => {
                for element in elements.iter() {
                    let text = element.own_text();
                    if !text.is_empty() {
                        text_s.push(text);
                    }
                }
            }
            "html" => {
                // fix: 剔除 script/style 后取 HTML（Kotlin jsoup 从 DOM 移除；Element 仅有 html 字符串——正则剔除）
                for element in elements.iter() {
                    let html = crate::runtime::html::strip_script_style_html(&element.outer_html());
                    if !html.is_empty() {
                        text_s.push(html);
                    }
                }
            }
            "all" => text_s.push(elements.outer_html()),
            _ => {
                for element in elements.iter() {
                    // CSS 选择器优先（div.content 等），否则按属性读取
                    let sel = element.select(last_rule);
                    if sel.size() > 0 {
                        for e in sel.iter() {
                            let t = e.text();
                            if !t.is_empty() && !text_s.contains(&t) {
                                text_s.push(t);
                            }
                        }
                    } else {
                        let url = element.attr(last_rule);
                        if is_blank(&url) || text_s.contains(&url) {
                            continue;
                        }
                        text_s.push(url);
                    }
                }
            }
        }
        return text_s;
    }
}

/**
 * 1.支持阅读原有写法，':'分隔索引，!或.表示筛选方式，索引可为负数
 * 例如 tag.div.-1:10:2 或 tag.div!0:3
 *
 * 2. 支持与jsonPath类似的[]索引写法
 * 格式形如 [it,it，。。。] 或 [!it,it，。。。] 其中[!开头表示筛选方式为排除，it为单个索引或区间。
 * 区间格式为 start:end 或 start:end:step，其中start为0可省略，end为-1可省略。
 * 索引，区间两端及间隔都支持负数
 * 例如 tag.div[-1, 3:-2:-10, 2]
 * 特殊用法 tag.div[-1:0] 可在任意地方让列表反向
 * */
// data class ElementsSingle(
//     var split: Char = '.',
//     var beforeRule: String = "",
//     val indexDefault: MutableList<Int> = mutableListOf(),
//     val indexes: MutableList<Any> = mutableListOf()
// ) {
pub struct ElementsSingle {
    split: char, // var split: Char = '.'
    before_rule: String, // var beforeRule: String = ""
    index_default: Vec<i32>, // val indexDefault: MutableList<Int> = mutableListOf()
    indexes: Vec<Any>, // val indexes: MutableList<Any> = mutableListOf()
}

impl ElementsSingle {
    fn new() -> ElementsSingle {
        ElementsSingle {
            split: '.',
            before_rule: String::new(),
            index_default: Vec::new(),
            indexes: Vec::new(),
        }
    }

    /**
     * 获取Elements按照一个规则
     */
    // fun getElementsSingle(temp: Element, rule: String): Elements {
    fn get_elements_single(&mut self, temp: &Element, rule: &str) -> Elements {
        self.find_index_set(rule); //执行索引列表处理器

        /**
         * 获取所有元素
         */
        let mut elements = if self.before_rule.is_empty() {
            temp.children() //允许索引直接作为根元素，此时前置规则为空，效果与children相同
        } else {
            let rules: Vec<&str> = self.before_rule.split('.').collect();
            match rules[0] {
                "children" => temp.children(), //允许索引直接作为根元素，此时前置规则为空，效果与children相同
                "class" => temp.get_elements_by_class(rules[1]),
                "tag" => temp.get_elements_by_tag(rules[1]),
                "id" => Collector::collect(Evaluator::Id(rules[1].to_string()), temp),
                "text" => temp.get_elements_containing_own_text(rules[1]),
                _ => temp.select(&self.before_rule),
            }
        };

        let len = elements.size() as i32;
        let last_indexes = {
            let a = self.index_default.len() as i32 - 1;
            if a != -1 {
                a
            } else {
                self.indexes.len() as i32 - 1
            }
        }; // (indexDefault.size - 1).takeIf { it != -1 } ?: indexes.size - 1
        let mut index_set: Vec<i32> = Vec::new(); // mutableSetOf<Int>(), 用 Vec 保持插入顺序以模拟 LinkedHashSet

        /**
         * 获取无重且不越界的索引集合
         */
        if last_indexes >= 0 {
            if self.indexes.is_empty() {
                for ix in (0..=last_indexes).rev() {
                    //indexes为空，表明是非[]式索引，集合是逆向遍历插入的，所以这里也逆向遍历，好还原顺序
                    let it = self.index_default[ix as usize];
                    if it >= 0 && it < len {
                        add_index(&mut index_set, it); //将正数不越界的索引添加到集合
                    } else if it < 0 && len >= -it {
                        add_index(&mut index_set, it + len); //将负数不越界的索引添加到集合
                    }
                }
            } else {
                for ix in (0..=last_indexes).rev() {
                    //indexes不空，表明是[]式索引，集合是逆向遍历插入的，所以这里也逆向遍历，好还原顺序
                    if let Any::Triple(_, _, _) = self.indexes[ix as usize] {
                        //区间
                        let (start_x, end_x, step_x) = match self.indexes[ix as usize] {
                            Any::Triple(s, e, st) => (s, e, st),
                            _ => unreachable!(), //还原储存时的类型
                        };

                        let start = if start_x.is_none() {
                            0 //左端省略表示0
                        } else if start_x.unwrap() >= 0 {
                            if start_x.unwrap() < len {
                                start_x.unwrap()
                            } else {
                                len - 1 //右端越界，设置为最大索引
                            }
                        } else if -start_x.unwrap() <= len {
                            len + start_x.unwrap() /* 将负索引转正 */
                        } else {
                            0 //左端越界，设置为最小索引
                        };

                        let end = if end_x.is_none() {
                            len - 1 //右端省略表示 len - 1
                        } else if end_x.unwrap() >= 0 {
                            if end_x.unwrap() < len {
                                end_x.unwrap()
                            } else {
                                len - 1 //右端越界，设置为最大索引
                            }
                        } else if -end_x.unwrap() <= len {
                            len + end_x.unwrap() /* 将负索引转正 */
                        } else {
                            0 //左端越界，设置为最小索引
                        };

                        if start == end || step_x >= len {
                            //两端相同，区间里只有一个数。或间隔过大，区间实际上仅有首位
                            add_index(&mut index_set, start);
                            continue;
                        }

                        let step = if step_x > 0 {
                            step_x
                        } else if -step_x < len {
                            step_x + len
                        } else {
                            1 //最小正数间隔为1
                        };

                        //将区间展开到集合中,允许列表反向。
                        if end > start {
                            let mut i = start;
                            while i <= end {
                                add_index(&mut index_set, i);
                                i += step;
                            }
                        } else {
                            let mut i = start;
                            while i >= end {
                                add_index(&mut index_set, i);
                                i -= step;
                            }
                        }
                    } else {
                        //单个索引
                        let it = match self.indexes[ix as usize] {
                            Any::Int(v) => v,
                            _ => unreachable!(), //还原储存时的类型
                        };

                        if it >= 0 && it < len {
                            add_index(&mut index_set, it); //将正数不越界的索引添加到集合
                        } else if it < 0 && len >= -it {
                            add_index(&mut index_set, it + len); //将负数不越界的索引添加到集合
                        }
                    }
                }
            }
        }

        /**
         * 根据索引集合筛选元素
         */
        if self.split == '!' {
            //排除
            let mut excluded: Vec<usize> = Vec::new();
            for pc_int in &index_set {
                excluded.push(*pc_int as usize); // 收集需要排除的索引（原 Kotlin 置空后 removeAll { it == null }）
            }
            let mut new_list: Vec<Element> = Vec::new();
            for (i, e) in elements.list.into_iter().enumerate() {
                if !excluded.contains(&i) {
                    new_list.push(e);
                }
            }
            elements.list = new_list;
        } else if self.split == '.' {
            //选择
            let mut es = Elements::new();

            for pc_int in &index_set {
                es.add(elements.get(*pc_int as usize));
            }

            elements = es;
        }

        return elements; //返回筛选结果
    }

    // private fun findIndexSet(rule: String) {
    fn find_index_set(&mut self, rule: &str) {
        let rus = rule.trim_matches(|c| c <= ' ').to_string(); // rule.trim { it <= ' ' }

        let mut len = rus.len() as isize;
        let mut cur_int: Option<i32> = None; //当前数字
        let mut cur_minus = false; //当前数字是否为负
        let mut cur_list: Vec<Option<i32>> = Vec::new(); //当前数字区间
        let mut l = String::new(); //暂存数字字符串

        let head = rus.chars().last() == Some(']'); //是否为常规索引写法

        if head {
            //常规索引写法[index...]
            len -= 1; //跳过尾部']'

            while len >= 0 {
                len -= 1;
                let mut rl = rus.chars().nth(len as usize).unwrap();
                if rl == ' ' {
                    continue; //跳过空格
                }

                if rl.is_ascii_digit() {
                    l.insert(0, rl); //将数值累接入临时字串中，遇到分界符才取出
                } else if rl == '-' {
                    cur_minus = true;
                } else {
                    cur_int = if l.is_empty() {
                        None
                    } else if cur_minus {
                        Some(-l.parse::<i32>().unwrap())
                    } else {
                        Some(l.parse::<i32>().unwrap())
                    }; //当前数字

                    match rl {
                        ':' => cur_list.push(cur_int), //区间右端或区间间隔
                        _ => {
                            //为保证查找顺序，区间和单个索引都添加到同一集合
                            if cur_list.is_empty() {
                                if cur_int.is_none() {
                                    break; //是jsoup选择器而非索引列表，跳出
                                }
                                self.indexes.push(Any::Int(cur_int.unwrap()));
                            } else {
                                //列表最后压入的是区间右端，若列表有两位则最先压入的是间隔
                                self.indexes.push(Any::Triple(
                                    cur_int,
                                    cur_list.last().copied().flatten(),
                                    if cur_list.len() == 2 {
                                        cur_list.first().copied().flatten().unwrap_or(1)
                                    } else {
                                        1
                                    },
                                ));
                                cur_list.clear(); //重置临时列表，避免影响到下个区间的处理
                            }

                            if rl == '!' {
                                self.split = '!';
                                loop {
                                    len -= 1;
                                    rl = rus.chars().nth(len as usize).unwrap();
                                    if !(len > 0 && rl == ' ') {
                                        break;
                                    }
                                } //跳过所有空格
                            }

                            if rl == '[' {
                                self.before_rule = rus[0..len as usize].to_string(); //遇到索引边界，返回结果
                                return;
                            }

                            if rl != ',' {
                                break; //非索引结构，跳出
                            }
                        }
                    }

                    l = String::new(); //清空
                    cur_minus = false; //重置
                }
            }
        } else {
            while len >= 0 {
                //阅读原本写法，逆向遍历,可以无前置规则
                len -= 1;
                let rl = rus.chars().nth(len as usize).unwrap();
                if rl == ' ' {
                    continue; //跳过空格
                }

                if rl.is_ascii_digit() {
                    l.insert(0, rl); //将数值累接入临时字串中，遇到分界符才取出
                } else if rl == '-' {
                    cur_minus = true;
                } else {
                    if rl == '!' || rl == '.' || rl == ':' {
                        //分隔符或起始符
                        self.index_default.push(
                            if cur_minus { -l.parse::<i32>().unwrap() } else { l.parse::<i32>().unwrap() },
                        ); // 当前数字追加到列表

                        if rl != ':' {
                            //rl == '!'  || rl == '.'
                            self.split = rl;
                            self.before_rule = rus[0..len as usize].to_string();
                            return;
                        }
                    } else {
                        break; //非索引结构，跳出循环
                    }

                    l = String::new(); //清空
                    cur_minus = false; //重置
                }
            }
        }

        self.split = ' ';
        self.before_rule = rus;
    }
}

// internal inner class SourceRule(ruleStr: String) {
//     var isCss = false
//     var elementsRule: String = if (ruleStr.startsWith("@CSS:", true)) {
//         isCss = true
//         ruleStr.substring(5).trim { it <= ' ' }
//     } else {
//         ruleStr
//     }
// }
pub struct SourceRule {
    is_css: bool, // var isCss = false
    elements_rule: String, // var elementsRule: String
}

impl SourceRule {
    fn new(rule_str: &str) -> SourceRule {
        let mut is_css = false;
        let elements_rule = if starts_with_ignore_case(rule_str, "@CSS:") {
            is_css = true;
            rule_str[5..].trim_matches(|c| c <= ' ').to_string()
        } else {
            rule_str.to_string()
        };
        SourceRule {
            is_css,
            elements_rule,
        }
    }
}
