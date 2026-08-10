// package io.legado.app.model.analyzeRule;
//
// import org.apache.commons.lang3.tuple.Pair;
// import org.apache.commons.lang3.tuple.Triple;
//
// import java.nio.charset.Charset;
// import java.nio.charset.StandardCharsets;
// import java.util.ArrayList;
// import java.util.HashMap;
// import java.util.LinkedList;
// import java.util.List;
// import java.util.Map;

// @SuppressWarnings({"FieldCanBeLocal", "StatementWithEmptyBody", "unused"})
pub struct QueryTTF {
    font_reader: ByteArrayReader,
    file_header: Header, // private final Header fileHeader = new Header()
    directorys: Vec<Directory>, // private final List<Directory> directorys = new LinkedList<>()
    name: NameLayout, // private final NameLayout name = new NameLayout()
    head: HeadLayout, // private final HeadLayout head = new HeadLayout()
    maxp: MaxpLayout, // private final MaxpLayout maxp = new MaxpLayout()
    loca: Vec<i32>, // private final List<Integer> loca = new LinkedList<>()
    cmap: CmapLayout, // private final CmapLayout Cmap = new CmapLayout()
    glyf: Vec<GlyfLayout>, // private final List<GlyfLayout> glyf = new LinkedList<>()
    // @SuppressWarnings("unchecked")
    // private final Pair<Integer, Integer>[] pps = new Pair[]{
    //         Pair.of(3, 10),
    //         Pair.of(0, 4),
    //         Pair.of(3, 1),
    //         Pair.of(1, 0),
    //         Pair.of(0, 3),
    //         Pair.of(0, 1)
    // };
    pps: [(i32, i32); 6], // (Pair.getLeft(), Pair.getRight()) 元组表示
    pub code_to_glyph: HashMap<i32, String>, // public final Map<Integer, String> codeToGlyph = new HashMap<>()
    pub glyph_to_code: HashMap<String, i32>, // public final Map<String, Integer> glyphToCode = new HashMap<>()
    limit_mix: i32, // private int limitMix = 0
    limit_max: i32, // private int limitMax = 0
}

// private static class Header {
struct Header {
    pub major_version: i32,
    pub minor_version: i32,
    pub num_of_tables: i32,
    pub search_range: i32,
    pub entry_selector: i32,
    pub range_shift: i32,
}

// private static class Directory {
struct Directory {
    pub tag: String, // public String tag;          // table name
    pub check_sum: i32, // public int checkSum;       // Check sum
    pub offset: i32, // public int offset;         // Offset from beginning of file
    pub length: i32, // public int length;         // length of the table in bytes
}

// private static class NameLayout {
struct NameLayout {
    pub format: i32,
    pub count: i32,
    pub string_offset: i32,
    pub records: Vec<NameRecord>, // public List<NameRecord> records = new LinkedList<>()
}

// private static class NameRecord {
struct NameRecord {
    pub platform_id: i32, // public int platformID;           // 平台标识符<0:Unicode, 1:Mac, 2:ISO, 3:Windows, 4:Custom>
    pub encoding_id: i32, // public int encodingID;           // 编码标识符
    pub language_id: i32, // public int languageID;           // 语言标识符
    pub name_id: i32, // public int nameID;               // 名称标识符
    pub length: i32, // public int length;               // 名称字符串的长度
    pub offset: i32, // public int offset;               // 名称字符串相对于stringOffset的字节偏移量
}

// private static class HeadLayout {
struct HeadLayout {
    pub major_version: i32,
    pub minor_version: i32,
    pub font_revision: i32,
    pub check_sum_adjustment: i32,
    pub magic_number: i32,
    pub flags: i32,
    pub units_per_em: i32,
    pub created: i64, // public long created
    pub modified: i64, // public long modified
    pub x_min: i16, // public short xMin
    pub y_min: i16, // public short yMin
    pub x_max: i16, // public short xMax
    pub y_max: i16, // public short yMax
    pub mac_style: i32,
    pub lowest_rec_ppem: i32,
    pub font_direction_hint: i16, // public short fontDirectionHint
    pub index_to_loc_format: i16, // public short indexToLocFormat;      // <0:loca是2字节数组, 1:loca是4字节数组>
    pub glyph_data_format: i16, // public short glyphDataFormat
}

// private static class MaxpLayout {
struct MaxpLayout {
    pub major_version: i32,
    pub minor_version: i32,
    pub num_glyphs: i32, // public int numGlyphs;                // 字体中的字形数量
    pub max_points: i32,
    pub max_contours: i32,
    pub max_composite_points: i32,
    pub max_composite_contours: i32,
    pub max_zones: i32,
    pub max_twilight_points: i32,
    pub max_storage: i32,
    pub max_function_defs: i32,
    pub max_instruction_defs: i32,
    pub max_stack_elements: i32,
    pub max_size_of_instructions: i32,
    pub max_component_elements: i32,
    pub max_component_depth: i32,
}

