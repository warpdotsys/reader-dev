// package me.ag2s.umdlib.domain;
//
//
// import java.io.File;
// import java.io.IOException;
//
// import me.ag2s.umdlib.tool.UmdUtils;
// import me.ag2s.umdlib.tool.WrapOutputStream;

/**
 * This is the cover part of the UMD file.
 * <P>
 * NOTICE: if the "coverData" is empty, it will be skipped when building UMD file.
 * </P>
 * There are 3 ways to load the image data:
 * <ol>
 *     <li>new constructor function of UmdCover.</li>
 *     <li>use UmdCover.load function.</li>
 *     <li>use UmdCover.initDefaultCover, it will generate a simple image with text.</li>
 * </ol>
 * @author Ray Liang (liangguanhui@qq.com)
 * 2009-12-20
 */
pub struct UmdCover {
    default_cover_width: i32,
    default_cover_height: i32,

    pub cover_data: Vec<u8>,
}

impl UmdCover {

    pub fn new() -> Self {
        UmdCover {
            default_cover_width: 120,
            default_cover_height: 160,
            cover_data: Vec::new(),
        }
    }

    pub fn with_data(cover_data: Vec<u8>) -> Self {
        UmdCover {
            default_cover_width: 120,
            default_cover_height: 160,
            cover_data,
        }
    }

    pub fn load(&mut self, f: &File) {
        self.cover_data = UmdUtils::read_file(f);
    }

    pub fn load_file_name(&mut self, file_name: &str) {
        self.load(&File::new(file_name));
    }

    pub fn init_default_cover(&mut self, title: &str) {
        //		BufferedImage img = new BufferedImage(DEFAULT_COVER_WIDTH, DEFAULT_COVER_HEIGHT, BufferedImage.TYPE_INT_RGB);
        //		Graphics g = img.getGraphics();
        //		g.setColor(Color.BLACK);
        //		g.fillRect(0, 0, img.getWidth(), img.getHeight());
        //		g.setColor(Color.WHITE);
        //		g.setFont(new Font("����", Font.PLAIN, 12));
        //
        //		FontMetrics fm = g.getFontMetrics();
        //		int ascent = fm.getAscent();
        //		int descent = fm.getDescent();
        //		int strWidth = fm.stringWidth(title);
        //		int x = (img.getWidth() - strWidth) / 2;
        //		int y = (img.getHeight() - ascent - descent) / 2;
        //		g.drawString(title, x, y);
        //		g.dispose();
        //
        //		ByteArrayOutputStream baos = new ByteArrayOutputStream();
        //
        //		JPEGImageEncoder encoder = JPEGCodec.createJPEGEncoder(baos);
        //		JPEGEncodeParam param = encoder.getDefaultJPEGEncodeParam(img);
        //		param.setQuality(0.5f, false);
        //		encoder.setJPEGEncodeParam(param);
        //		encoder.encode(img);
        //
        //		coverData = baos.toByteArray();
    }

    pub fn build_cover(&self, wos: &mut WrapOutputStream) {
        if self.cover_data.is_empty() {
            return;
        }
        wos.write_bytes(&[b'#', 0x82, 0, 0x01, 0x0A, 0x01]);
        let rb = UmdUtils::gen_random_bytes(4);
        wos.write_bytes(&rb); //random numbers
        wos.write(b'$');
        wos.write_bytes(&rb); //random numbers
        wos.write_int(self.cover_data.len() as i32 + 9);
        wos.write(&self.cover_data);
    }

    pub fn get_cover_data(&self) -> &Vec<u8> {
        return &self.cover_data;
    }

    pub fn set_cover_data(&mut self, cover_data: Vec<u8>) {
        self.cover_data = cover_data;
    }
}
