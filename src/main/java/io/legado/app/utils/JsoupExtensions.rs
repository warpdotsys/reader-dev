use crate::prelude::*;
use crate::stubs::{Element, Node, NodeTraversor, NodeVisitor, TextNode};

pub fn textArray(element: &Element) -> Vec<String> {
    let mut sb = StringUtil::borrowBuilder();
    NodeTraversor::traverse(
        &mut NodeVisitorImpl {
            sb: &mut sb,
        },
        element
    );
    let text = StringUtil::releaseBuilder(&mut sb).trim().trim_matches(|c| c <= ' ').to_string();
    text.split_not_blank("\n")
}

pub struct NodeVisitorImpl<'a> {
    sb: &'a mut StringBuilder,
}

impl NodeVisitor for NodeVisitorImpl<'_> {
    fn head(&mut self, node: &Node, _depth: i32) {
        if node.is_TextNode() {
            appendNormalisedText(self.sb, &node.as_text_node());
        } else if node.is_Element() {
            let node = node.as_element();
            if self.sb.length() > 0
                && (node.isBlock() || node.tag().name() == "br")
                && !lastCharIsWhitespace(self.sb)
            {
                self.sb.append("\n");
            }
        }
    }

    fn tail(&mut self, node: &Node, _depth: i32) {
        if node.is_Element() {
            let node = node.as_element();
            if node.isBlock() && node.nextSibling().map_or(false, |n| n.is_TextNode())
                && !lastCharIsWhitespace(self.sb)
            {
                self.sb.append("\n");
            }
        }
    }
}

pub fn appendNormalisedText(sb: &mut StringBuilder, textNode: &TextNode) {
    let text = textNode.wholeText();
    if preserveWhitespace(textNode.parentNode().as_ref()) || textNode.is_CDataNode() {
        sb.append(&text);
    } else {
        StringUtil::appendNormalisedWhitespace(sb, &text, lastCharIsWhitespace(sb));
    }
}

pub fn preserveWhitespace(node: Option<&Node>) -> bool {
    if let Some(node) = node {
        if node.is_Element() {
            let mut el: Option<Element> = Some(node.as_element());
            let mut i = 0;
            loop {
                if el.as_ref().unwrap().tag().preserveWhitespace() {
                    return true;
                }
                el = el.unwrap().parent();
                i += 1;
                if !(i < 6 && el.is_some()) {
                    break;
                }
            }
        }
    }
    false
}

pub fn lastCharIsWhitespace(sb: &StringBuilder) -> bool {
    sb.length() > 0 && sb.to_string().ends_with(' ')
}