// private static class CmapLayout {
struct CmapLayout {
    pub version: i32,
    pub num_tables: i32,
    pub records: Vec<CmapRecord>, // public List<CmapRecord> records = new LinkedList<>()
    pub tables: HashMap<i32, CmapFormat>, // public Map<Integer, CmapFormat> tables = new HashMap<>()
}

// private static class CmapRecord {
struct CmapRecord {
    pub platform_id: i32,
    pub encoding_id: i32,
    pub offset: i32,
}

// private static class CmapFormat {
struct CmapFormat {
    pub format: i32,
    pub length: i32,
    pub language: i32,
    pub glyph_id_array: Vec<u8>, // public byte[] glyphIdArray
}

// private static class CmapFormat4 extends CmapFormat {
struct CmapFormat4 {
    pub format: i32, // (extends CmapFormat)
    pub length: i32,
    pub language: i32,
    pub seg_count_x2: i32,
    pub search_range: i32,
    pub entry_selector: i32,
    pub range_shift: i32,
    pub end_code: Vec<i32>, // public int[] endCode
    pub reserved_pad: i32,
    pub start_code: Vec<i32>, // public int[] startCode
    pub id_delta: Vec<i16>, // public short[] idDelta
    pub id_range_offset: Vec<i32>, // public int[] idRangeOffset
    pub glyph_id_array: Vec<i32>, // public int[] glyphIdArray
}

// private static class CmapFormat6 extends CmapFormat {
struct CmapFormat6 {
    pub format: i32, // (extends CmapFormat)
    pub length: i32,
    pub language: i32,
    pub first_code: i32,
    pub entry_count: i32,
    pub glyph_id_array: Vec<i32>, // public int[] glyphIdArray
}

// private static class CmapFormat12 extends CmapFormat {
struct CmapFormat12 {
    pub format: i32, // (extends CmapFormat)
    pub reserved: i32,
    pub length: i32,
    pub language: i32,
    pub num_groups: i32,
    pub groups: Vec<(i32, i32, i32)>, // public List<Triple<Integer, Integer, Integer>> groups; (Triple.of(left, middle, right))
}

// private static class GlyfLayout {
struct GlyfLayout {
    pub number_of_contours: i16, // public short numberOfContours;      // 非负值为简单字型,负值为符合字型
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
    pub end_pts_of_contours: Vec<i32>, // public int[] endPtsOfContours;   // length=numberOfContours
    pub instruction_length: i32,
    pub instructions: Vec<u8>, // public byte[] instructions;         // length=instructionLength
    pub flags: Vec<u8>, // public byte[] flags
    pub x_coordinates: Vec<i16>, // public short[] xCoordinates;        // length = flags.length
    pub y_coordinates: Vec<i16>, // public short[] yCoordinates;        // length = flags.length
}

// private static class ByteArrayReader {
struct ByteArrayReader {
    pub index: i32, // public int index
    pub buffer: Vec<u8>, // public byte[] buffer
}

impl ByteArrayReader {
    // public ByteArrayReader(byte[] buffer, int index) {
    pub fn new(buffer: Vec<u8>, index: i32) -> Self {
        Self { index, buffer }
    }

    // public long ReadUIntX(long len) {
    pub fn read_uint_x(&mut self, len: i64) -> i64 {
        let mut result: i64 = 0;
        let mut i: i64 = 0;
        while i < len {
            result <<= 8;
            result |= (self.buffer[self.index as usize] as i64) & 0xFF; // result |= buffer[index++] & 0xFF
            self.index += 1;
            i += 1;
        }
        return result;
    }

    // public long ReadUInt64() {
    pub fn read_uint64(&mut self) -> i64 {
        return self.read_uint_x(8);
    }

    // public int ReadUInt32() {
    pub fn read_uint32(&mut self) -> i32 {
        return self.read_uint_x(4) as i32; // (int) ReadUIntX(4)
    }

    // public int ReadUInt16() {
    pub fn read_uint16(&mut self) -> i32 {
        return self.read_uint_x(2) as i32; // (int) ReadUIntX(2)
    }

    // public short ReadInt16() {
    pub fn read_int16(&mut self) -> i16 {
        return self.read_uint_x(2) as i16; // (short) ReadUIntX(2)
    }

    // public short ReadUInt8() {
    pub fn read_uint8(&mut self) -> i16 {
        return self.read_uint_x(1) as i16; // (short) ReadUIntX(1)
    }

