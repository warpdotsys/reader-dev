use crate::prelude::*;
use crate::stubs::Any;
use crate::stubs::TextUtils;
// package io.legado.app.model.analyzeRule
// import io.legado.app.utils.splitNotBlank
// import io.legado.app.utils.TextUtils
// import org.jsoup.nodes.Document
// import org.jsoup.nodes.Element
// import org.jsoup.select.Elements
// import org.seimicrawler.xpath.JXDocument
// import org.seimicrawler.xpath.JXNode
// import java.util.*

pub struct AnalyzeByXPath {
    // private var jxNode: Any = parse(doc)
    jx_node: Any,
}

impl AnalyzeByXPath {
    pub fn new(doc: &Any) -> AnalyzeByXPath {
        AnalyzeByXPath {
            jx_node: Self::parse(doc),
        }
    }

    fn parse(doc: &Any) -> Any {
        // fix: Kotlin when(is X -> ...) 智能转换 → 占位 Any 枚举匹配；非元素 JXNode 及文本输入转为 JXDocument 变体
        return match doc {
            Any::JXNode(n) => {
                if n.is_element() {
                    Any::JXNode(n.clone())
                } else {
                    Any::JXDocument(Self::str_to_jx_document(n.to_string()))
                }
            }
            Any::Document(d) => Any::JXDocument(JXDocument::create(d.clone())),
            // fix: Kotlin Elements(doc) 单元素构造 → Elements::new_single
            Any::Element(e) => Any::JXDocument(JXDocument::create(Elements::new_single(e.clone()))),
            Any::Elements(es) => Any::JXDocument(JXDocument::create(es.clone())),
            _ => Any::JXDocument(Self::str_to_jx_document(doc.to_string())),
        }
    }

    // fix: Kotlin `strToJXDocument(html: String)` 原参数即为 String
    fn str_to_jx_document(html: String) -> JXDocument {
        let mut html1 = html;
        if html1.ends_with("</td>") {
            html1 = format!("<tr>{}</tr>", html1);
        }
        if html1.ends_with("</tr>") || html1.ends_with("</tbody>") {
            html1 = format!("<table>{}</table>", html1);
        }
        return JXDocument::create(html1);
    }

    pub fn get_result(&self, x_path: &str) -> Option<List<JXNode>> {
        let node = &self.jx_node;
        // fix: Kotlin `node is JXNode` 智能转换 → match 占位 Any 枚举
        return match node {
            Any::JXNode(_) => node.as_jx_node().sel(x_path),
            _ => node.as_jx_document().sel_n(x_path),
        }
    }

    pub fn get_elements(&self, x_path: &str) -> Option<List<JXNode>> {
        if x_path.is_empty() { return None; }

        let mut jx_nodes = ArrayList::<JXNode>::new();
        // fix: Kotlin `RuleAnalyzer(xPath)` 默认参数 code = false
        let mut rule_analyzes = RuleAnalyzer::new(x_path.to_string(), false);
        let rules = rule_analyzes.split_rule(&["&&", "||", "%%"]);

        if rules.len() == 1 {
            return self.get_result(&rules[0]);
        } else {
            let mut results = ArrayList::<List<JXNode>>::new();
            for rl in rules {
                let temp = self.get_elements(&rl);
                if temp.is_some() && !temp.as_ref().unwrap().is_empty() {
                    results.add(temp.unwrap());
                    // fix: 原 `!temp.as_ref().unwrap().is_empty() && ...` 与 if 条件重复，且 temp 已 move，直接判断 ||
                    if rule_analyzes.elements_type == "||" {
                        break;
                    }
                }
            }
            if results.len() > 0 {
                if "%%" == rule_analyzes.elements_type {
                    for i in results[0].indices() {
                        for temp in &results {
                            if i < temp.len() {
                                jx_nodes.add(temp[i].clone());
                            }
                        }
                    }
                } else {
                    for temp in &results {
                        jx_nodes.add_all(temp.clone());
                    }
                }
            }
        }
        return Some(jx_nodes);
    }

    pub fn get_string_list(&self, x_path: &str) -> List<String> {
        let mut result = ArrayList::<String>::new();
        // fix: Kotlin `RuleAnalyzer(xPath)` 默认参数 code = false
        let mut rule_analyzes = RuleAnalyzer::new(x_path.to_string(), false);
        let rules = rule_analyzes.split_rule(&["&&", "||", "%%"]);

        if rules.len() == 1 {
            let nodes = self.get_result(x_path);
            if let Some(nodes) = nodes {
                for node in nodes {
                    result.add(node.as_string());
                }
            }
            return result;
        } else {
            let mut results = ArrayList::<List<String>>::new();
            for rl in rules {
                let temp = self.get_string_list(&rl);
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
                        result.add_all(temp.clone());
                    }
                }
            }
        }
        return result;
    }

    pub fn get_string(&self, rule: &str) -> Option<String> {
        // fix: Kotlin `RuleAnalyzer(rule)` 默认参数 code = false
        let mut rule_analyzes = RuleAnalyzer::new(rule.to_string(), false);
        let rules = rule_analyzes.split_rule(&["&&", "||"]);
        if rules.len() == 1 {
            let nodes = self.get_result(rule);
            if nodes.is_some() {
                return Some(TextUtils::join("\n", nodes.unwrap()));
            }
            return None;
        } else {
            let mut text_list = array_list_of::<String>();
            for rl in rules {
                let temp = self.get_string(&rl);
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
}
