pub struct XmlUtils;

impl XmlUtils {
    pub fn xml2map(source: &dyn Any) -> LinkedHashMap<String, Any> {
        let doc = LinkedHashMap::<String, Any>::new();
        let result: Result<LinkedHashMap<String, Any>, ()> = (|| {
            let builder = DocumentBuilderFactory::newInstance().newDocumentBuilder();
            if source.is_string() {
                Ok(Self::parseNode(&builder.parse_str(source.as_str()).childNodes))
            } else if source.is_input_stream() {
                Ok(Self::parseNode(&builder.parse_stream(source.as_input_stream()).childNodes))
            } else if source.is_input_source() {
                Ok(Self::parseNode(&builder.parse_input_source(source.as_input_source()).childNodes))
            } else {
                Ok(doc)
            }
        })();
        match result {
            Ok(m) => m,
            Err(e) => {
                e.printStackTrace();
                doc
            }
        }
    }

    pub fn parseNode(list: &NodeList) -> LinkedHashMap<String, Any> {
        let mut doc = LinkedHashMap::<String, Any>::new();
        for index in 0..list.length {
            let node = list.item(index);
            if node.nodeType != Node::ELEMENT_NODE {
                continue;
            }

            let children = node.childNodes;
            if children.length == 1 && node.firstChild().nodeType == Node::TEXT_NODE {
                doc.insert(node.nodeName, node.firstChild().nodeValue);
            } else if children.length > 1 {
                doc.insert(node.nodeName, Any::from_map(Self::parseNode(&children)));
            }
        }
        doc
    }
}