    // public String ReadStrings(int len, Charset charset) {
    pub fn read_strings(&mut self, len: i32, charset: &Charset) -> String {
        let mut result: Vec<u8> = Vec::with_capacity(if len > 0 { len as usize } else { 0 }); // byte[] result = len > 0 ? new byte[len] : null
        let mut i: i32 = 0;
        while i < len {
            result.push(self.buffer[self.index as usize]); // result[i] = buffer[index++]
            self.index += 1;
            i += 1;
        }
        return String::from_utf8_lossy(&result).to_string(); // new String(result, charset)
    }

    // public byte GetByte() {
    pub fn get_byte(&mut self) -> u8 {
        let b = self.buffer[self.index as usize];
        self.index += 1;
        return b;
    }

    // public byte[] GetBytes(int len) {
    pub fn get_bytes(&mut self, len: i32) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::with_capacity(if len > 0 { len as usize } else { 0 }); // byte[] result = len > 0 ? new byte[len] : null
        let mut i: i32 = 0;
        while i < len {
            result.push(self.buffer[self.index as usize]); // result[i] = buffer[index++]
            self.index += 1;
            i += 1;
        }
        return result;
    }

    // public int[] GetUInt16Array(int len) {
    pub fn get_uint16_array(&mut self, len: i32) -> Vec<i32> {
        let mut result: Vec<i32> = Vec::with_capacity(if len > 0 { len as usize } else { 0 }); // int[] result = len > 0 ? new int[len] : null
        let mut i: i32 = 0;
        while i < len {
            result.push(self.read_uint16()); // result[i] = ReadUInt16()
            i += 1;
        }
        return result;
    }

    // public short[] GetInt16Array(int len) {
    pub fn get_int16_array(&mut self, len: i32) -> Vec<i16> {
        let mut result: Vec<i16> = Vec::with_capacity(if len > 0 { len as usize } else { 0 }); // short[] result = len > 0 ? new short[len] : null
        let mut i: i32 = 0;
        while i < len {
            result.push(self.read_int16()); // result[i] = ReadInt16()
            i += 1;
        }
        return result;
    }
}

