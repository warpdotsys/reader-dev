// package org.kxml2.wap.wml;

// import org.kxml2.wap.*;


/** This class contains the wml coding tables for elements
 *  and attributes needed by the WmlParser.
 */


pub struct Wml;

impl Wml {

    /** Creates a WbxmlParser with the WML code pages set */

    pub fn create_parser() -> WbxmlParser {
        let mut p = WbxmlParser::new();
        p.set_tag_table(0, &TAG_TABLE);
        p.set_attr_start_table(0, &ATTR_START_TABLE);
        p.set_attr_value_table(0, &ATTR_VALUE_TABLE);
        return p;
    }

    pub fn create_serializer() -> WbxmlSerializer {
        let s = WbxmlSerializer::new();
        s.set_tag_table(0, &TAG_TABLE);
        s.set_attr_start_table(0, &ATTR_START_TABLE);
        s.set_attr_value_table(0, &ATTR_VALUE_TABLE);
        return s;
    }
}

    pub const TAG_TABLE: [Option<&'static str>; 59] = [

    None, // 05
    None, // 06
    None, // 07
    None, // 08
    None, // 09
    None, // 0A
    None, // 0B
    None, // 0C
    None, // 0D
    None, // 0E
    None, // 0F

    None, // 10
    None, // 11
    None, // 12
    None, // 13
    None, // 14
    None, // 15
    None, // 16
    None, // 17
    None, // 18
    None, // 19
    None, // 1A
    None, // 1B
    Some("a"),  // 1C
    Some("td"), // 1D
    Some("tr"), // 1E
    Some("table"), // 1F

    Some("p"), // 20
    Some("postfield"), // 21
    Some("anchor"), // 22
    Some("access"), // 23
    Some("b"),  // 24
    Some("big"), // 25
    Some("br"), // 26
    Some("card"), // 27
    Some("do"), // 28
    Some("em"), // 29
    Some("fieldset"), // 2A
    Some("go"), // 2B
    Some("head"), // 2C
    Some("i"), // 2D
    Some("img"), // 2E
    Some("input"), // 2F

    Some("meta"), // 30
    Some("noop"), // 31
    Some("prev"), // 32
    Some("onevent"), // 33
    Some("optgroup"), // 34
    Some("option"), // 35
    Some("refresh"), // 36
    Some("select"), // 37
    Some("small"), // 38
    Some("strong"), // 39
    None, // 3A
    Some("template"), // 3B
    Some("timer"), // 3C
    Some("u"), // 3D
    Some("setvar"), // 3E
    Some("wml"), // 3F
    };


    pub const ATTR_START_TABLE: [Option<&'static str>; 91] = {
    Some("accept-charset"), // 05
    Some("align=bottom"), // 06
    Some("align=center"), // 07
    Some("align=left"), // 08
    Some("align=middle"), // 09
    Some("align=right"), // 0A
    Some("align=top"), // 0B
    Some("alt"), // 0C
    Some("content"), // 0D
    None, // 0E
    Some("domain"), // 0F

    Some("emptyok=false"), // 10
    Some("emptyok=true"), // 11
    Some("format"), // 12
    Some("height"), // 13
    Some("hspace"), // 14
    Some("ivalue"), // 15
    Some("iname"), // 16
    None, // 17
    Some("label"), // 18
    Some("localsrc"), // 19
    Some("maxlength"), // 1A
    Some("method=get"), // 1B
    Some("method=post"), // 1C
    Some("mode=nowrap"), // 1D
    Some("mode=wrap"), // 1E
    Some("multiple=false"), // 1F

    Some("multiple=true"), // 20
    Some("name"), // 21
    Some("newcontext=false"), // 22
    Some("newcontext=true"), // 23
    Some("onpick"), // 24
    Some("onenterbackward"), // 25
    Some("onenterforward"), // 26
    Some("ontimer"), // 27
    Some("optimal=false"), // 28
    Some("optimal=true"), // 29
    Some("path"), // 2A
    None, // 2B
    None, // 2C
    None, // 2D
    Some("scheme"), // 2E
    Some("sendreferer=false"), // 2F

    Some("sendreferer=true"), // 30
    Some("size"), // 31
    Some("src"), // 32
    Some("ordered=true"), // 33
    Some("ordered=false"), // 34
    Some("tabindex"), // 35
    Some("title"), // 36
    Some("type"), // 37
    Some("type=accept"), // 38
    Some("type=delete"), // 39
    Some("type=help"), // 3A
    Some("type=password"), // 3B
    Some("type=onpick"), // 3C
    Some("type=onenterbackward"), // 3D
    Some("type=onenterforward"), // 3E
    Some("type=ontimer"), // 3F

    None, // 40
    None, // 41
    None, // 42
    None, // 43
    None, // 44
    Some("type=options"), // 45
    Some("type=prev"), // 46
    Some("type=reset"), // 47
    Some("type=text"), // 48
    Some("type=vnd."), // 49
    Some("href"), // 4A
    Some("href=http://"), // 4B
    Some("href=https://"), // 4C
    Some("value"), // 4D
    Some("vspace"), // 4E
    Some("width"), // 4F

    Some("xml:lang"), // 50
    None, // 51
    Some("align"), // 52
    Some("columns"), // 53
    Some("class"), // 54
    Some("id"), // 55
    Some("forua=false"), // 56
    Some("forua=true"), // 57
    Some("src=http://"), // 58
    Some("src=https://"), // 59
    Some("http-equiv"), // 5A
    Some("http-equiv=Content-Type"), // 5B
    Some("content=application/vnd.wap.wmlc;charset="), // 5C
    Some("http-equiv=Expires"), // 5D
    None, // 5E
    None, // 5F
    };


    pub const ATTR_VALUE_TABLE: [Option<&'static str>; 29] = {
    Some(".com/"), // 85
    Some(".edu/"), // 86
    Some(".net/"), // 87
    Some(".org/"), // 88
    Some("accept"), // 89
    Some("bottom"), // 8A
    Some("clear"), // 8B
    Some("delete"), // 8C
    Some("help"), // 8D
    Some("http://"), // 8E
    Some("http://www."), // 8F

    Some("https://"), // 90
    Some("https://www."), // 91
    None, // 92
    Some("middle"), // 93
    Some("nowrap"), // 94
    Some("onpick"), // 95
    Some("onenterbackward"), // 96
    Some("onenterforward"), // 97
    Some("ontimer"), // 98
    Some("options"), // 99
    Some("password"), // 9A
    Some("reset"), // 9B
    None, // 9C
    Some("text"), // 9D
    Some("top"), // 9E
    Some("unknown"), // 9F

    Some("wrap"), // A0
    Some("www."), // A1
    };