impl QueryTTF {
    /**
     * 构造函数
     *
     * @param buffer 传入TTF字体二进制数组
     */
    // public QueryTTF(byte[] buffer) {
    pub fn new(buffer: Vec<u8>) -> Self {
        let mut font_reader = ByteArrayReader::new(buffer, 0);
        // 获取文件头
        let mut file_header = Header {
            major_version: font_reader.read_uint16(),
            minor_version: font_reader.read_uint16(),
            num_of_tables: font_reader.read_uint16(),
            search_range: font_reader.read_uint16(),
            entry_selector: font_reader.read_uint16(),
            range_shift: font_reader.read_uint16(),
        };
        // 获取目录
        let mut directorys: Vec<Directory> = Vec::new();
        let mut i: i32 = 0;
        while i < file_header.num_of_tables {
            let mut d = Directory {
                tag: String::new(),
                check_sum: 0,
                offset: 0,
                length: 0,
            };
            d.tag = font_reader.read_strings(4, &Charset::US_ASCII); // d.tag = fontReader.ReadStrings(4, StandardCharsets.US_ASCII)
            d.check_sum = font_reader.read_uint32();
            d.offset = font_reader.read_uint32();
            d.length = font_reader.read_uint32();
            directorys.push(d);
            i += 1;
        }
        // 解析表 name (字体信息,包含版权、名称、作者等...)
        let mut name = NameLayout {
            format: 0,
            count: 0,
            string_offset: 0,
            records: Vec::new(),
        };
        for temp in &directorys {
            if temp.tag == "name" { // Temp.tag.equals("name")
                font_reader.index = temp.offset;
                name.format = font_reader.read_uint16();
                name.count = font_reader.read_uint16();
                name.string_offset = font_reader.read_uint16();
                let mut i: i32 = 0;
                while i < name.count {
                    let mut record = NameRecord {
                        platform_id: 0,
                        encoding_id: 0,
                        language_id: 0,
                        name_id: 0,
                        length: 0,
                        offset: 0,
                    };
                    record.platform_id = font_reader.read_uint16();
                    record.encoding_id = font_reader.read_uint16();
                    record.language_id = font_reader.read_uint16();
                    record.name_id = font_reader.read_uint16();
                    record.length = font_reader.read_uint16();
                    record.offset = font_reader.read_uint16();
                    name.records.push(record);
                    i += 1;
                }
            }
        }
        // 解析表 head (获取 head.indexToLocFormat)
        let mut head = HeadLayout {
            major_version: 0,
            minor_version: 0,
            font_revision: 0,
            check_sum_adjustment: 0,
            magic_number: 0,
            flags: 0,
            units_per_em: 0,
            created: 0,
            modified: 0,
            x_min: 0,
            y_min: 0,
            x_max: 0,
            y_max: 0,
            mac_style: 0,
            lowest_rec_ppem: 0,
            font_direction_hint: 0,
            index_to_loc_format: 0,
            glyph_data_format: 0,
        };
        for temp in &directorys {
            if temp.tag == "head" { // Temp.tag.equals("head")
                font_reader.index = temp.offset;
                head.major_version = font_reader.read_uint16();
                head.minor_version = font_reader.read_uint16();
                head.font_revision = font_reader.read_uint32();
                head.check_sum_adjustment = font_reader.read_uint32();
                head.magic_number = font_reader.read_uint32();
                head.flags = font_reader.read_uint16();
                head.units_per_em = font_reader.read_uint16();
                head.created = font_reader.read_uint64();
                head.modified = font_reader.read_uint64();
                head.x_min = font_reader.read_int16();
                head.y_min = font_reader.read_int16();
                head.x_max = font_reader.read_int16();
                head.y_max = font_reader.read_int16();
                head.mac_style = font_reader.read_uint16();
                head.lowest_rec_ppem = font_reader.read_uint16();
                head.font_direction_hint = font_reader.read_int16();
                head.index_to_loc_format = font_reader.read_int16();
                head.glyph_data_format = font_reader.read_int16();
            }
        }
        // 解析表 maxp (获取 maxp.numGlyphs)
        let mut maxp = MaxpLayout {
            major_version: 0,
            minor_version: 0,
            num_glyphs: 0,
            max_points: 0,
            max_contours: 0,
            max_composite_points: 0,
            max_composite_contours: 0,
            max_zones: 0,
            max_twilight_points: 0,
            max_storage: 0,
            max_function_defs: 0,
            max_instruction_defs: 0,
            max_stack_elements: 0,
            max_size_of_instructions: 0,
            max_component_elements: 0,
            max_component_depth: 0,
        };
        for temp in &directorys {
            if temp.tag == "maxp" { // Temp.tag.equals("maxp")
                font_reader.index = temp.offset;
                maxp.major_version = font_reader.read_uint16();
                maxp.minor_version = font_reader.read_uint16();
                maxp.num_glyphs = font_reader.read_uint16();
                maxp.max_points = font_reader.read_uint16();
                maxp.max_contours = font_reader.read_uint16();
                maxp.max_composite_points = font_reader.read_uint16();
                maxp.max_composite_contours = font_reader.read_uint16();
                maxp.max_zones = font_reader.read_uint16();
                maxp.max_twilight_points = font_reader.read_uint16();
                maxp.max_storage = font_reader.read_uint16();
                maxp.max_function_defs = font_reader.read_uint16();
                maxp.max_instruction_defs = font_reader.read_uint16();
                maxp.max_stack_elements = font_reader.read_uint16();
                maxp.max_size_of_instructions = font_reader.read_uint16();
                maxp.max_component_elements = font_reader.read_uint16();
                maxp.max_component_depth = font_reader.read_uint16();
            }
        }
        // 解析表 loca (轮廓数据偏移地址表)
        let mut loca: Vec<i32> = Vec::new();
        for temp in &directorys {
            if temp.tag == "loca" { // Temp.tag.equals("loca")
                font_reader.index = temp.offset;
                let offset: i32 = if head.index_to_loc_format == 0 { 2 } else { 4 }; // int offset = head.indexToLocFormat == 0 ? 2 : 4
                let mut l: i64 = 0;
                while l < temp.length as i64 {
                    if offset == 2 {
                        loca.push(font_reader.read_uint16() << 1); // loca.add(offset == 2 ? fontReader.ReadUInt16() << 1 : ...)
                    } else {
                        loca.push(font_reader.read_uint32());
                    }
                    l += offset as i64;
                }
            }
        }
        // 解析表 cmap (Unicode编码轮廓索引对照表)
        let mut cmap = CmapLayout {
            version: 0,
            num_tables: 0,
            records: Vec::new(),
            tables: HashMap::new(),
        };
        for temp in &directorys {
            if temp.tag == "cmap" { // Temp.tag.equals("cmap")
                font_reader.index = temp.offset;
                cmap.version = font_reader.read_uint16();
                cmap.num_tables = font_reader.read_uint16();

                let mut i: i32 = 0;
                while i < cmap.num_tables {
                    let mut record = CmapRecord {
                        platform_id: 0,
                        encoding_id: 0,
                        offset: 0,
                    };
                    record.platform_id = font_reader.read_uint16();
                    record.encoding_id = font_reader.read_uint16();
                    record.offset = font_reader.read_uint32();
                    cmap.records.push(record);
                    i += 1;
                }
                let mut i: i32 = 0;
                while i < cmap.num_tables {
                    let fmt_offset = cmap.records[i as usize].offset; // int fmtOffset = Cmap.records.get(i).offset
                    font_reader.index = temp.offset + fmt_offset;
                    let end_index = font_reader.index; // int EndIndex = fontReader.index

                    let format = font_reader.read_uint16(); // int format = fontReader.ReadUInt16()
                    if cmap.tables.contains_key(&fmt_offset) {
                        continue; // if (Cmap.tables.containsKey(fmtOffset)) continue
                    }
                    if format == 0 {
                        // CmapFormat f = new CmapFormat()
                        // f.format = format; f.length = fontReader.ReadUInt16();
                        let length = font_reader.read_uint16();
                        let f = CmapFormat {
                            format,
                            length,
                            language: font_reader.read_uint16(),
                            glyph_id_array: font_reader.get_bytes(length - 6), // f.glyphIdArray = fontReader.GetBytes(f.length - 6)
                        };
                        cmap.tables.insert(fmt_offset, CmapFormat::Base(f)); // Cmap.tables.put(fmtOffset, f)
                    } else if format == 4 {
                        // CmapFormat4 f = new CmapFormat4()
                        let mut f = CmapFormat4 {
                            format,
                            length: font_reader.read_uint16(),
                            language: font_reader.read_uint16(),
                            seg_count_x2: font_reader.read_uint16(),
                            search_range: 0,
                            entry_selector: 0,
                            range_shift: 0,
                            end_code: Vec::new(),
                            reserved_pad: 0,
                            start_code: Vec::new(),
                            id_delta: Vec::new(),
                            id_range_offset: Vec::new(),
                            glyph_id_array: Vec::new(),
                        };
                        let seg_count = f.seg_count_x2 >> 1; // int segCount = f.segCountX2 >> 1
                        f.search_range = font_reader.read_uint16();
                        f.entry_selector = font_reader.read_uint16();
                        f.range_shift = font_reader.read_uint16();
                        f.end_code = font_reader.get_uint16_array(seg_count); // f.endCode = fontReader.GetUInt16Array(segCount)
                        f.reserved_pad = font_reader.read_uint16();
                        f.start_code = font_reader.get_uint16_array(seg_count); // f.startCode = fontReader.GetUInt16Array(segCount)
                        f.id_delta = font_reader.get_int16_array(seg_count); // f.idDelta = fontReader.GetInt16Array(segCount)
                        f.id_range_offset = font_reader.get_uint16_array(seg_count); // f.idRangeOffset = fontReader.GetUInt16Array(segCount)
                        // f.glyphIdArray = fontReader.GetUInt16Array((EndIndex + f.length - fontReader.index) >> 1)
                        f.glyph_id_array = font_reader.get_uint16_array((end_index + f.length - font_reader.index) >> 1);
                        cmap.tables.insert(fmt_offset, CmapFormat::Format4(f)); // Cmap.tables.put(fmtOffset, f)
                    } else if format == 6 {
                        // CmapFormat6 f = new CmapFormat6()
                        let mut f = CmapFormat6 {
                            format,
                            length: font_reader.read_uint16(),
                            language: font_reader.read_uint16(),
                            first_code: font_reader.read_uint16(),
                            entry_count: font_reader.read_uint16(),
                            glyph_id_array: Vec::new(),
                        };
                        f.glyph_id_array = font_reader.get_uint16_array(f.entry_count); // f.glyphIdArray = fontReader.GetUInt16Array(f.entryCount)
                        cmap.tables.insert(fmt_offset, CmapFormat::Format6(f)); // Cmap.tables.put(fmtOffset, f)
                    } else if format == 12 {
                        // CmapFormat12 f = new CmapFormat12()
                        let mut f = CmapFormat12 {
                            format,
                            reserved: font_reader.read_uint16(),
                            length: font_reader.read_uint32(),
                            language: font_reader.read_uint32(),
                            num_groups: font_reader.read_uint32(),
                            groups: Vec::new(), // f.groups = new ArrayList<>(f.numGroups)
                        };
                        let mut n: i32 = 0;
                        while n < f.num_groups {
                            // f.groups.add(Triple.of(fontReader.ReadUInt32(), fontReader.ReadUInt32(), fontReader.ReadUInt32()))
                            f.groups.push((font_reader.read_uint32(), font_reader.read_uint32(), font_reader.read_uint32()));
                            n += 1;
                        }
                        cmap.tables.insert(fmt_offset, CmapFormat::Format12(f)); // Cmap.tables.put(fmtOffset, f)
                    }
                    i += 1;
                }
            }
        }
        // 解析表 glyf (字体轮廓数据表)
        let mut glyf: Vec<GlyfLayout> = Vec::new();
        for temp in &directorys {
            if temp.tag == "glyf" { // Temp.tag.equals("glyf")
                font_reader.index = temp.offset;
                let mut i: i32 = 0;
                while i < maxp.num_glyphs {
                    font_reader.index = temp.offset + loca[i as usize]; // fontReader.index = Temp.offset + loca.get(i)

                    let number_of_contours = font_reader.read_int16(); // short numberOfContours = fontReader.ReadInt16()
                    if number_of_contours > 0 {
                        let mut g = GlyfLayout {
                            number_of_contours,
                            x_min: font_reader.read_int16(),
                            y_min: font_reader.read_int16(),
                            x_max: font_reader.read_int16(),
                            y_max: font_reader.read_int16(),
                            end_pts_of_contours: Vec::new(),
                            instruction_length: 0,
                            instructions: Vec::new(),
                            flags: Vec::new(),
                            x_coordinates: Vec::new(),
                            y_coordinates: Vec::new(),
                        };
                        g.end_pts_of_contours = font_reader.get_uint16_array(number_of_contours as i32); // g.endPtsOfContours = fontReader.GetUInt16Array(numberOfContours)
                        g.instruction_length = font_reader.read_uint16(); // g.instructionLength = fontReader.ReadUInt16()
                        g.instructions = font_reader.get_bytes(g.instruction_length); // g.instructions = fontReader.GetBytes(g.instructionLength)
                        // int flagLength = g.endPtsOfContours[g.endPtsOfContours.length - 1] + 1
                        let flag_length: i32 = g.end_pts_of_contours[g.end_pts_of_contours.len() - 1] + 1;
                        // 获取轮廓点描述标志
                        // g.flags = new byte[flagLength]
                        g.flags = Vec::with_capacity(flag_length as usize);
                        let mut n: i32 = 0;
                        while n < flag_length {
                            g.flags.push(font_reader.get_byte()); // g.flags[n] = fontReader.GetByte()
                            if (g.flags[n as usize] & 0x08) != 0x00 { // (g.flags[n] & 0x08) != 0x00
                                // for (int m = fontReader.ReadUInt8(); m > 0; --m) { g.flags[++n] = g.flags[n - 1]; }
                                let mut m = font_reader.read_uint8();
                                while m > 0 {
                                    n += 1;
                                    g.flags.push(g.flags[(n - 1) as usize]);
                                    m -= 1;
                                }
                            }
                            n += 1;
                        }
                        // 获取轮廓点描述x轴相对值
                        // g.xCoordinates = new short[flagLength]
                        g.x_coordinates = Vec::with_capacity(flag_length as usize);
                        let mut n: i32 = 0;
                        while n < flag_length {
                            // short same = (short) ((g.flags[n] & 0x10) != 0 ? 1 : -1)
                            let same: i16 = if (g.flags[n as usize] & 0x10) != 0 { 1 } else { -1 };
                            if (g.flags[n as usize] & 0x02) != 0 {
                                g.x_coordinates.push((same * font_reader.read_uint8()) as i16); // g.xCoordinates[n] = (short) (same * fontReader.ReadUInt8())
                            } else {
                                // g.xCoordinates[n] = same == 1 ? (short) 0 : fontReader.ReadInt16()
                                g.x_coordinates.push(if same == 1 { 0 } else { font_reader.read_int16() });
                            }
                            n += 1;
                        }
                        // 获取轮廓点描述y轴相对值
                        // g.yCoordinates = new short[flagLength]
                        g.y_coordinates = Vec::with_capacity(flag_length as usize);
                        let mut n: i32 = 0;
                        while n < flag_length {
                            // short same = (short) ((g.flags[n] & 0x20) != 0 ? 1 : -1)
                            let same: i16 = if (g.flags[n as usize] & 0x20) != 0 { 1 } else { -1 };
                            if (g.flags[n as usize] & 0x04) != 0 {
                                g.y_coordinates.push((same * font_reader.read_uint8()) as i16); // g.yCoordinates[n] = (short) (same * fontReader.ReadUInt8())
                            } else {
                                // g.yCoordinates[n] = same == 1 ? (short) 0 : fontReader.ReadInt16()
                                g.y_coordinates.push(if same == 1 { 0 } else { font_reader.read_int16() });
                            }
                            n += 1;
                        }
                        // 相对坐标转绝对坐标
                        // for (int n = 1; n < flagLength; ++n) {
                        //     xCoordinates[n] += xCoordinates[n - 1];
                        //     yCoordinates[n] += yCoordinates[n - 1];
                        // }

                        glyf.push(g); // glyf.add(g)
                    } else {
                        // 复合字体暂未使用
                    }
                    i += 1;
                }
            }
        }

        // 建立Unicode&Glyph双向表
        let mut code_to_glyph: HashMap<i32, String> = HashMap::new();
        let mut glyph_to_code: HashMap<String, i32> = HashMap::new();
        let mut limit_mix: i32 = 0;
        let mut limit_max: i32 = 0;
        // for (int key = 0; key < 130000; ++key) { ... if (gid == 0) continue; ... }
        let mut key: i32 = 0;
        loop {
            if key >= 130000 {
                break;
            }
            if key == 0xFF {
                key = 0x3400;
            }
            let gid = get_glyf_index(&cmap, &pps, key); // int gid = getGlyfIndex(key)
            if gid != 0 {
                // StringBuilder sb = new StringBuilder()
                let mut sb = String::new();
                // 字型数据转String，方便存HashMap
                for b in &glyf[gid as usize].x_coordinates {
                    sb.push_str(&b.to_string()); // sb.append(b)
                }
                for b in &glyf[gid as usize].y_coordinates {
                    sb.push_str(&b.to_string()); // sb.append(b)
                }
                let val = sb; // String val = sb.toString()
                if limit_mix == 0 {
                    limit_mix = key;
                }
                limit_max = key;
                code_to_glyph.insert(key, val.clone()); // codeToGlyph.put(key, val)
                if !glyph_to_code.contains_key(&val) {
                    glyph_to_code.insert(val, key); // glyphToCode.put(val, key)
                }
            }
            key += 1; // ++key (continue 时同样递增)
        }

        Self {
            font_reader,
            file_header,
            directorys,
            name,
            head,
            maxp,
            loca,
            cmap,
            glyf,
            pps: pps(),
            code_to_glyph,
            glyph_to_code,
            limit_mix,
            limit_max,
        }
    }

    /**
     * 获取字体信息 (1=字体名称)
     *
     * @param nameId 传入十进制字体信息索引
     * @return 返回查询结果字符串
     */
    // public String getNameById(int nameId) {
    pub fn get_name_by_id(&mut self, name_id: i32) -> String {
        for temp in &self.directorys {
            if temp.tag != "name" {
                continue; // if (!Temp.tag.equals("name")) continue
            }
            self.font_reader.index = temp.offset;
            break;
        }
        for record in &self.name.records {
            if record.name_id != name_id {
                continue; // if (record.nameID != nameId) continue
            }
            self.font_reader.index += self.name.string_offset + record.offset; // fontReader.index += name.stringOffset + record.offset
            // return fontReader.ReadStrings(record.length, record.platformID == 1 ? StandardCharsets.UTF_8 : StandardCharsets.UTF_16BE)
            let charset = if record.platform_id == 1 { Charset::UTF_8 } else { Charset::UTF_16BE };
            return self.font_reader.read_strings(record.length, &charset);
        }
        return "error".to_string();
    }

    /**
     * 使用Unicode值查找轮廓索引
     *
     * @param code 传入Unicode十进制值
     * @return 返回十进制轮廓索引
     */
    // private int getGlyfIndex(int code) {
    fn get_glyf_index(&self, code: i32) -> i32 {
        return get_glyf_index(&self.cmap, &self.pps, code);
    }
}

// 构造函数中使用的静态版 getGlyfIndex
fn get_glyf_index(cmap: &CmapLayout, pps: &[(i32, i32)], code: i32) -> i32 {
        if code == 0 {
            return 0;
        }
        let mut fmt_key: i32 = 0;
        for item in pps {
            for record in &cmap.records {
                // if ((item.getLeft() == record.platformID) && (item.getRight() == record.encodingID))
                if item.0 == record.platform_id && item.1 == record.encoding_id {
                    fmt_key = record.offset;
                    break;
                }
            }
            if fmt_key > 0 {
                break;
            }
        }
        if fmt_key == 0 {
            return 0;
        }

        let mut glyf_id: i32 = 0;
        // CmapFormat table = Cmap.tables.get(fmtKey)
        let table = cmap.tables.get(&fmt_key).expect("assert table != null"); // assert table != null
        let fmt = table.format();
        if fmt == 0 {
            // if (code < table.glyphIdArray.length) glyfID = table.glyphIdArray[code] & 0xFF
            if code < table.glyph_id_array_len() {
                glyf_id = table.glyph_id_array_get(code) & 0xFF;
            }
        } else if fmt == 4 {
            let tab = match table {
                CmapFormat::Format4(t) => t,
                _ => unreachable!(),
            };
            if code > tab.end_code[tab.end_code.len() - 1] {
                return 0;
            }
            // 二分法查找数值索引
            let mut start: i32 = 0;
            let mut end: i32 = tab.end_code.len() as i32 - 1;
            let mut middle: i32;
            while start + 1 < end {
                middle = (start + end) / 2;
                if tab.end_code[middle as usize] <= code {
                    start = middle;
                } else {
                    end = middle;
                }
            }
            if tab.end_code[start as usize] < code {
                start += 1;
            }
            if code < tab.start_code[start as usize] {
                return 0;
            }
            if tab.id_range_offset[start as usize] != 0 {
                // glyfID = tab.glyphIdArray[code - tab.startCode[start] + (tab.idRangeOffset[start] >> 1) - (tab.idRangeOffset.length - start)]
                glyf_id = tab.glyph_id_array[(code - tab.start_code[start as usize] + (tab.id_range_offset[start as usize] >> 1) - (tab.id_range_offset.len() as i32 - start)) as usize];
            } else {
                glyf_id = code + tab.id_delta[start as usize]; // glyfID = code + tab.idDelta[start]
            }
            glyf_id &= 0xFFFF;
        } else if fmt == 6 {
            let tab = match table {
                CmapFormat::Format6(t) => t,
                _ => unreachable!(),
            };
            let index = code - tab.first_code;
            if index < 0 || index >= tab.glyph_id_array.len() as i32 {
                glyf_id = 0;
            } else {
                glyf_id = tab.glyph_id_array[index as usize];
            }
        } else if fmt == 12 {
            let tab = match table {
                CmapFormat::Format12(t) => t,
                _ => unreachable!(),
            };
            // if (code > tab.groups.get(tab.numGroups - 1).getMiddle()) return 0
            if code > tab.groups[(tab.num_groups - 1) as usize].1 {
                return 0;
            }
            // 二分法查找数值索引
            let mut start: i32 = 0;
            let mut end: i32 = tab.num_groups - 1;
            let mut middle: i32;
            while start + 1 < end {
                middle = (start + end) / 2;
                if tab.groups[middle as usize].0 <= code {
                    start = middle;
                } else {
                    end = middle;
                }
            }
            // if (tab.groups.get(start).getLeft() <= code && code <= tab.groups.get(start).getMiddle())
            if tab.groups[start as usize].0 <= code && code <= tab.groups[start as usize].1 {
                // glyfID = tab.groups.get(start).getRight() + code - tab.groups.get(start).getLeft()
                glyf_id = tab.groups[start as usize].2 + code - tab.groups[start as usize].0;
            }
        }
        return glyf_id;
    }

    /**
     * 判断Unicode值是否在字体范围内
     *
     * @param code 传入Unicode十进制值
     * @return 返回bool查询结果
     */
    // public boolean inLimit(char code) {
    pub fn in_limit(&self, code: u16) -> bool {
        return (self.limit_mix as u16 <= code) && (code < self.limit_max as u16); // (limitMix <= code) && (code < limitMax)
    }

    /**
     * 使用Unicode值获取轮廓数据
     *
     * @param key 传入Unicode十进制值
     * @return 返回轮廓数组的String值
     */
    // public String getGlyfByCode(int key) {
    pub fn get_glyf_by_code(&self, key: i32) -> String {
        return self.code_to_glyph.get(&key).cloned().unwrap_or_default(); // codeToGlyph.getOrDefault(key, "")
    }

    /**
     * 使用轮廓数据获取Unicode值
     *
     * @param val 传入轮廓数组的String值
     * @return 返回Unicode十进制值
     */
    // public int getCodeByGlyf(String val) {
    pub fn get_code_by_glyf(&self, val: String) -> i32 {
        //noinspection ConstantConditions
        return self.glyph_to_code.get(&val).cloned().unwrap_or(0); // glyphToCode.getOrDefault(val, 0)
    }
}

// private final Pair<Integer, Integer>[] pps = new Pair[]{ Pair.of(3, 10), Pair.of(0, 4), Pair.of(3, 1), Pair.of(1, 0), Pair.of(0, 3), Pair.of(0, 1) }
fn pps() -> [(i32, i32); 6] {
    return [
        (3, 10),
        (0, 4),
        (3, 1),
        (1, 0),
        (0, 3),
        (0, 1),
    ];
}

// CmapFormat 子类统一存储(CmapFormat4/6/12 extends CmapFormat)
enum CmapFormat {
    Base(CmapFormat),
    Format4(CmapFormat4),
    Format6(CmapFormat6),
    Format12(CmapFormat12),
}

impl CmapFormat {
    fn format(&self) -> i32 {
        match self {
            CmapFormat::Base(f) => f.format,
            CmapFormat::Format4(f) => f.format,
            CmapFormat::Format6(f) => f.format,
            CmapFormat::Format12(f) => f.format,
        }
    }

    fn glyph_id_array_len(&self) -> i32 {
        match self {
            CmapFormat::Base(f) => f.glyph_id_array.len() as i32,
            _ => 0,
        }
    }

    fn glyph_id_array_get(&self, code: i32) -> i32 {
        match self {
            CmapFormat::Base(f) => f.glyph_id_array[code as usize] as i32,
            _ => 0,
        }
    }
}
